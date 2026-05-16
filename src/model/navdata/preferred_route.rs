use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PreferredRoute {
    pub id: Uuid,
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

impl PreferredRoute {
    #[allow(unused)]
    pub fn is_public(&self) -> bool {
        self.remarks.to_ascii_lowercase().contains("aip route")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LevelRestrictionType {
    StandardEven,
    StandardOdd,
    Standard,
    FlightLevelEven,
    FlightLevelOdd,
    FlightLevel,
}

impl FromStr for LevelRestrictionType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "StandardEven" | "standard_even" | "standard-even" => Self::StandardEven,
            "StandardOdd" | "standard_odd" | "standard-odd" => Self::StandardOdd,
            "FlightLevelEven" | "flight_level_even" | "flight-level-even" => Self::FlightLevelEven,
            "FlightLevelOdd" | "flight_level_odd" | "flight-level-odd" => Self::FlightLevelOdd,
            "FlightLevel" | "flight_level" | "flight-level" => Self::FlightLevel,
            _ => Self::Standard,
        })
    }
}
