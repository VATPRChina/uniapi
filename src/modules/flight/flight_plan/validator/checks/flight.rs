use std::{collections::HashMap, sync::LazyLock};

use super::CheckResult;
use super::{ValidationContext, Validator};
use crate::modules::flight::models::{FlightRule, ValidatorStatus};
use crate::modules::flight::models::{ValidationField, ValidationProblem, ValidatorIdentifier};
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
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::Callsign;
    const FIELD: ValidationField = ValidationField::Callsign;
}

/// Flight rules must be permitted by the current weather conditions.
pub struct FlightRuleWeatherValidator;

impl Validator for FlightRuleWeatherValidator {
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::FlightRuleWeather;
    const FIELD: ValidationField = ValidationField::FlightRules;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        CheckResult {
            id: Self::IDENT,
            field: Self::FIELD,
            status: if matches!(context.flight.flight_plan.flight_rules, FlightRule::Ifr) {
                ValidatorStatus::Suppressed
            } else {
                ValidatorStatus::Unavailable
            },
            problems: Vec::new(),
        }
    }
}

/// Aircraft type must be a valid aircraft type designator.
pub struct AircraftTypeValidator;

impl Validator for AircraftTypeValidator {
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::AircraftType;
    const FIELD: ValidationField = ValidationField::AircraftType;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        let aircraft = context
            .flight
            .flight_plan
            .aircraft
            .trim()
            .to_ascii_uppercase();
        let status = if aircraft.is_empty() {
            ValidatorStatus::Unavailable
        } else if AIRCRAFT_TYPES.contains_key(&aircraft) {
            ValidatorStatus::Pass
        } else {
            ValidatorStatus::Rejected
        };
        CheckResult {
            id: Self::IDENT,
            field: Self::FIELD,
            status,
            problems: if status == ValidatorStatus::Rejected {
                vec![ValidationProblem::InvalidAircraftType]
            } else {
                Vec::new()
            },
        }
    }
}

/// Declared wake category must match the aircraft type.
pub struct WakeCategoryValidator;

impl Validator for WakeCategoryValidator {
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::WakeCategory;
    const FIELD: ValidationField = ValidationField::WakeCategory;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        let aircraft = context
            .flight
            .flight_plan
            .aircraft
            .trim()
            .to_ascii_uppercase();
        let wake = serde_plain::to_string(&context.flight.flight_plan.wake_category)
            .expect("wake category is a string");
        let categories = AIRCRAFT_TYPES.get(&aircraft);
        let status = match categories {
            None => ValidatorStatus::Unavailable,
            Some(categories) if categories.split('/').any(|category| category == wake) => {
                ValidatorStatus::Pass
            }
            Some(_) => ValidatorStatus::Rejected,
        };
        CheckResult {
            id: Self::IDENT,
            field: Self::FIELD,
            status,
            problems: if status == ValidatorStatus::Rejected {
                vec![ValidationProblem::WakeCategoryMismatch {
                    expected: categories.unwrap().clone(),
                }]
            } else {
                Vec::new()
            },
        }
    }
}
