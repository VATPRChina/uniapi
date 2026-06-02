use itertools::Itertools;
use serde::Serialize;

use crate::adapter::flight::Flight;
use crate::adapter::navdata::{InvalidNavdataError, NavdataAdapter};
use crate::flight_plan::parser::{self, ParserError};
use crate::flight_plan::validator::flight_validator::{
    EquipmentRnav1Validator, NavigationPerformanceRnav1Validator, RnpArValidator,
    RnpArWithoutRfValidator, RvsmValidator,
};
use crate::flight_plan::validator::leg_validator::LegValidator;
use crate::flight_plan::validator::matching_route_validator::{
    AllowedAltitudesValidator, CruisingLevelRestrictionValidator, MinimalAltitudeValidator,
    NoMatchingRouteValidator, RouteMatchValidator,
};
use crate::model::navdata::{AnyFix, Fix, PreferredRoute, ResolvedLeg};

mod flight_validator;
mod leg_validator;
mod matching_route_validator;

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
pub enum WarningMessageField {
    Equipment,
    Transponder,
    NavigationPerformance,
    Route,
    CruisingLevel,
}

#[derive(Debug, Clone, Copy, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
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
    CruisingLevelTooLow,
    RouteMatchPreferred,
}

pub async fn validate_route(
    navdata: &NavdataAdapter,
    flight: &Flight,
    legs: &[ResolvedLeg],
) -> Result<Vec<WarningMessage>, ValidatorError> {
    let preferred_routes = navdata
        .list_preferred_routes(&flight.departure, &flight.arrival)
        .await
        .map_err(ValidatorError::Navdata)?;
    let matching_route = find_matching_route(navdata, legs, &preferred_routes).await?;

    let messages = MessageContainer::new()
        .validate::<RvsmValidator, _>(flight)
        .validate::<EquipmentRnav1Validator, _>(flight)
        .validate::<NavigationPerformanceRnav1Validator, _>(flight)
        .validate::<RnpArWithoutRfValidator, _>(flight)
        .validate::<RnpArValidator, _>(flight);

    let messages =
        messages.validate::<NoMatchingRouteValidator, _>((matching_route, preferred_routes));

    let context_matching_route = (flight, matching_route);
    let messages = messages
        .validate::<RouteMatchValidator, _>(context_matching_route)
        .validate::<CruisingLevelRestrictionValidator, _>(context_matching_route)
        .validate::<AllowedAltitudesValidator, _>(context_matching_route)
        .validate::<MinimalAltitudeValidator, _>(context_matching_route);

    let messages = messages.validate_over::<LegValidator, _>(legs.iter().enumerate());

    Ok(messages.build().into_iter().collect())
}

struct MessageContainer<T: IntoIterator<Item = WarningMessage>>(T);

impl MessageContainer<std::iter::Empty<WarningMessage>> {
    pub fn new() -> Self {
        MessageContainer(std::iter::empty())
    }
}

trait Validator<C> {
    fn validate(context: C) -> impl IntoIterator<Item = WarningMessage>;
}

impl<T: IntoIterator<Item = WarningMessage>> MessageContainer<T> {
    pub fn join(
        self,
        other: impl IntoIterator<Item = WarningMessage>,
    ) -> MessageContainer<impl IntoIterator<Item = WarningMessage>> {
        MessageContainer(self.0.into_iter().chain(other))
    }

    pub fn validate<V: Validator<C>, C>(
        self,
        context: C,
    ) -> MessageContainer<impl IntoIterator<Item = WarningMessage>> {
        self.join(V::validate(context))
    }

    pub fn validate_over<V: Validator<C>, C>(
        self,
        contexts: impl IntoIterator<Item = C>,
    ) -> MessageContainer<impl IntoIterator<Item = WarningMessage>> {
        self.join(
            contexts
                .into_iter()
                .flat_map(|context| V::validate(context)),
        )
    }

    pub fn build(self) -> T {
        self.0
    }
}

async fn find_matching_route<'a>(
    navdata: &NavdataAdapter,
    legs: &[ResolvedLeg],
    preferred_routes: &[&'a PreferredRoute],
) -> Result<Option<&'a PreferredRoute>, ValidatorError> {
    for &preferred_route in preferred_routes
        .iter()
        .sorted_by_key(|route| if route.is_public { 0 } else { 1 })
    {
        tracing::info!(
            "checking preferred route {}: {}",
            preferred_route.name,
            preferred_route.raw_route
        );
        let parsed = parser::parse_route(navdata, &preferred_route.raw_route).await?;
        if route_matches_expected(legs, &parsed) {
            return Ok(Some(preferred_route));
        }
    }
    Ok(None)
}

