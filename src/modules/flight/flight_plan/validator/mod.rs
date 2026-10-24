use serde::Serialize;

use crate::modules::flight::flight_plan::ParseRouteError;
use crate::modules::flight::flight_plan::v1::parser::ParserError;
use crate::modules::flight::models::{ValidationScope, ValidatorResult};
use crate::modules::navdata::service::InvalidNavdataError;
use crate::modules::navdata::service::NavdataService;

use self::checks::*;

pub mod checks;
mod segments;

pub(crate) use segments::validate_analyzed_route;
pub use segments::validate_route;

#[derive(Debug, thiserror::Error)]
pub enum ValidatorError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("parser error: {0}")]
    Parser(#[from] ParseRouteError),
    #[error("parser error: {0}")]
    ParserV1(#[from] ParserError),
    #[error("navdata error: {0}")]
    Navdata(InvalidNavdataError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, utoipa::ToSchema)]
pub struct WarningMessage {
    pub message_code: WarningMessageCode,
    pub parameter: Option<String>,
    pub field: WarningMessageField,
    pub field_index: Option<usize>,
}

impl WarningMessage {
    pub fn new(field: WarningMessageField, code: WarningMessageCode) -> Self {
        Self {
            message_code: code,
            parameter: None,
            field,
            field_index: None,
        }
    }

    pub fn with_parameter(
        field: WarningMessageField,
        code: WarningMessageCode,
        parameter: impl Into<String>,
    ) -> Self {
        Self {
            parameter: Some(parameter.into()),
            ..Self::new(field, code)
        }
    }

    pub fn route_indexed(index: usize, code: WarningMessageCode) -> Self {
        Self {
            field_index: Some(index),
            ..Self::new(WarningMessageField::Route, code)
        }
    }

    pub fn route_indexed_with_param(
        index: usize,
        code: WarningMessageCode,
        param: impl Into<String>,
    ) -> Self {
        Self {
            field_index: Some(index),
            ..Self::with_parameter(WarningMessageField::Route, code, param)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum WarningMessageField {
    Callsign,
    FlightRules,
    AircraftType,
    WakeCategory,
    Equipment,
    Transponder,
    Departure,
    Airspeed,
    CruisingLevel,
    Route,
    Arrival,
    NavigationPerformance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum WarningMessageCode {
    InvalidAirport,
    InvalidAircraftType,
    WakeCategoryMismatch,
    AircraftWingspanTooLarge,
    AircraftWingspanManualCheck,
    NoRvsm,
    NoRnav1,
    RnpAr,
    RnpArWithoutRf,
    NoTransponder,
    RouteDirectSegment,
    RouteUnknownLegOrFix,
    RouteLegDirection,
    AirwayRequireApproval,
    NotPreferredRoute,
    CruisingLevelMismatch,
    CruisingLevelNotAllowed,
    CruisingLevelTooLow,
    RouteMatchPreferred,
}

/// Run flight-level checks once, in a stable order.
pub fn validate_flight(
    context: &ValidationContext<'_>,
    navdata: &NavdataService,
) -> Vec<ValidatorResult> {
    vec![
        CallsignValidator::execute(context, navdata, ValidationScope::Flight),
        FlightRuleWeatherValidator::execute(context, navdata, ValidationScope::Flight),
        AircraftTypeValidator::execute(context, navdata, ValidationScope::Flight),
        WakeCategoryValidator::execute(context, navdata, ValidationScope::Flight),
        DepartureAircraftTypeValidator::execute(context, navdata, ValidationScope::Flight),
        DepartureAirportValidator::execute(context, navdata, ValidationScope::Flight),
        ArrivalAircraftTypeValidator::execute(context, navdata, ValidationScope::Flight),
        ArrivalAirportValidator::execute(context, navdata, ValidationScope::Flight),
        DirectionalCruiseAltitudeValidator::execute(context, navdata, ValidationScope::Flight),
        EquipmentRvsmValidator::execute(context, navdata, ValidationScope::Flight),
        EquipmentRnav1Validator::execute(context, navdata, ValidationScope::Flight),
        EquipmentRnpArValidator::execute(context, navdata, ValidationScope::Flight),
    ]
}
