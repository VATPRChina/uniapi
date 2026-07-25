use chrono::{DateTime, Utc};
use ulid::Ulid;

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
