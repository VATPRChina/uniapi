use super::{ValidationContext, Validator, WarningMessageField};
use crate::modules::flight::flight_plan::altitude::{
    level_restriction_matches, level_restriction_param, level_restriction_type,
    standard_altitude_from_flight_level,
};
use crate::modules::flight::flight_plan::validator::{WarningMessage, WarningMessageCode};
use crate::modules::flight::models::{ValidatorResult, ValidatorStatus};
use crate::modules::navdata::models::{Fix, LevelRestrictionType, ResolvedLeg};
use crate::modules::navdata::service::NavdataService;

/// Validate metric or imperial altitude restrictions on a matched preferred route.
pub struct PreferredRouteCruiseAltitudeValidator;

impl Validator for PreferredRouteCruiseAltitudeValidator {
    const IDENT: &'static str = "cruise-altitude-preferred-route";
    const FIELD: WarningMessageField = WarningMessageField::CruisingLevel;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        let Some(route) = context.preferred_route else {
            return result::<Self>(ValidatorStatus::Suppressed, Vec::new());
        };
        let Some(level) = cruising_level(context) else {
            return result::<Self>(ValidatorStatus::Unavailable, Vec::new());
        };
        let mut warnings = restriction_warnings(level, route.cruising_level_restriction);
        if !route.allowed_altitudes.is_empty() && !route.allowed_altitudes.contains(&level) {
            warnings.push(WarningMessage::with_parameter(
                Self::FIELD,
                WarningMessageCode::CruisingLevelNotAllowed,
                route
                    .allowed_altitudes
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        if route.minimal_altitude > 0 && level < route.minimal_altitude {
            warnings.push(WarningMessage::with_parameter(
                Self::FIELD,
                WarningMessageCode::CruisingLevelTooLow,
                route.minimal_altitude.to_string(),
            ));
        }
        checked_result::<Self>(warnings)
    }
}

/// For unmatched routes departing China, use the first leg's metric altitude direction.
pub struct DirectionalCruiseAltitudeValidator;

impl Validator for DirectionalCruiseAltitudeValidator {
    const IDENT: &'static str = "cruise-altitude-directional";
    const FIELD: WarningMessageField = WarningMessageField::CruisingLevel;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        if context.preferred_route.is_some() {
            return result::<Self>(ValidatorStatus::Suppressed, Vec::new());
        }
        let departure = context.flight.departure.trim().to_ascii_uppercase();
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
            LevelRestrictionType::StandardEven
        } else {
            LevelRestrictionType::StandardOdd
        };
        checked_result::<Self>(restriction_warnings(level, restriction))
    }
}

fn cruising_level(context: &ValidationContext<'_>) -> Option<i32> {
    i32::try_from(context.flight.cruising_level)
        .ok()
        .filter(|level| *level > 0)
}

fn restriction_warnings(level: i32, restriction: LevelRestrictionType) -> Vec<WarningMessage> {
    // Imperial requirements use feet directly, including levels also in the metric table.
    let matches_restriction = match restriction {
        LevelRestrictionType::FlightLevelEven => level % 2000 == 0,
        LevelRestrictionType::FlightLevelOdd => level % 2000 == 1000,
        _ => {
            standard_altitude_from_flight_level(level).is_some()
                && level_restriction_matches(level_restriction_type(level), restriction)
        }
    };
    if matches_restriction {
        Vec::new()
    } else {
        vec![WarningMessage::with_parameter(
            WarningMessageField::CruisingLevel,
            WarningMessageCode::CruisingLevelMismatch,
            level_restriction_param(restriction),
        )]
    }
}

fn checked_result<V: Validator>(warnings: Vec<WarningMessage>) -> ValidatorResult {
    result::<V>(
        if warnings.is_empty() {
            ValidatorStatus::Pass
        } else {
            ValidatorStatus::Rejected
        },
        warnings,
    )
}

fn result<V: Validator>(status: ValidatorStatus, warnings: Vec<WarningMessage>) -> ValidatorResult {
    ValidatorResult {
        validator_ident: V::IDENT.to_owned(),
        field: V::FIELD,
        status,
        warnings,
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
