use serde::Serialize;

use crate::modules::flight::flight_plan::ParseRouteError;
use crate::modules::flight::flight_plan::v1::parser::ParserError;
use crate::modules::flight::models::ValidatorResult;
use crate::modules::navdata::service::InvalidNavdataError;
use crate::modules::navdata::service::NavdataService;

use self::checks::*;

pub mod checks;
pub(crate) mod matcher;

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

/// Run every flight-plan check in a stable order.
pub fn validate_all(
    context: &ValidationContext<'_>,
    navdata: &NavdataService,
) -> Result<Vec<ValidatorResult>, ValidatorError> {
    Ok(vec![
        CallsignValidator::validate(context, navdata),
        FlightRuleWeatherValidator::validate(context, navdata),
        AircraftTypeValidator::validate(context, navdata),
        WakeCategoryValidator::validate(context, navdata),
        DepartureAircraftTypeValidator::validate(context, navdata),
        DepartureAirportValidator::validate(context, navdata),
        ArrivalAircraftTypeValidator::validate(context, navdata),
        ArrivalAirportValidator::validate(context, navdata),
        ChinaRvsmLevelValidator::validate(context, navdata),
        CruisingLevelRestrictionValidator::validate(context, navdata),
        EquipmentRvsmValidator::validate(context, navdata),
        EquipmentRnav1Validator::validate(context, navdata),
        EquipmentRnpArValidator::validate(context, navdata),
        PreferredRouteValidator::validate(context, navdata),
        RouteDirectLegValidator::validate(context, navdata),
        RouteUnknownLegOrFixValidator::validate(context, navdata),
        AirwayApprovalValidator::validate(context, navdata),
        AirwayDirectionValidator::validate(context, navdata),
    ])
}
