use chrono::{DateTime, Utc};
use ulid::Ulid;

use crate::modules::controller::models::CompatFutureController;

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