fn route_matches_expected(actual: &[ResolvedLeg], expected: &[ResolvedLeg]) -> bool {
    !expected.is_empty()
        && expected.len() <= actual.len()
        && actual.windows(expected.len()).any(|legs| {
            legs.iter()
                .zip(expected)
                .all(|(actual, expected)| leg_matches(actual, expected))
        })
}

fn leg_matches(actual: &ResolvedLeg, expected: &ResolvedLeg) -> bool {
    actual.identifier == expected.identifier
        && fix_matches(&actual.from, &expected.from)
        && fix_matches(&actual.to, &expected.to)
}

fn fix_matches(actual: &AnyFix, expected: &AnyFix) -> bool {
    match (actual.identifier(), expected.identifier()) {
        (Some(actual), Some(expected)) => actual.eq_ignore_ascii_case(expected),
        (None, None) => {
            approx::relative_eq!(actual.latitude(), expected.latitude(), max_relative = 1e-6)
                && approx::relative_eq!(
                    actual.longitude(),
                    expected.longitude(),
                    max_relative = 1e-6
                )
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use arrayvec::ArrayString;

    use super::*;
    use crate::model::navdata::{Airport, DirectionRestriction, Waypoint, WaypointKind};

    #[test]
    fn route_matches_exact_leg_sequence() {
        let actual = vec![airway_leg("A", "B", "W1"), airway_leg("B", "C", "W1")];
        let expected = vec![airway_leg("A", "B", "W1"), airway_leg("B", "C", "W1")];

        assert!(route_matches_expected(&actual, &expected));
    }

    #[test]
    fn route_matches_preferred_subsequence_inside_airport_boundaries() {
        let actual = vec![
            direct_leg(airport("ZBAA"), fix("A")),
            airway_leg("A", "B", "W1"),
            airway_leg("B", "C", "W1"),
            direct_leg(fix("C"), airport("CYUL")),
        ];
        let expected = vec![airway_leg("A", "B", "W1"), airway_leg("B", "C", "W1")];

        assert!(route_matches_expected(&actual, &expected));
    }

    #[test]
    fn route_does_not_match_non_contiguous_expected_legs() {
        let actual = vec![
            airway_leg("A", "B", "W1"),
            direct_leg(fix("B"), fix("X")),
            airway_leg("X", "C", "W1"),
        ];
        let expected = vec![airway_leg("A", "B", "W1"), airway_leg("B", "C", "W1")];

        assert!(!route_matches_expected(&actual, &expected));
    }

    #[test]
    fn route_does_not_match_different_airway_identifier() {
        let actual = vec![airway_leg("A", "B", "W2")];
        let expected = vec![airway_leg("A", "B", "W1")];

        assert!(!route_matches_expected(&actual, &expected));
    }

    #[test]
    fn route_does_not_match_empty_expected_route() {
        assert!(!route_matches_expected(
            &[direct_leg(fix("A"), fix("B"))],
            &[]
        ));
    }

    fn airway_leg(from: &str, to: &str, airway: &str) -> ResolvedLeg {
        ResolvedLeg {
            from: fix(from),
            to: fix(to),
            identifier: Some(airway.to_owned()),
            direction_restriction: DirectionRestriction::None,
        }
    }

    fn direct_leg(from: AnyFix, to: AnyFix) -> ResolvedLeg {
        ResolvedLeg {
            from,
            to,
            identifier: None,
            direction_restriction: DirectionRestriction::None,
        }
    }

    fn airport(identifier: &str) -> AnyFix {
        AnyFix::Airport(Airport {
            identifier: ArrayString::from(identifier).unwrap(),
            latitude: 0.0,
            longitude: 0.0,
        })
    }

    fn fix(identifier: &str) -> AnyFix {
        AnyFix::Waypoint(Waypoint {
            icao_code: ArrayString::from("ZH").unwrap(),
            identifier: ArrayString::from(identifier).unwrap(),
            latitude: 0.0,
            longitude: 0.0,
            kind: WaypointKind::Enroute,
        })
    }
}
