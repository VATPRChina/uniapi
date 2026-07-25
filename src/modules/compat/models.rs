use chrono::{DateTime, Utc};

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

pub struct CompatFutureController {
    pub callsign: String,
    pub name: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}
