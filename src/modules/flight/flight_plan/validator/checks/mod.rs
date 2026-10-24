//! The new validation checks, independent of the legacy warning pipeline.
//! Each placeholder reports `Unavailable` until its validator overrides
//! `Validator::validate`. All checks are exposed through the validations endpoint.

use crate::modules::flight::flight_plan::validator::WarningMessageField;
use crate::modules::flight::models::{
    Flight, ParsedRoute, ValidationScope, ValidatorResult, ValidatorStatus,
};
use crate::modules::navdata::models::PreferredRoute;
use crate::modules::navdata::service::NavdataService;

mod airport;
mod cruising_level;
mod equipment;
mod flight;
mod route;

pub use airport::{
    ArrivalAircraftTypeValidator, ArrivalAirportValidator, DepartureAircraftTypeValidator,
    DepartureAirportValidator,
};
pub use cruising_level::{
    DirectionalCruiseAltitudeValidator, PreferredRouteCruiseAltitudeValidator,
};
pub use equipment::{EquipmentRnav1Validator, EquipmentRnpArValidator, EquipmentRvsmValidator};
pub use flight::{
    AircraftTypeValidator, CallsignValidator, FlightRuleWeatherValidator, WakeCategoryValidator,
};
pub use route::{
    AirwayApprovalValidator, AirwayDirectionValidator, PreferredRouteValidator,
    RouteDirectLegValidator, RouteUnknownLegOrFixValidator,
};

/// Borrow the inputs shared by the new flight-plan checks.
#[derive(Debug)]
pub struct ValidationContext<'a> {
    pub flight: &'a Flight,
    pub route: &'a ParsedRoute,
    pub preferred_route: Option<&'a PreferredRoute>,
    /// Candidates for the departure/arrival pair, including private routes.
    pub preferred_routes: &'a [&'a PreferredRoute],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationLevel {
    Flight,
    Segment,
    Leg,
}

/// A check outcome becomes an API result only when attached to its execution scope.
pub struct CheckResult {
    pub validator_ident: String,
    pub field: WarningMessageField,
    pub status: ValidatorStatus,
    pub warnings: Vec<super::WarningMessage>,
}

impl CheckResult {
    pub fn scoped(self, scope: ValidationScope) -> ValidatorResult {
        ValidatorResult {
            scope,
            validator_ident: self.validator_ident,
            field: self.field,
            status: self.status,
            warnings: self.warnings,
        }
    }
}

pub trait Validator {
    const LEVEL: ValidationLevel = ValidationLevel::Flight;

    fn execute(
        context: &ValidationContext<'_>,
        navdata: &NavdataService,
        scope: ValidationScope,
    ) -> ValidatorResult {
        let level = match scope {
            ValidationScope::Flight => ValidationLevel::Flight,
            ValidationScope::Segment { .. } => ValidationLevel::Segment,
            ValidationScope::Leg { .. } => ValidationLevel::Leg,
        };
        assert_eq!(Self::LEVEL, level, "validator executed at the wrong level");
        Self::validate(context, navdata).scoped(scope)
    }

    /// Stable identifier used by `ValidatorResult::validator_ident`.
    const IDENT: &'static str;

    /// Flight-plan field this check validates.
    const FIELD: WarningMessageField;

    /// Override this placeholder when implementing a check.
    fn validate(_context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        CheckResult {
            validator_ident: Self::IDENT.to_owned(),
            field: Self::FIELD,
            status: ValidatorStatus::Unavailable,
            warnings: Vec::new(),
        }
    }
}
