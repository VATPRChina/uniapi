use super::{ValidationContext, Validator, WarningMessageField};
use crate::modules::flight::flight_plan::validator::{WarningMessage, WarningMessageCode};
use crate::modules::flight::models::{ValidatorResult, ValidatorStatus};
use crate::modules::navdata::models::AnyFix;
use crate::modules::navdata::service::NavdataService;

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

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        validate_airport::<Self>(
            &context.flight.departure,
            context.route.legs.first().map(|leg| &leg.from),
        )
    }
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

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        validate_airport::<Self>(
            &context.flight.arrival,
            context.route.legs.last().map(|leg| &leg.to),
        )
    }
}

/// The route parser has already resolved these endpoints against navdata.
fn validate_airport<V: Validator>(identifier: &str, endpoint: Option<&AnyFix>) -> ValidatorResult {
    let identifier = identifier.trim();
    let valid_identifier = identifier.len() == 4
        && identifier
            .bytes()
            .all(|character| character.is_ascii_alphabetic());
    let status = if identifier.is_empty() || identifier == "ZZZZ" {
        ValidatorStatus::Unavailable
    } else if !valid_identifier {
        ValidatorStatus::Rejected
    } else {
        match endpoint {
            None => ValidatorStatus::Unavailable,
            Some(AnyFix::Airport(airport))
                if airport.identifier.as_str().eq_ignore_ascii_case(identifier) =>
            {
                ValidatorStatus::Pass
            }
            Some(_) => ValidatorStatus::Rejected,
        }
    };
    let warnings = if status == ValidatorStatus::Rejected {
        vec![WarningMessage::with_parameter(
            V::FIELD,
            WarningMessageCode::InvalidAirport,
            identifier,
        )]
    } else {
        Vec::new()
    };
    ValidatorResult {
        validator_ident: V::IDENT.to_owned(),
        field: V::FIELD,
        status,
        warnings,
    }
}
