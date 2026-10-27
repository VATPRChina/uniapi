use crate::modules::flight::flight_plan::ParseRouteError;
use crate::modules::flight::flight_plan::v1::parser::ParserError;
use crate::modules::flight::models::ValidatorExecution;
use crate::modules::navdata::service::InvalidNavdataError;
use crate::modules::navdata::service::NavdataService;

use self::checks::*;

pub mod checks;
mod segments;

pub use segments::validate_route;

#[derive(Debug, thiserror::Error)]
pub enum ValidatorError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("parser error: {0}")]
    Parser(#[from] ParseRouteError),
    #[error("parser error: {0}")]
    #[deprecated]
    ParserV1(#[from] ParserError),
    #[error("navdata error: {0}")]
    Navdata(InvalidNavdataError),
}

/// Run flight-level checks once, in a stable order.
pub fn validate_flight(
    context: &ValidationContext<'_>,
    navdata: &NavdataService,
) -> Vec<ValidatorExecution> {
    validate_flight_checks(context, navdata, true)
}

pub(super) fn validate_flight_checks(
    context: &ValidationContext<'_>,
    navdata: &NavdataService,
    include_directional_altitude: bool,
) -> Vec<ValidatorExecution> {
    let mut results = vec![
        FlightRuleWeatherValidator::execute(context, navdata, FlightRuleWeatherValidator::FIELD),
        AircraftTypeValidator::execute(context, navdata, AircraftTypeValidator::FIELD),
        WakeCategoryValidator::execute(context, navdata, WakeCategoryValidator::FIELD),
        DepartureAircraftTypeValidator::execute(
            context,
            navdata,
            DepartureAircraftTypeValidator::FIELD,
        ),
        DepartureAirportValidator::execute(context, navdata, DepartureAirportValidator::FIELD),
        ArrivalAircraftTypeValidator::execute(
            context,
            navdata,
            ArrivalAircraftTypeValidator::FIELD,
        ),
        ArrivalAirportValidator::execute(context, navdata, ArrivalAirportValidator::FIELD),
    ];
    if include_directional_altitude {
        results.push(DirectionalCruiseAltitudeValidator::execute(
            context,
            navdata,
            DirectionalCruiseAltitudeValidator::FIELD,
        ));
    }
    results.extend([
        EquipmentRvsmValidator::execute(context, navdata, EquipmentRvsmValidator::FIELD),
        EquipmentRnav1Validator::execute(context, navdata, EquipmentRnav1Validator::FIELD),
        EquipmentRnpArValidator::execute(context, navdata, EquipmentRnpArValidator::FIELD),
    ]);
    results
}
