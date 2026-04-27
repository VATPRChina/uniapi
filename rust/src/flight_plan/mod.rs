use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

pub mod lexer;
pub mod parser;

#[derive(Debug, Clone, PartialEq)]
pub struct Fix {
    pub kind: FixKind,
    pub icao_code: Option<String>,
    pub identifier: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
}

impl Fix {
    pub fn identified(
        kind: FixKind,
        icao_code: impl Into<String>,
        identifier: impl Into<String>,
        latitude: f64,
        longitude: f64,
    ) -> Self {
        Self {
            kind,
            icao_code: Some(icao_code.into()),
            identifier: Some(identifier.into()),
            latitude,
            longitude,
        }
    }

    pub fn geo(latitude: f64, longitude: f64) -> Self {
        Self {
            kind: FixKind::GeoPoint,
            icao_code: None,
            identifier: None,
            latitude,
            longitude,
        }
    }

    pub fn name(&self) -> String {
        self.identifier
            .clone()
            .unwrap_or_else(|| format!("{:.6},{:.6}", self.latitude, self.longitude))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixKind {
    Airport,
    Waypoint,
    VhfNavaid,
    NdbNavaid,
    GeoPoint,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RouteToken {
    Unknown {
        value: String,
    },
    Fix {
        value: String,
        fix: Fix,
    },
    DirectLeg {
        value: String,
    },
    AirwayLeg {
        value: String,
    },
    SidLeg {
        value: String,
        procedure: Option<String>,
    },
    StarLeg {
        value: String,
        procedure: Option<String>,
    },
    SpeedAndAltitude {
        value: String,
    },
}

impl RouteToken {
    pub fn value(&self) -> &str {
        match self {
            Self::Unknown { value }
            | Self::Fix { value, .. }
            | Self::DirectLeg { value }
            | Self::AirwayLeg { value }
            | Self::SidLeg { value, .. }
            | Self::StarLeg { value, .. }
            | Self::SpeedAndAltitude { value } => value,
        }
    }

    pub fn is_fix(&self) -> bool {
        matches!(self, Self::Fix { .. })
    }

    pub fn as_fix(&self) -> Option<&Fix> {
        match self {
            Self::Fix { fix, .. } => Some(fix),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Leg {
    Direct(DirectLeg),
    Airway(AirwayLeg),
}

impl Leg {
    pub fn from(&self) -> &Fix {
        match self {
            Self::Direct(leg) => &leg.from,
            Self::Airway(leg) => &leg.from,
        }
    }

    pub fn to(&self) -> &Fix {
        match self {
            Self::Direct(leg) => &leg.to,
            Self::Airway(leg) => &leg.to,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirectLeg {
    pub from: Fix,
    pub to: Fix,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AirwayLeg {
    pub from: Fix,
    pub to: Fix,
    pub identifier: String,
    pub direction: AirwayDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AirwayDirection {
    Forward,
    Backward,
    Both,
}

#[derive(Debug, Clone)]
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
