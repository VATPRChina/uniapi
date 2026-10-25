use crate::modules::flight::models::{
    Flight, ParsedRoute, ValidationField, ValidationProblem, ValidatorExecution,
    ValidatorIdentifier, ValidatorStatus,
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
pub use flight::{AircraftTypeValidator, FlightRuleWeatherValidator, WakeCategoryValidator};
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

/// A check outcome receives a field or leg range when it is executed.
pub struct CheckResult {
    pub id: ValidatorIdentifier,
    pub field: ValidationField,
    pub status: ValidatorStatus,
    pub problems: Vec<ValidationProblem>,
}

impl CheckResult {
    pub fn scoped(self, field: ValidationField) -> ValidatorExecution {
        ValidatorExecution {
            id: self.id,
            field,
            status: self.status,
            problems: self.problems,
        }
    }
}

pub trait Validator {
    const LEVEL: ValidationLevel = ValidationLevel::Flight;
    const IDENT: ValidatorIdentifier;
    const FIELD: ValidationField;

    fn execute(
        context: &ValidationContext<'_>,
        navdata: &NavdataService,
        field: ValidationField,
    ) -> ValidatorExecution {
        let level = match &field {
            ValidationField::Leg { .. } => ValidationLevel::Leg,
            ValidationField::MultipleLegs { .. } => ValidationLevel::Segment,
            _ => ValidationLevel::Flight,
        };
        assert_eq!(Self::LEVEL, level, "validator executed at the wrong level");
        Self::validate(context, navdata).scoped(field)
    }

    fn validate(_context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        CheckResult {
            id: Self::IDENT,
            field: Self::FIELD,
            status: ValidatorStatus::Unavailable,
            problems: Vec::new(),
        }
    }
}
