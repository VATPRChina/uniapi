use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::modules::controller::models::CompatFutureController;
use crate::modules::flight::models::{
    FlightRule, ValidatorIdentifier, ValidatorStatus, WakeCategory,
};
use crate::modules::navdata::models::CruiseAltitudeType;

use super::models::{CompatController, CompatPilot, CompatStatus, Flight};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct MetarQuery {
    pub id: String,
}

#[derive(Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TemporaryFlightQuery {
    pub departure: String,
    pub arrival: String,
    #[serde(default)]
    pub aircraft: String,
    #[serde(default)]
    /// I (IFR) or V (VFR); defaults to I when omitted or empty.
    pub flight_rules: String,
    #[serde(default)]
    /// J, H, M or L; defaults to M when omitted or empty.
    pub wake_category: String,
    #[serde(default)]
    pub equipment: String,
    #[serde(default)]
    pub navigation_performance: String,
    #[serde(default)]
    pub transponder: String,
    #[serde(default)]
    pub raw_route: String,
    #[serde(default)]
    pub cruising_level: i64,
}

impl From<TemporaryFlightQuery> for Flight {
    fn from(query: TemporaryFlightQuery) -> Self {
        Self {
            cid: String::new(),
            flight_plan: super::models::FlightPlan {
                callsign: String::new(),
                departure: query.departure.trim().to_ascii_uppercase(),
                arrival: query.arrival.trim().to_ascii_uppercase(),
                aircraft: query.aircraft.trim().to_ascii_uppercase(),
                flight_rules: if query.flight_rules.trim().eq_ignore_ascii_case("V") {
                    FlightRule::Vfr
                } else {
                    FlightRule::Ifr
                },
                wake_category: match query.wake_category.trim().to_ascii_uppercase().as_str() {
                    "J" => WakeCategory::Super,
                    "H" => WakeCategory::Heavy,
                    "L" => WakeCategory::Light,
                    _ => WakeCategory::Medium,
                },
                equipment: query.equipment.trim().to_ascii_uppercase(),
                navigation_performance: query.navigation_performance.trim().to_ascii_uppercase(),
                transponder: query.transponder.trim().to_ascii_uppercase(),
                raw_route: query.raw_route.trim().to_ascii_uppercase(),
                cruising_level: query.cruising_level,
            },
            last_observed_at: Utc::now(),
            altitude: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlightDto {
    pub cid: String,
    pub flight_plan: FlightPlanDto,
    pub last_observed_at: DateTime<Utc>,
    pub altitude: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlightPlanDto {
    pub callsign: String,
    pub departure: String,
    pub arrival: String,
    pub equipment: String,
    pub navigation_performance: String,
    pub transponder: String,
    pub raw_route: String,
    pub aircraft: String,
    pub flight_rules: FlightRule,
    pub wake_category: WakeCategory,
    pub cruising_level: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlightValidationDto {
    pub validators: Vec<ValidatorExecutionDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ValidatorExecutionDto {
    pub id: ValidatorIdentifier,
    pub field: ValidationFieldDto,
    pub status: ValidatorStatus,
    pub problems: Vec<ValidationProblemDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ValidationFieldDto {
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
    Leg {
        index: usize,
    },
    /// Zero-based, half-open range of legs: `from` is inclusive, `to` is exclusive.
    MultipleLegs {
        from: usize,
        to: usize,
    },
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ValidationProblemDto {
    InvalidAirport,
    InvalidAircraftType,
    WakeCategoryMismatch {
        expected: String,
    },
    AircraftWingspanTooLarge {
        maximum: String,
    },
    NoRvsm,
    NoRnav1,
    RnpAr,
    RnpArWithoutRf,
    NoTransponder,
    RouteDirectSegment {
        from: String,
        to: String,
    },
    RouteUnknownLegOrFix {
        ident: String,
    },
    RouteLegDirection {
        from: String,
        ident: String,
        to: String,
    },
    AirwayRequireApproval {
        from: String,
        ident: String,
        to: String,
    },
    NotPreferredRoute {
        from: String,
        to: String,
        raw_route: String,
    },
    CruisingLevelMismatch {
        expected_type: CruiseAltitudeType,
    },
    CruisingLevelNotAllowed {
        allowlist_ft: Vec<i64>,
    },
    CruisingLevelTooLow {
        minimum_ft: i64,
    },
    RouteMatchPreferred,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatVatprcStatusDto {
    pub last_updated: DateTime<Utc>,
    pub pilots: Vec<CompatPilotDto>,
    pub controllers: Vec<CompatControllerDto>,
    pub future_controllers: Vec<CompatFutureControllerDto>,
}

impl From<CompatStatus> for CompatVatprcStatusDto {
    fn from(status: CompatStatus) -> Self {
        Self {
            last_updated: status.last_updated,
            pilots: status.pilots.into_iter().map(Into::into).collect(),
            controllers: status.controllers.into_iter().map(Into::into).collect(),
            future_controllers: status
                .future_controllers
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatPilotDto {
    pub cid: i32,
    pub name: String,
    pub callsign: String,
    pub departure: Option<String>,
    pub arrival: Option<String>,
    pub aircraft: Option<String>,
}

impl From<CompatPilot> for CompatPilotDto {
    fn from(pilot: CompatPilot) -> Self {
        Self {
            cid: pilot.cid,
            name: pilot.name,
            callsign: pilot.callsign,
            departure: pilot.departure,
            arrival: pilot.arrival,
            aircraft: pilot.aircraft,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatControllerDto {
    pub cid: i32,
    pub name: String,
    pub callsign: String,
    pub frequency: String,
}

impl From<CompatController> for CompatControllerDto {
    fn from(controller: CompatController) -> Self {
        Self {
            cid: controller.cid,
            name: controller.name,
            callsign: controller.callsign,
            frequency: controller.frequency,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatFutureControllerDto {
    pub callsign: String,
    pub name: String,
    pub start: String,
    pub start_utc: DateTime<Utc>,
    pub end: String,
    pub end_utc: DateTime<Utc>,
}

impl From<CompatFutureController> for CompatFutureControllerDto {
    fn from(controller: CompatFutureController) -> Self {
        Self {
            callsign: controller.callsign,
            name: controller.name,
            start: controller.start_at.format("%d %H:%M").to_string(),
            start_utc: controller.start_at,
            end: controller.end_at.format("%d %H:%M").to_string(),
            end_utc: controller.end_at,
        }
    }
}

impl From<Flight> for FlightDto {
    fn from(flight: Flight) -> Self {
        Self {
            cid: flight.cid,
            flight_plan: flight.flight_plan.into(),
            last_observed_at: flight.last_observed_at,
            altitude: flight.altitude,
        }
    }
}

impl From<super::models::FlightPlan> for FlightPlanDto {
    fn from(plan: super::models::FlightPlan) -> Self {
        Self {
            callsign: plan.callsign,
            departure: plan.departure,
            arrival: plan.arrival,
            equipment: plan.equipment,
            navigation_performance: plan.navigation_performance,
            transponder: plan.transponder,
            raw_route: plan.raw_route,
            aircraft: plan.aircraft,
            flight_rules: plan.flight_rules,
            wake_category: plan.wake_category,
            cruising_level: plan.cruising_level,
        }
    }
}

impl From<Vec<super::models::ValidatorExecution>> for FlightValidationDto {
    fn from(validators: Vec<super::models::ValidatorExecution>) -> Self {
        Self {
            validators: validators.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<super::models::ValidatorExecution> for ValidatorExecutionDto {
    fn from(execution: super::models::ValidatorExecution) -> Self {
        Self {
            id: execution.id,
            field: execution.field.into(),
            status: execution.status,
            problems: execution.problems.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<super::models::ValidationField> for ValidationFieldDto {
    fn from(field: super::models::ValidationField) -> Self {
        use super::models::ValidationField as Field;
        match field {
            Field::Callsign => Self::Callsign,
            Field::FlightRules => Self::FlightRules,
            Field::AircraftType => Self::AircraftType,
            Field::WakeCategory => Self::WakeCategory,
            Field::Equipment => Self::Equipment,
            Field::Transponder => Self::Transponder,
            Field::Departure => Self::Departure,
            Field::Airspeed => Self::Airspeed,
            Field::CruisingLevel => Self::CruisingLevel,
            Field::Route => Self::Route,
            Field::Arrival => Self::Arrival,
            Field::NavigationPerformance => Self::NavigationPerformance,
            Field::Leg { index } => Self::Leg { index },
            Field::MultipleLegs { indices } => Self::MultipleLegs {
                from: indices.start,
                to: indices.end,
            },
        }
    }
}

impl From<super::models::ValidationProblem> for ValidationProblemDto {
    fn from(problem: super::models::ValidationProblem) -> Self {
        use super::models::ValidationProblem as Problem;
        match problem {
            Problem::InvalidAirport => Self::InvalidAirport,
            Problem::InvalidAircraftType => Self::InvalidAircraftType,
            Problem::WakeCategoryMismatch { expected } => Self::WakeCategoryMismatch { expected },
            Problem::AircraftWingspanTooLarge { maximum } => {
                Self::AircraftWingspanTooLarge { maximum }
            }
            Problem::NoRvsm => Self::NoRvsm,
            Problem::NoRnav1 => Self::NoRnav1,
            Problem::RnpAr => Self::RnpAr,
            Problem::RnpArWithoutRf => Self::RnpArWithoutRf,
            Problem::NoTransponder => Self::NoTransponder,
            Problem::RouteDirectSegment { from, to } => Self::RouteDirectSegment { from, to },
            Problem::RouteUnknownLegOrFix { ident } => Self::RouteUnknownLegOrFix { ident },
            Problem::RouteLegDirection { from, ident, to } => {
                Self::RouteLegDirection { from, ident, to }
            }
            Problem::AirwayRequireApproval { from, ident, to } => {
                Self::AirwayRequireApproval { from, ident, to }
            }
            Problem::NotPreferredRoute {
                from,
                to,
                raw_route,
            } => Self::NotPreferredRoute {
                from,
                to,
                raw_route,
            },
            Problem::CruisingLevelMismatch { expected_type } => {
                Self::CruisingLevelMismatch { expected_type }
            }
            Problem::CruisingLevelNotAllowed { allowlist_ft } => {
                Self::CruisingLevelNotAllowed { allowlist_ft }
            }
            Problem::CruisingLevelTooLow { minimum_ft } => Self::CruisingLevelTooLow { minimum_ft },
            Problem::RouteMatchPreferred => Self::RouteMatchPreferred,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::flight::models::{ValidationField, ValidationProblem, ValidatorExecution};
    use serde_json::json;

    #[test]
    fn validation_snapshot_preserves_structured_problems_and_half_open_ranges() {
        let validation = FlightValidationDto::from(vec![ValidatorExecution {
            id: ValidatorIdentifier::PreferredRouteCruiseAltitude,
            field: ValidationField::MultipleLegs { indices: 2..5 },
            status: ValidatorStatus::Rejected,
            problems: vec![
                ValidationProblem::CruisingLevelMismatch {
                    expected_type: CruiseAltitudeType::StandardOdd,
                },
                ValidationProblem::CruisingLevelNotAllowed {
                    allowlist_ft: vec![29100, 33100],
                },
                ValidationProblem::CruisingLevelTooLow { minimum_ft: 29100 },
            ],
        }]);
        assert_eq!(
            serde_json::to_value(validation).unwrap(),
            json!({
                "validators": [{
                    "id": "PreferredRouteCruiseAltitude",
                    "field": { "kind": "MultipleLegs", "from": 2, "to": 5 },
                    "status": "rejected",
                    "problems": [
                        { "kind": "CruisingLevelMismatch", "expected_type": "standard-odd" },
                        { "kind": "CruisingLevelNotAllowed", "allowlist_ft": [29100, 33100] },
                        { "kind": "CruisingLevelTooLow", "minimum_ft": 29100 }
                    ]
                }]
            })
        );
    }
}
