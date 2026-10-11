use super::{ValidationContext, Validator, WarningMessageField};
use crate::modules::flight::flight_plan::altitude::{
    level_restriction_matches, level_restriction_param, level_restriction_type,
    standard_altitude_from_flight_level,
};
use crate::modules::flight::flight_plan::validator::{WarningMessage, WarningMessageCode};
use crate::modules::flight::models::{ValidatorResult, ValidatorStatus};
use crate::modules::navdata::models::{Fix, LevelRestrictionType, ResolvedLeg};
use crate::modules::navdata::service::NavdataService;

/// Cruising altitude must meet metric or imperial preferred-route restrictions,
/// falling back to China metric directional levels when no route matches.
pub struct CruiseAltitudeValidator;

impl Validator for CruiseAltitudeValidator {
    const IDENT: &'static str = "cruise-altitude";
    const FIELD: WarningMessageField = WarningMessageField::CruisingLevel;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        let Some(level) = i32::try_from(context.flight.cruising_level)
            .ok()
            .filter(|level| *level > 0)
        else {
            return result(ValidatorStatus::Unavailable, Vec::new());
        };
        let restriction = if let Some(route) = context.preferred_route {
            route.cruising_level_restriction
        } else {
            let Some(westbound) = context.route.legs.first().and_then(is_westbound) else {
                return result(ValidatorStatus::Unavailable, Vec::new());
            };
            if westbound {
                LevelRestrictionType::StandardEven
            } else {
                LevelRestrictionType::StandardOdd
            }
        };
        // Imperial requirements use feet directly: some imperial levels also occur
        // in the China metric conversion table, so classify against the requirement.
        let matches_restriction = match restriction {
            LevelRestrictionType::FlightLevelEven => level % 2000 == 0,
            LevelRestrictionType::FlightLevelOdd => level % 2000 == 1000,
            _ => {
                standard_altitude_from_flight_level(level).is_some()
                    && level_restriction_matches(level_restriction_type(level), restriction)
            }
        };
        let mut warnings = Vec::new();
        if !matches_restriction {
            warnings.push(WarningMessage::with_parameter(
                Self::FIELD,
                WarningMessageCode::CruisingLevelMismatch,
                level_restriction_param(restriction),
            ));
        }
        if let Some(route) = context.preferred_route {
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
        }
        result(
            if warnings.is_empty() {
                ValidatorStatus::Pass
            } else {
                ValidatorStatus::Rejected
            },
            warnings,
        )
    }
}

fn result(status: ValidatorStatus, warnings: Vec<WarningMessage>) -> ValidatorResult {
    ValidatorResult {
        validator_ident: CruiseAltitudeValidator::IDENT.to_owned(),
        field: CruiseAltitudeValidator::FIELD,
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
