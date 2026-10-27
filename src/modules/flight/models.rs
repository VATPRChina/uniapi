use std::ops::Range;

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::modules::controller::models::CompatFutureController;
use crate::modules::navdata::models::{CruiseAltitudeType, ResolvedLeg};

pub struct CompatStatus {
    pub last_updated: DateTime<Utc>,
    pub pilots: Vec<CompatPilot>,
    pub controllers: Vec<CompatController>,
    pub future_controllers: Vec<CompatFutureController>,
}

pub struct CompatPilot {
    pub cid: i32,
    pub name: String,
    pub callsign: String,
    pub departure: Option<String>,
    pub arrival: Option<String>,
    pub aircraft: Option<String>,
}

pub struct CompatController {
    pub cid: i32,
    pub name: String,
    pub callsign: String,
    pub frequency: String,
}

#[derive(Debug, Clone)]
pub struct Flight {
    pub cid: String,
    pub flight_plan: FlightPlan,
    pub last_observed_at: DateTime<Utc>,
    pub altitude: i64,
}

#[derive(Debug, Clone)]
pub struct FlightPlan {
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

#[derive(Debug, Copy, Clone, Serialize, ToSchema)]
pub enum FlightRule {
    #[serde(rename = "I")]
    Ifr,
    #[serde(rename = "V")]
    Vfr,
}

#[derive(Debug, Copy, Clone, Serialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum WakeCategory {
    #[serde(rename = "J")]
    Super,
    #[serde(rename = "H")]
    Heavy,
    #[serde(rename = "M")]
    Medium,
    #[serde(rename = "L")]
    Light,
}

#[derive(Debug, Clone)]
pub struct ParsedRoute {
    pub legs: Vec<ResolvedLeg>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorExecution {
    pub id: ValidatorIdentifier,
    pub field: ValidationField,
    pub status: ValidatorStatus,
    pub problems: Vec<ValidationProblem>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ValidatorIdentifier {
    Callsign,
    FlightRuleWeather,
    AircraftType,
    WakeCategory,
    DepartureAircraftType,
    DepartureAirport,
    ArrivalAircraftType,
    ArrivalAirport,
    DirectionalCruiseAltitude,
    EquipmentRvsm,
    EquipmentRnav1,
    EquipmentRnpAr,
    PreferredRoute,
    PreferredRouteCruiseAltitude,
    RouteDirectLeg,
    RouteUnknownLegOrFix,
    AirwayApproval,
    AirwayDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationField {
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
    Leg { index: usize },
    MultipleLegs { indices: Range<usize> },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ValidatorStatus {
    Pass,
    Suppressed,
    Rejected,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationProblem {
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
        routes: Vec<PreferredRouteDescription>,
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
    RouteMatchPreferred {
        from: String,
        to: String,
        raw_route: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferredRouteDescription {
    pub from: String,
    pub to: String,
    pub raw_route: String,
}
