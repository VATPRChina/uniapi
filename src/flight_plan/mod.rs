use crate::model::navdata::AnyFix;

pub mod lexer;
pub mod parser;
pub mod validator;

#[derive(Debug, Clone, PartialEq)]
pub enum RouteToken {
    Unknown {
        value: String,
    },
    Fix {
        value: String,
        fix: AnyFix,
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
}
