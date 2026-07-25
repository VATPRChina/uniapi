use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PreferredRoute {
    pub name: String,
    pub departure: String,
    pub arrival: String,
    pub raw_route: String,
    pub cruising_level_restriction: LevelRestrictionType,
    pub allowed_altitudes: Vec<i32>,
    pub minimal_altitude: i32,
    pub remarks: String,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub is_public: bool,
}

impl PreferredRoute {
    #[allow(unused)]
    pub fn is_public(&self) -> bool {
        self.remarks.to_ascii_lowercase().contains("aip route")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LevelRestrictionType {
    StandardEven,
    StandardOdd,
    Standard,
    FlightLevelEven,
    FlightLevelOdd,
}

impl LevelRestrictionType {
    pub(crate) fn from_csv_str(value: &str) -> Result<Self, ()> {
        Ok(match value {
            "SE" => LevelRestrictionType::StandardEven,
            "SO" => LevelRestrictionType::StandardOdd,
            "FE" => LevelRestrictionType::FlightLevelEven,
            "FO" => LevelRestrictionType::FlightLevelOdd,
            _ => Self::Standard,
        })
    }
}
