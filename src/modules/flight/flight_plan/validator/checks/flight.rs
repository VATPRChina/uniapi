use super::{Validator, WarningMessageField};

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
}

/// Aircraft type must be a valid aircraft type designator.
pub struct AircraftTypeValidator;

impl Validator for AircraftTypeValidator {
    const IDENT: &'static str = "aircraft-type";
    const FIELD: WarningMessageField = WarningMessageField::AircraftType;
}

/// Declared wake category must match the aircraft type.
pub struct WakeCategoryValidator;

impl Validator for WakeCategoryValidator {
    const IDENT: &'static str = "wake-category";
    const FIELD: WarningMessageField = WarningMessageField::WakeCategory;
}
