use itertools::Itertools;

use crate::modules::flight::flight_plan::v1::parser;
use crate::modules::flight::flight_plan::v1::validator::flight_validator::{
    EquipmentRnav1Validator, NavigationPerformanceRnav1Validator, RnpArValidator,
    RnpArWithoutRfValidator, RvsmValidator,
};
use crate::modules::flight::flight_plan::v1::validator::leg_validator::LegValidator;
use crate::modules::flight::flight_plan::v1::validator::matching_route_validator::{
    AllowedAltitudesValidator, CruisingLevelRestrictionValidator, MinimalAltitudeValidator,
    NoMatchingRouteValidator, RouteMatchValidator,
};
use crate::modules::flight::flight_plan::validator::{
    ValidatorError, WarningMessage, WarningMessageCode, WarningMessageField,
};
use crate::modules::flight::models::Flight;
use crate::modules::navdata::models::{AnyFix, Fix, PreferredRoute, ResolvedLeg};
use crate::modules::navdata::service::NavdataService;

mod flight_validator;
mod leg_validator;
mod matching_route_validator;

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
