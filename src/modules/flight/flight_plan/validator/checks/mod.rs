//! The new validation checks, independent of the legacy warning pipeline.
//! Each placeholder reports `NotImplemented` until its validator overrides
//! `Validator::validate`. All checks are exposed through the validations endpoint.

use crate::modules::flight::models::{Flight, ParsedRoute, ValidatorResult, ValidatorStatus};
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
pub use cruising_level::{ChinaRvsmLevelValidator, CruisingLevelRestrictionValidator};
pub use equipment::{
    EquipmentRnav1Validator, EquipmentRnpArValidator, EquipmentRnpValidator, EquipmentRvsmValidator,
};
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
}

pub trait Validator {
    /// Stable identifier used by `ValidatorResult::validator_ident`.
    const IDENT: &'static str;

    /// Override this placeholder when implementing a check.
    fn validate(_context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        ValidatorResult {
            validator_ident: Self::IDENT.to_owned(),
            status: ValidatorStatus::NotImplemented,
            warnings: Vec::new(),
        }
    }
}
