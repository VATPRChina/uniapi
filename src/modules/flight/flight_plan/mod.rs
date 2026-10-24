use crate::modules::navdata::models::AnyFix;

pub(crate) mod altitude;
pub(crate) mod diagnostics;
mod matcher;
mod route_parser;
mod segmenter;
pub mod v1;
pub mod validator;

pub use matcher::PreferredRouteMatcher;
pub use segmenter::{RouteSegment, RouteSegmenter, Segment, SegmentKind};

pub use route_parser::{ParseRouteError, RouteParseStep, parse_route, parse_route_with_observer};

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
