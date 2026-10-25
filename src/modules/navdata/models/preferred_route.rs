use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone)]
pub struct PreferredRoute {
    pub name: String,
    pub departure: String,
    pub arrival: String,
    pub raw_route: String,
    pub cruising_level_restriction: CruiseAltitudeType,
    pub allowed_altitudes: Vec<i32>,
    pub minimal_altitude: i32,
    pub remarks: String,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub is_public: bool,
}

impl PreferredRoute {
    pub fn is_public(&self) -> bool {
        !self.remarks.to_ascii_lowercase().contains("aip route")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CruiseAltitudeType {
    StandardEven,
    StandardOdd,
    Standard,
    FlightLevelEven,
    FlightLevelOdd,
}

impl CruiseAltitudeType {
    pub(crate) fn from_code(value: &str) -> Result<Self, ()> {
        Ok(match value {
            "SE" => CruiseAltitudeType::StandardEven,
            "SO" => CruiseAltitudeType::StandardOdd,
            "FE" => CruiseAltitudeType::FlightLevelEven,
            "FO" => CruiseAltitudeType::FlightLevelOdd,
            _ => Self::Standard,
        })
    }
}
