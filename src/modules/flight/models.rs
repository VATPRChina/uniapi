use chrono::{DateTime, Utc};
use ulid::Ulid;

use crate::modules::controller::models::CompatFutureController;
use crate::modules::flight::flight_plan::validator::{WarningMessage, WarningMessageField};
use crate::modules::navdata::models::ResolvedLeg;

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
    pub id: Ulid,
    pub cid: String,
    pub callsign: String,
    pub last_observed_at: DateTime<Utc>,
    pub departure: String,
    pub arrival: String,
    pub equipment: String,
    pub navigation_performance: String,
    pub transponder: String,
    pub raw_route: String,
    pub aircraft: String,
    pub altitude: i64,
    pub cruising_level: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedRoute {
    pub legs: Vec<ResolvedLeg>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlightDeliveryData {
    pub parsed_route: ParsedRoute,
    pub sid_candidates: Vec<SidCandidate>,
    pub validations: Vec<ValidatorResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SidCandidate {
    pub identifier: String,
    pub runway: String,
    pub enroute_transition: String,
}

// TODO: add a separate DTO
#[derive(Debug, Clone, PartialEq, serde::Serialize, utoipa::ToSchema)]
pub struct ValidatorResult {
    pub validator_ident: String,
    pub field: WarningMessageField,
    pub status: ValidatorStatus,
    pub warnings: Vec<WarningMessage>,
}

// TODO: add a separate DTO
#[derive(Debug, Clone, PartialEq, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ValidatorStatus {
    Pass,
    Suppressed,
    Rejected,
    Unavailable,
}
