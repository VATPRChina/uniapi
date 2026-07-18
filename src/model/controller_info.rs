use chrono::{DateTime, Utc};
use std::str::FromStr;

use crate::model::user::UserSummary;
use crate::model::user_controller_state::UserControllerState;

#[derive(Debug, Clone)]
pub struct ControllerInfo {
    pub user: UserSummary,
    pub is_visiting: bool,
    pub is_absent: bool,
    pub rating: ControllerRating,
    pub permissions: Vec<ControllerPermission>,
}

#[derive(Debug, Clone)]
pub struct ControllerPermission {
    pub position_kind: ControllerPositionKind,
    pub state: UserControllerState,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerRating {
    Obs,
    S1,
    S2,
    S3,
    C1,
    C3,
    I1,
    I3,
}

impl ControllerRating {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Obs => "OBS",
            Self::S1 => "S1",
            Self::S2 => "S2",
            Self::S3 => "S3",
            Self::C1 => "C1",
            Self::C3 => "C3",
            Self::I1 => "I1",
            Self::I3 => "I3",
        }
    }
}

impl std::fmt::Display for ControllerRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ControllerRating {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "OBS" => Ok(Self::Obs),
            "S1" => Ok(Self::S1),
            "S2" => Ok(Self::S2),
            "S3" => Ok(Self::S3),
            "C1" => Ok(Self::C1),
            "C3" => Ok(Self::C3),
            "I1" => Ok(Self::I1),
            "I3" => Ok(Self::I3),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerPositionKind {
    Del,
    Gnd,
    Twr,
    T2,
    App,
    Ctr,
    Fss,
    Fmp,
}

impl ControllerPositionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Del => "DEL",
            Self::Gnd => "GND",
            Self::Twr => "TWR",
            Self::T2 => "T2",
            Self::App => "APP",
            Self::Ctr => "CTR",
            Self::Fss => "FSS",
            Self::Fmp => "FMP",
        }
    }
}

impl std::fmt::Display for ControllerPositionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ControllerPositionKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "DEL" => Ok(Self::Del),
            "GND" => Ok(Self::Gnd),
            "TWR" => Ok(Self::Twr),
            "T2" => Ok(Self::T2),
            "APP" => Ok(Self::App),
            "CTR" => Ok(Self::Ctr),
            "FSS" => Ok(Self::Fss),
            "FMP" => Ok(Self::Fmp),
            _ => Err(()),
        }
    }
}
