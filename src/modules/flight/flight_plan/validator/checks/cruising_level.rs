use super::CheckResult;
use super::{ValidationContext, Validator};
use crate::modules::flight::flight_plan::altitude::{
    level_restriction_matches, level_restriction_type, standard_altitude_from_flight_level,
};
use crate::modules::flight::models::ValidatorStatus;
use crate::modules::flight::models::{ValidationField, ValidationProblem, ValidatorIdentifier};
use crate::modules::navdata::models::{CruiseAltitudeType, Fix, ResolvedLeg};
use crate::modules::navdata::service::NavdataService;

/// Validate metric or imperial altitude restrictions on a matched preferred route.
pub struct PreferredRouteCruiseAltitudeValidator;

impl Validator for PreferredRouteCruiseAltitudeValidator {
    const LEVEL: super::ValidationLevel = super::ValidationLevel::Segment;
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::PreferredRouteCruiseAltitude;
    const FIELD: ValidationField = ValidationField::CruisingLevel;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        let Some(route) = context.preferred_route else {
            return result::<Self>(ValidatorStatus::Suppressed, Vec::new());
        };
        let Some(level) = cruising_level(context) else {
            return result::<Self>(ValidatorStatus::Unavailable, Vec::new());
        };
        let mut problems = restriction_problems(level, route.cruising_level_restriction);
        if !route.allowed_altitudes.is_empty() && !route.allowed_altitudes.contains(&level) {
            problems.push(ValidationProblem::CruisingLevelNotAllowed {
                allowlist_ft: route
                    .allowed_altitudes
                    .iter()
                    .map(|level| i64::from(*level))
                    .collect(),
            });
        }
        if route.minimal_altitude > 0 && level < route.minimal_altitude {
            problems.push(ValidationProblem::CruisingLevelTooLow {
                minimum_ft: i64::from(route.minimal_altitude),
            });
        }
        checked_result::<Self>(problems)
    }
}

/// For unmatched routes departing China, use the first leg's metric altitude direction.
pub struct DirectionalCruiseAltitudeValidator;

impl Validator for DirectionalCruiseAltitudeValidator {
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::DirectionalCruiseAltitude;
    const FIELD: ValidationField = ValidationField::CruisingLevel;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        if context.preferred_route.is_some() {
            return result::<Self>(ValidatorStatus::Suppressed, Vec::new());
        }
        let departure = context
            .flight
            .flight_plan
            .departure
            .trim()
            .to_ascii_uppercase();
        if departure.len() != 4
            || !departure.bytes().all(|byte| byte.is_ascii_alphabetic())
            || departure == "ZZZZ"
        {
            return result::<Self>(ValidatorStatus::Unavailable, Vec::new());
        }
        if !departure.starts_with('Z') || departure.starts_with("ZM") || departure.starts_with("ZK")
        {
            return result::<Self>(ValidatorStatus::Suppressed, Vec::new());
        }
        let Some(level) = cruising_level(context) else {
            return result::<Self>(ValidatorStatus::Unavailable, Vec::new());
        };
        let Some(westbound) = context.route.legs.first().and_then(is_westbound) else {
            return result::<Self>(ValidatorStatus::Unavailable, Vec::new());
        };
        let restriction = if westbound {
            CruiseAltitudeType::StandardEven
        } else {
            CruiseAltitudeType::StandardOdd
        };
        checked_result::<Self>(restriction_problems(level, restriction))
    }
}

fn cruising_level(context: &ValidationContext<'_>) -> Option<i32> {
    i32::try_from(context.flight.flight_plan.cruising_level)
        .ok()
        .filter(|level| *level > 0)
}

fn restriction_problems(level: i32, restriction: CruiseAltitudeType) -> Vec<ValidationProblem> {
    // Imperial requirements use feet directly, including levels also in the metric table.
    let matches_restriction = match restriction {
        CruiseAltitudeType::FlightLevelEven => level % 2000 == 0,
        CruiseAltitudeType::FlightLevelOdd => level % 2000 == 1000,
        _ => {
            standard_altitude_from_flight_level(level).is_some()
                && level_restriction_matches(level_restriction_type(level), restriction)
        }
    };
    if matches_restriction {
        Vec::new()
    } else {
        vec![ValidationProblem::CruisingLevelMismatch {
            expected_type: restriction,
        }]
    }
}

fn checked_result<V: Validator>(problems: Vec<ValidationProblem>) -> CheckResult {
    result::<V>(
        if problems.is_empty() {
            ValidatorStatus::Pass
        } else {
            ValidatorStatus::Rejected
        },
        problems,
    )
}

fn result<V: Validator>(status: ValidatorStatus, problems: Vec<ValidationProblem>) -> CheckResult {
    CheckResult {
        id: V::IDENT,
        field: V::FIELD,
        status,
        problems,
    }
}

/// Initial great-circle bearing, including wraparound at the date line.
fn is_westbound(leg: &ResolvedLeg) -> Option<bool> {
    let (from_lat, from_lon) = leg.from.position()?;
    let (to_lat, to_lon) = leg.to.position()?;
    if !(-90.0..=90.0).contains(&from_lat)
        || !(-90.0..=90.0).contains(&to_lat)
        || !(-180.0..=180.0).contains(&from_lon)
        || !(-180.0..=180.0).contains(&to_lon)
    {
        return None;
    }
    let delta_lon = (to_lon - from_lon).to_radians();
    let from_lat = from_lat.to_radians();
    let to_lat = to_lat.to_radians();
    let y = delta_lon.sin() * to_lat.cos();
    let x = from_lat.cos() * to_lat.sin() - from_lat.sin() * to_lat.cos() * delta_lon.cos();
    if x.abs() < 1e-12 && y.abs() < 1e-12 {
        return None;
    }
    Some(y.atan2(x).to_degrees().rem_euclid(360.0) >= 180.0)
}
