use chrono::{DateTime, Utc};
use std::str::FromStr;

use serde::Serialize;
use uuid::Uuid;

pub struct SectorPermission {
    pub has_permission: bool,
}

pub struct CompatFutureController {
    pub callsign: String,
    pub name: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Controller {
    pub user_id: Uuid,
    pub is_visiting: bool,
    pub is_absent: bool,
    pub rating: ControllerRating,
    pub permissions: Vec<ControllerPermission>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ControllerPermission {
    pub position_kind: ControllerPositionKind,
    pub state: UserControllerState,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ControllerSave {
    pub is_visiting: bool,
    pub is_absent: bool,
    pub rating: String,
    pub permissions: Vec<ControllerPermissionSave>,
}

#[derive(Debug, Clone)]
pub struct ControllerPermissionSave {
    pub position_kind_id: String,
    pub state: String,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum UserControllerState {
    Student,
    UnderMentor,
    Solo,
    Certified,
    Mentor,
}

impl UserControllerState {
    pub const fn as_db_str(self) -> &'static str {
        match self {
            Self::Student => "Student",
            Self::UnderMentor => "UnderMentor",
            Self::Solo => "Solo",
            Self::Certified => "Certified",
            Self::Mentor => "Mentor",
        }
    }

    pub const fn to_db_value(self) -> i32 {
        match self {
            Self::Student => 0,
            Self::UnderMentor => 1,
            Self::Solo => 2,
            Self::Certified => 3,
            Self::Mentor => 4,
        }
    }

    pub const fn from_db_value(value: i32) -> Self {
        match value {
            1 => Self::UnderMentor,
            2 => Self::Solo,
            3 => Self::Certified,
            4 => Self::Mentor,
            _ => Self::Student,
        }
    }
}

impl std::fmt::Display for UserControllerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_db_str())
    }
}

impl FromStr for UserControllerState {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Student" => Ok(Self::Student),
            "UnderMentor" => Ok(Self::UnderMentor),
            "Solo" => Ok(Self::Solo),
            "Certified" => Ok(Self::Certified),
            "Mentor" => Ok(Self::Mentor),
            _ => Err(()),
        }
    }
}
