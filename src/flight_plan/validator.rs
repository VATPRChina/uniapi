use serde::Serialize;

use crate::adapter::flight::Flight;
use crate::adapter::navdata::{InvalidNavdataError, NavdataAdapter};
use crate::flight_plan::parser::{self, ParserError};
use crate::model::navdata::{
    AnyFix, DirectionRestriction, LevelRestrictionType, PreferredRoute, ResolvedLeg,
};

#[derive(Debug, thiserror::Error)]
pub enum ValidatorError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("parser error: {0}")]
    Parser(#[from] ParserError),
    #[error("navdata error: {0}")]
    Navdata(InvalidNavdataError),
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct WarningMessage {
    pub message_code: WarningMessageCode,
    pub parameter: Option<String>,
    pub field: WarningMessageField,
    pub field_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
pub enum WarningMessageField {
    Equipment,
    Transponder,
    NavigationPerformance,
    Route,
    CruisingLevel,
}

#[derive(Debug, Clone, Copy, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
pub enum WarningMessageCode {
    NoRvsm,
    NoRnav1,
    RnpAr,
    RnpArWithoutRf,
    NoTransponder,
    RouteDirectSegment,
    RouteLegDirection,
    AirwayRequireApproval,
    NotPreferredRoute,
    CruisingLevelMismatch,
    CruisingLevelNotAllowed,
    RouteMatchPreferred,
}

pub async fn validate_route(
    navdata: &NavdataAdapter,
    flight: &Flight,
    legs: &[ResolvedLeg],
) -> Result<Vec<WarningMessage>, ValidatorError> {
    let mut messages = validate_plan(flight);

    let preferred_routes = navdata
        .list_preferred_routes()
        .await
        .map_err(ValidatorError::Navdata)?;
    let matching_route = find_matching_route(navdata, legs, &preferred_routes).await?;
    messages.extend(validate_preferred_route_match(
        flight,
        matching_route.as_ref(),
        &preferred_routes,
    ));

    for (index, leg) in legs.iter().enumerate() {
        messages.extend(validate_leg(leg, index));
    }

    Ok(messages)
}

fn validate_plan(flight: &Flight) -> Vec<WarningMessage> {
    let mut messages = Vec::new();
    if !flight.equipment.contains('W') {
        messages.push(message(
            WarningMessageField::Equipment,
            WarningMessageCode::NoRvsm,
        ));
    }
    if !flight.equipment.contains('R') && (29000..=41100).contains(&flight.cruising_level) {
        messages.push(message(
            WarningMessageField::Equipment,
            WarningMessageCode::NoRnav1,
        ));
    }
    if !flight.navigation_performance.contains("D1")
        && !flight.navigation_performance.contains("D2")
    {
        messages.push(message(
            WarningMessageField::NavigationPerformance,
            WarningMessageCode::NoRnav1,
        ));
    }
    if flight.navigation_performance.contains("T2") {
        messages.push(message(
            WarningMessageField::NavigationPerformance,
            WarningMessageCode::RnpArWithoutRf,
        ));
    }
    if flight.navigation_performance.contains("T1") {
        messages.push(message(
            WarningMessageField::NavigationPerformance,
            WarningMessageCode::RnpAr,
        ));
    }
    messages
}

async fn find_matching_route(
    navdata: &NavdataAdapter,
    legs: &[ResolvedLeg],
    preferred_routes: &[PreferredRoute],
) -> Result<Option<PreferredRoute>, ValidatorError> {
    for preferred_route in preferred_routes {
        let parsed = parser::parse_route(navdata, &preferred_route.raw_route).await?;
        if route_matches_expected(legs, &parsed) {
            return Ok(Some(preferred_route.clone()));
        }
    }
    Ok(None)
}

fn validate_preferred_route_match(
    flight: &Flight,
    preferred_route: Option<&PreferredRoute>,
    preferred_routes: &[PreferredRoute],
) -> Vec<WarningMessage> {
    let mut messages = Vec::new();
    if let Some(route) = preferred_route {
        messages.push(WarningMessage {
            field: WarningMessageField::Route,
            field_index: None,
            message_code: WarningMessageCode::RouteMatchPreferred,
            parameter: Some(if route.is_public() {
                route.raw_route.clone()
            } else {
                flight.raw_route.clone()
            }),
        });

        let actual = level_restriction_type(flight.cruising_level as i32);
        if !level_restriction_matches(actual, route.cruising_level_restriction) {
            messages.push(WarningMessage {
                field: WarningMessageField::CruisingLevel,
                field_index: None,
                message_code: WarningMessageCode::CruisingLevelMismatch,
                parameter: Some(level_restriction_param(route.cruising_level_restriction)),
            });
        }

        if !route.allowed_altitudes.is_empty()
            && !route
                .allowed_altitudes
                .contains(&(flight.cruising_level as i32))
        {
            messages.push(WarningMessage {
                field: WarningMessageField::CruisingLevel,
                field_index: None,
                message_code: WarningMessageCode::CruisingLevelNotAllowed,
                parameter: Some(
                    route
                        .allowed_altitudes
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                ),
            });
        }
    } else if !preferred_routes.is_empty() {
        let mut routes = preferred_routes
            .iter()
            .filter(|route| !route.is_public())
            .collect::<Vec<_>>();
        routes.sort_by_key(|route| route.id);
        messages.push(WarningMessage {
            field: WarningMessageField::Route,
            field_index: None,
            message_code: WarningMessageCode::NotPreferredRoute,
            parameter: Some(
                routes
                    .into_iter()
                    .map(|route| route.raw_route.clone())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        });
    }
    messages
}

fn validate_leg(leg: &ResolvedLeg, index: usize) -> Vec<WarningMessage> {
    let mut messages = Vec::new();

    let is_from_zh = leg
        .from
        .identifier()
        .is_none_or(|identifier| identifier.starts_with('Z'));
    let is_to_zh = leg
        .to
        .identifier()
        .is_none_or(|identifier| identifier.starts_with('Z'));

    if let Some(ident) = &leg.identifier {
        if leg.direction_restriction == DirectionRestriction::Backward {
            messages.push(route_indexed_message(
                index,
                WarningMessageCode::RouteLegDirection,
            ));
        }

        let from_zh = leg
            .from
            .identifier()
            .is_some_and(|identifier| identifier.starts_with('Z'));
        let to_zh = leg
            .to
            .identifier()
            .is_some_and(|identifier| identifier.starts_with('Z'));
        if from_zh && to_zh && (ident.starts_with('V') || ident.starts_with('X')) {
            messages.push(route_indexed_message(
                index,
                WarningMessageCode::AirwayRequireApproval,
            ));
        }
    } else if !matches!(leg.from, AnyFix::Airport { .. })
        && !matches!(leg.to, AnyFix::Airport { .. })
        && is_from_zh
        && is_to_zh
    {
        messages.push(route_indexed_message(
            index,
            WarningMessageCode::RouteDirectSegment,
        ));
    }
    messages
}

fn route_matches_expected(_actual: &[ResolvedLeg], _expected: &[ResolvedLeg]) -> bool {
    // TODO: implement
    false
}

fn message(field: WarningMessageField, code: WarningMessageCode) -> WarningMessage {
    WarningMessage {
        message_code: code,
        parameter: None,
        field,
        field_index: None,
    }
}

fn route_indexed_message(index: usize, code: WarningMessageCode) -> WarningMessage {
    WarningMessage {
        message_code: code,
        parameter: None,
        field: WarningMessageField::Route,
        field_index: Some(index),
    }
}

fn level_restriction_type(cruising_level: i32) -> LevelRestrictionType {
    if let Some(metric_level) = standard_altitude_from_flight_level(cruising_level) {
        if metric_level % 200 == 0 {
            LevelRestrictionType::StandardEven
        } else {
            LevelRestrictionType::StandardOdd
        }
    } else if cruising_level % 1000 == 0 {
        if cruising_level % 2000 == 0 {
            LevelRestrictionType::FlightLevelEven
        } else {
            LevelRestrictionType::FlightLevelOdd
        }
    } else {
        LevelRestrictionType::Standard
    }
}

fn level_restriction_matches(actual: LevelRestrictionType, expected: LevelRestrictionType) -> bool {
    match expected {
        LevelRestrictionType::StandardEven => actual == LevelRestrictionType::StandardEven,
        LevelRestrictionType::StandardOdd => actual == LevelRestrictionType::StandardOdd,
        LevelRestrictionType::Standard => matches!(
            actual,
            LevelRestrictionType::Standard
                | LevelRestrictionType::StandardEven
                | LevelRestrictionType::StandardOdd
        ),
        LevelRestrictionType::FlightLevelEven => actual == LevelRestrictionType::FlightLevelEven,
        LevelRestrictionType::FlightLevelOdd => actual == LevelRestrictionType::FlightLevelOdd,
    }
}

fn level_restriction_param(restriction: LevelRestrictionType) -> String {
    match restriction {
        LevelRestrictionType::StandardEven => "standard_even",
        LevelRestrictionType::StandardOdd => "standard_odd",
        LevelRestrictionType::Standard => "standard",
        LevelRestrictionType::FlightLevelEven => "flight_level_even",
        LevelRestrictionType::FlightLevelOdd => "flight_level_odd",
    }
    .to_owned()
}

fn standard_altitude_from_flight_level(flight_level: i32) -> Option<i32> {
    STANDARD_ALTITUDE_TO_FLIGHT_LEVEL
        .iter()
        .find_map(|(standard, flight)| (*flight == flight_level).then_some(*standard))
}

const STANDARD_ALTITUDE_TO_FLIGHT_LEVEL: &[(i32, i32)] = &[
    (300, 1000),
    (900, 3000),
    (1500, 4900),
    (2100, 6900),
    (2700, 8900),
    (3300, 10800),
    (3900, 12800),
    (4500, 14800),
    (5100, 16700),
    (5700, 18700),
    (6300, 20700),
    (6900, 22600),
    (7500, 24600),
    (8100, 26600),
    (8900, 29100),
    (9500, 31100),
    (10100, 33100),
    (10700, 35100),
    (11300, 37100),
    (11900, 39100),
    (12500, 41100),
    (13700, 44900),
    (14900, 48900),
    (600, 2000),
    (1200, 3900),
    (1800, 5900),
    (2400, 7900),
    (3000, 9800),
    (3600, 11800),
    (4200, 13800),
    (4800, 15700),
    (5400, 17700),
    (6000, 19700),
    (6600, 21700),
    (7200, 23600),
    (7800, 25600),
    (8400, 27600),
    (9200, 30100),
    (9800, 32100),
    (10400, 34100),
    (11000, 36100),
    (11600, 38100),
    (12200, 40100),
    (13100, 43000),
    (14300, 46900),
];
