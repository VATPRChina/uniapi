use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
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

impl fmt::Display for UserControllerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseUserControllerStateError {
    #[error("invalid controller state {0}")]
    InvalidString(String),
}

impl FromStr for UserControllerState {
    type Err = ParseUserControllerStateError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Student" => Ok(Self::Student),
            "UnderMentor" => Ok(Self::UnderMentor),
            "Solo" => Ok(Self::Solo),
            "Certified" => Ok(Self::Certified),
            "Mentor" => Ok(Self::Mentor),
            _ => Err(ParseUserControllerStateError::InvalidString(
                value.to_owned(),
            )),
        }
    }
}
