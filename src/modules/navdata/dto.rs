use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct PreferredRouteSaveRequest {
    pub departure: String,
    pub arrival: String,
    pub raw_route: String,
    pub cruising_level_restriction: LevelRestrictionType,
    #[serde(default)]
    pub allowed_altitudes: Vec<i32>,
    pub minimal_altitude: i32,
    pub remarks: String,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LevelRestrictionType {
    StandardEven,
    StandardOdd,
    Standard,
    FlightLevelEven,
    FlightLevelOdd,
    FlightLevel,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct PreferredRouteDto {
    pub id: String,
    pub departure: String,
    pub arrival: String,
    pub raw_route: String,
    pub cruising_level_restriction: LevelRestrictionType,
    pub allowed_altitudes: Vec<i32>,
    pub minimal_altitude: i32,
    pub remarks: String,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
}
