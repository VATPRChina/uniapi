use crate::modules::flight::flight_plan::altitude::{
    level_restriction_matches, level_restriction_type,
};
use itertools::Itertools;

use crate::modules::flight::flight_plan::v1::validator::{
    Validator, WarningMessage, WarningMessageCode, WarningMessageField,
};
use crate::modules::flight::models::Flight;
use crate::modules::navdata::models::{CruiseAltitudeType, PreferredRoute};

pub fn level_restriction_param(restriction: CruiseAltitudeType) -> String {
    match restriction {
        CruiseAltitudeType::StandardEven => "standard_even",
        CruiseAltitudeType::StandardOdd => "standard_odd",
        CruiseAltitudeType::Standard => "standard",
        CruiseAltitudeType::FlightLevelEven => "flight_level_even",
        CruiseAltitudeType::FlightLevelOdd => "flight_level_odd",
    }
    .to_owned()
}

pub struct NoMatchingRouteValidator;

impl Validator<(Option<&PreferredRoute>, &[PreferredRoute])> for NoMatchingRouteValidator {
    fn validate(
        (matching_route, preferred_routes): (Option<&PreferredRoute>, &[PreferredRoute]),
    ) -> impl IntoIterator<Item = WarningMessage> {
        (matching_route.is_none() && !preferred_routes.is_empty()).then(|| {
            let routes = preferred_routes
                .iter()
                .filter(|route| route.is_public())
                .sorted_by_key(|route| &route.name)
                .collect::<Vec<_>>();
            WarningMessage::with_parameter(
                WarningMessageField::Route,
                WarningMessageCode::NotPreferredRoute,
                routes
                    .into_iter()
                    .map(|route| route.raw_route.clone())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        })
    }
}

type MatchingRouteContext<'a, 'b> = (&'a Flight, Option<&'b PreferredRoute>);

pub struct RouteMatchValidator;

impl<'a, 'b> Validator<MatchingRouteContext<'a, 'b>> for RouteMatchValidator {
    fn validate(
        (flight, preferred_route): MatchingRouteContext<'a, 'b>,
    ) -> impl IntoIterator<Item = WarningMessage> {
        preferred_route.map(|route| {
            WarningMessage::with_parameter(
                WarningMessageField::Route,
                WarningMessageCode::RouteMatchPreferred,
                if route.is_public() {
                    route.raw_route.clone()
                } else {
                    flight.flight_plan.raw_route.clone()
                },
            )
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
                let actual = level_restriction_type(flight.flight_plan.cruising_level as i32);
                !level_restriction_matches(actual, route.cruising_level_restriction)
            })
            .map(|route| {
                WarningMessage::with_parameter(
                    WarningMessageField::CruisingLevel,
                    WarningMessageCode::CruisingLevelMismatch,
                    level_restriction_param(route.cruising_level_restriction),
                )
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
                        .contains(&(flight.flight_plan.cruising_level as i32))
            })
            .map(|route| {
                WarningMessage::with_parameter(
                    WarningMessageField::CruisingLevel,
                    WarningMessageCode::CruisingLevelNotAllowed,
                    route
                        .allowed_altitudes
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                )
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
                    && (flight.flight_plan.cruising_level as i32) < route.minimal_altitude
            })
            .map(|route| {
                WarningMessage::with_parameter(
                    WarningMessageField::CruisingLevel,
                    WarningMessageCode::CruisingLevelTooLow,
                    route.minimal_altitude.to_string(),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::modules::navdata::models::CruiseAltitudeType;

    use super::*;

    #[test]
    fn classifies_metric_levels_below_rvsm() {
        for flight_level in [
            1000, 3000, 4900, 6900, 8900, 10800, 12800, 14800, 16700, 18700, 20700, 22600, 24600,
            26600,
        ] {
            assert_eq!(
                level_restriction_type(flight_level),
                CruiseAltitudeType::StandardOdd
            );
        }

        for flight_level in [
            2000, 3900, 5900, 7900, 9800, 11800, 13800, 15700, 17700, 19700, 21700, 23600, 25600,
            27600,
        ] {
            assert_eq!(
                level_restriction_type(flight_level),
                CruiseAltitudeType::StandardEven
            );
        }
    }

    #[test]
    fn classifies_metric_levels_in_rvsm_airspace() {
        for flight_level in [29100, 31100, 33100, 35100, 37100, 39100, 41100] {
            assert_eq!(
                level_restriction_type(flight_level),
                CruiseAltitudeType::StandardOdd
            );
        }

        for flight_level in [30100, 32100, 34100, 36100, 38100, 40100] {
            assert_eq!(
                level_restriction_type(flight_level),
                CruiseAltitudeType::StandardEven
            );
        }
    }

    #[test]
    fn classifies_metric_levels_above_rvsm() {
        for flight_level in [44900, 48900] {
            assert_eq!(
                level_restriction_type(flight_level),
                CruiseAltitudeType::StandardOdd
            );
        }

        for flight_level in [43000, 46900] {
            assert_eq!(
                level_restriction_type(flight_level),
                CruiseAltitudeType::StandardEven
            );
        }
    }

    #[test]
    fn classifies_unmapped_flight_levels_and_non_standard_altitudes() {
        assert_eq!(
            level_restriction_type(30000),
            CruiseAltitudeType::FlightLevelEven
        );
        assert_eq!(
            level_restriction_type(31000),
            CruiseAltitudeType::FlightLevelOdd
        );
        assert_eq!(level_restriction_type(30500), CruiseAltitudeType::Standard);
    }
}
