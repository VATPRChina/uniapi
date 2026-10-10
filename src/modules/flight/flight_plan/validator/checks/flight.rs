use std::{collections::HashMap, sync::LazyLock};

use super::{ValidationContext, Validator, WarningMessageField};
use crate::modules::flight::flight_plan::validator::{WarningMessage, WarningMessageCode};
use crate::modules::flight::models::{ValidatorResult, ValidatorStatus};
use crate::modules::navdata::service::NavdataService;

// Generated from data/AircraftTypes.json by scripts/generate_aircraft_types.py.
static AIRCRAFT_TYPES: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/aircraft-types.json"
    )))
    .expect("embedded aircraft-type lookup must be valid")
});

/// Callsign must identify a valid airline flight or aircraft registration.
pub struct CallsignValidator;

impl Validator for CallsignValidator {
    const IDENT: &'static str = "callsign";
    const FIELD: WarningMessageField = WarningMessageField::Callsign;
}

/// Flight rules must be permitted by the current weather conditions.
pub struct FlightRuleWeatherValidator;

impl Validator for FlightRuleWeatherValidator {
    const IDENT: &'static str = "flight-rule-weather";
    const FIELD: WarningMessageField = WarningMessageField::FlightRules;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        ValidatorResult {
            validator_ident: Self::IDENT.to_owned(),
            field: Self::FIELD,
            status: if context.flight.flight_rules.trim().eq_ignore_ascii_case("I") {
                ValidatorStatus::Suppressed
            } else {
                ValidatorStatus::Unavailable
            },
            warnings: Vec::new(),
        }
    }
}

/// Aircraft type must be a valid aircraft type designator.
pub struct AircraftTypeValidator;

impl Validator for AircraftTypeValidator {
    const IDENT: &'static str = "aircraft-type";
    const FIELD: WarningMessageField = WarningMessageField::AircraftType;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        let aircraft = context.flight.aircraft.trim().to_ascii_uppercase();
        let status = if aircraft.is_empty() {
            ValidatorStatus::Unavailable
        } else if AIRCRAFT_TYPES.contains_key(&aircraft) {
            ValidatorStatus::Pass
        } else {
            ValidatorStatus::Rejected
        };
        ValidatorResult {
            validator_ident: Self::IDENT.to_owned(),
            field: Self::FIELD,
            status: status.clone(),
            warnings: if status == ValidatorStatus::Rejected {
                vec![WarningMessage::with_parameter(
                    Self::FIELD,
                    WarningMessageCode::InvalidAircraftType,
                    aircraft,
                )]
            } else {
                Vec::new()
            },
        }
    }
}

/// Declared wake category must match the aircraft type.
pub struct WakeCategoryValidator;

impl Validator for WakeCategoryValidator {
    const IDENT: &'static str = "wake-category";
    const FIELD: WarningMessageField = WarningMessageField::WakeCategory;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        let aircraft = context.flight.aircraft.trim().to_ascii_uppercase();
        let wake = context.flight.wake_category.trim().to_ascii_uppercase();
        let categories = AIRCRAFT_TYPES.get(&aircraft);
        let status = match categories {
            _ if wake.is_empty() => ValidatorStatus::Unavailable,
            None => ValidatorStatus::Unavailable,
            Some(categories) if categories.split('/').any(|category| category == wake) => {
                ValidatorStatus::Pass
            }
            Some(_) => ValidatorStatus::Rejected,
        };
        ValidatorResult {
            validator_ident: Self::IDENT.to_owned(),
            field: Self::FIELD,
            status: status.clone(),
            warnings: if status == ValidatorStatus::Rejected {
                vec![WarningMessage::with_parameter(
                    Self::FIELD,
                    WarningMessageCode::WakeCategoryMismatch,
                    categories.unwrap().clone(),
                )]
            } else {
                Vec::new()
            },
        }
    }
}
