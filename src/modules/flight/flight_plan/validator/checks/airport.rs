use super::{Validator, WarningMessageField};

/// Departure airport must permit the aircraft type.
pub struct DepartureAircraftTypeValidator;

impl Validator for DepartureAircraftTypeValidator {
    const IDENT: &'static str = "departure-aircraft-type";
    const FIELD: WarningMessageField = WarningMessageField::Departure;
}

/// Departure must be a valid airport.
pub struct DepartureAirportValidator;

impl Validator for DepartureAirportValidator {
    const IDENT: &'static str = "departure-airport";
    const FIELD: WarningMessageField = WarningMessageField::Departure;
}

/// Arrival airport must permit the aircraft type.
pub struct ArrivalAircraftTypeValidator;

impl Validator for ArrivalAircraftTypeValidator {
    const IDENT: &'static str = "arrival-aircraft-type";
    const FIELD: WarningMessageField = WarningMessageField::Arrival;
}

/// Arrival must be a valid airport.
pub struct ArrivalAirportValidator;

impl Validator for ArrivalAirportValidator {
    const IDENT: &'static str = "arrival-airport";
    const FIELD: WarningMessageField = WarningMessageField::Arrival;
}
