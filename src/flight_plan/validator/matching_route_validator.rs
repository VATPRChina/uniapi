use itertools::Itertools;

use crate::adapter::flight::Flight;
use crate::flight_plan::validator::{
    Validator, WarningMessage, WarningMessageCode, WarningMessageField,
};
use crate::model::navdata::{LevelRestrictionType, PreferredRoute};

pub struct NoMatchingRouteValidator;

impl Validator<(Option<&PreferredRoute>, Vec<&PreferredRoute>)> for NoMatchingRouteValidator {
    fn validate(
        (matching_route, preferred_routes): (Option<&PreferredRoute>, Vec<&PreferredRoute>),
    ) -> impl IntoIterator<Item = WarningMessage> {
        (matching_route.is_none() && !preferred_routes.is_empty()).then(|| {
            let routes = preferred_routes
                .iter()
                .filter(|route| !route.is_public())
                .sorted_by_key(|route| &route.name)
                .collect::<Vec<_>>();
            WarningMessage {
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
            }
        })
    }
}

type MatchingRouteContext<'a, 'b> = (&'a Flight, Option<&'b PreferredRoute>);

pub struct RouteMatchValidator;

impl<'a, 'b> Validator<MatchingRouteContext<'a, 'b>> for RouteMatchValidator {
    fn validate(
        (flight, preferred_route): MatchingRouteContext<'a, 'b>,
    ) -> impl IntoIterator<Item = WarningMessage> {
        preferred_route.map(|route| WarningMessage {
            field: WarningMessageField::Route,
            field_index: None,
            message_code: WarningMessageCode::RouteMatchPreferred,
            parameter: Some(if route.is_public() {
                route.raw_route.clone()
            } else {
                flight.raw_route.clone()
            }),
        })
    }
}

pub struct CruisingLevelRestrictionValidator;

impl<'a, 'b> Validator<MatchingRouteContext<'a, 'b>> for CruisingLevelRestrictionValidator {
    fn validate(
        (flight, preferred_route): MatchingRouteContext<'a, 'b>,
    ) -> impl IntoIterator<Item = WarningMessage> {
        preferred_route
            .filter(|route| {
                let actual = level_restriction_type(flight.cruising_level as i32);
                !level_restriction_matches(actual, route.cruising_level_restriction)
            })
            .map(|route| WarningMessage {
                field: WarningMessageField::CruisingLevel,
                field_index: None,
                message_code: WarningMessageCode::CruisingLevelMismatch,
                parameter: Some(level_restriction_param(route.cruising_level_restriction)),
            })
    }
}

pub struct AllowedAltitudesValidator;

impl<'a, 'b> Validator<MatchingRouteContext<'a, 'b>> for AllowedAltitudesValidator {
    fn validate(
        (flight, preferred_route): MatchingRouteContext<'a, 'b>,
    ) -> impl IntoIterator<Item = WarningMessage> {
        preferred_route
            .filter(|route| {
                !route.allowed_altitudes.is_empty()
                    && !route
                        .allowed_altitudes
                        .contains(&(flight.cruising_level as i32))
            })
            .map(|route| WarningMessage {
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
            })
    }
}

pub struct MinimalAltitudeValidator;

impl<'a, 'b> Validator<MatchingRouteContext<'a, 'b>> for MinimalAltitudeValidator {
    fn validate(
        (flight, preferred_route): MatchingRouteContext<'a, 'b>,
    ) -> impl IntoIterator<Item = WarningMessage> {
        preferred_route
            .filter(|route| {
                route.minimal_altitude != 0
                    && (flight.cruising_level as i32) < route.minimal_altitude
            })
            .map(|route| WarningMessage {
                field: WarningMessageField::CruisingLevel,
                field_index: None,
                message_code: WarningMessageCode::CruisingLevelTooLow,
                parameter: Some(route.minimal_altitude.to_string()),
            })
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
