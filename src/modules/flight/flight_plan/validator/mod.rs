use itertools::Itertools;
use serde::Serialize;
use tracing::info;

use crate::modules::flight::flight_plan::validator::flight_validator::{
    EquipmentRnav1Validator, NavigationPerformanceRnav1Validator, RnpArValidator,
    RnpArWithoutRfValidator, RvsmValidator,
};
use crate::modules::flight::flight_plan::validator::leg_validator::LegValidator;
use crate::modules::flight::flight_plan::validator::matching_route_validator::{
    AllowedAltitudesValidator, CruisingLevelRestrictionValidator, MinimalAltitudeValidator,
    NoMatchingRouteValidator, RouteMatchValidator,
};
use crate::modules::flight::flight_plan::{ParseRouteError, parse_route};
use crate::modules::flight::models::Flight;
use crate::modules::navdata::models::{AnyFix, Fix, LegKind, PreferredRoute, ResolvedLeg};
use crate::modules::navdata::service::{InvalidNavdataError, NavdataService};

mod flight_validator;
mod leg_validator;
mod matching_route_validator;

#[derive(Debug, thiserror::Error)]
pub enum ValidatorError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("parser error: {0}")]
    Parser(#[from] ParseRouteError),
    #[error("navdata error: {0}")]
    Navdata(InvalidNavdataError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, utoipa::ToSchema)]
pub struct WarningMessage {
    pub message_code: WarningMessageCode,
    pub parameter: Option<String>,
    pub field: WarningMessageField,
    pub field_index: Option<usize>,
}

impl WarningMessage {
    pub fn new(field: WarningMessageField, code: WarningMessageCode) -> Self {
        Self {
            message_code: code,
            parameter: None,
            field,
            field_index: None,
        }
    }

    pub fn with_parameter(
        field: WarningMessageField,
        code: WarningMessageCode,
        parameter: impl Into<String>,
    ) -> Self {
        Self {
            parameter: Some(parameter.into()),
            ..Self::new(field, code)
        }
    }

    pub fn route_indexed(index: usize, code: WarningMessageCode) -> Self {
        Self {
            field_index: Some(index),
            ..Self::new(WarningMessageField::Route, code)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum WarningMessageField {
    Equipment,
    Transponder,
    NavigationPerformance,
    Route,
    CruisingLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, utoipa::ToSchema)]
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
    navdata: &NavdataService,
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

    let messages = messages.validate_over::<LegValidator, _>(
        legs.iter().enumerate().filter(|_| matching_route.is_none()),
    );

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
    navdata: &NavdataService,
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
        let parsed = parse_route(
            navdata,
            &format!(
                "{} {} {}",
                preferred_route.departure, preferred_route.raw_route, preferred_route.arrival
            ),
        )
        .await?;
        if route_matches_expected(legs, &parsed) {
            return Ok(Some(preferred_route));
        }
    }
    Ok(None)
}

/// Match the actual enroute span between its leading SID and trailing STAR.
/// Procedure boundaries must occur in expected; absent procedures anchor that
/// end of the comparison to the corresponding route edge.
fn route_matches_expected(actual: &[ResolvedLeg], expected: &[ResolvedLeg]) -> bool {
    if actual.is_empty() || expected.is_empty() {
        return false;
    }

    let enroute_start = actual
        .iter()
        .take_while(|leg| leg.kind == LegKind::Sid)
        .count();
    let enroute_end = actual.len()
        - actual
            .iter()
            .rev()
            .take_while(|leg| leg.kind == LegKind::Star)
            .count();
    let enroute = &actual[enroute_start..enroute_end];
    let sid_exit = enroute_start.checked_sub(1).map(|index| &actual[index].to);
    let star_enter = actual.get(enroute_end).map(|leg| &leg.from);
    info!(
        "actual enroute={:?}[{}:{}], sid_exit={:?}, star_enter={:?}",
        enroute, enroute_start, enroute_end, sid_exit, star_enter
    );

    let expected_enroute_start = sid_exit
        .map(|sid_exit| {
            expected
                .iter()
                .take_while(|leg| !fix_matches(&leg.from, sid_exit))
                .count()
        })
        .unwrap_or_default();
    let expected_enroute_end = expected.len()
        - star_enter
            .map(|star_enter| {
                expected
                    .iter()
                    .rev()
                    .take_while(|leg| !fix_matches(&leg.from, star_enter))
                    .count()
            })
            .unwrap_or_default();
    if expected_enroute_end <= expected_enroute_start {
        info!("unable to find start and end on preferred route");
        return false;
    }
    let expected_enroute = &expected[expected_enroute_start..expected_enroute_end];
    info!(
        "expected enroute={:?}[{}:{}]",
        expected_enroute, expected_enroute_start, expected_enroute_end
    );

    if enroute.len() != expected_enroute.len() {
        return false;
    }

    enroute
        .iter()
        .zip(expected_enroute.iter())
        .all(|(expected, actual)| leg_matches(actual, expected))
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
