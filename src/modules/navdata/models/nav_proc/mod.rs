mod airway;
mod procedure;

pub use airway::Airway;
pub use procedure::{Sid, Star, TerminalProcedure};

use crate::modules::navdata::models::{LegKind, ResolvedLeg};

#[derive(Debug, Clone, PartialEq)]
pub enum NavProc {
    Airway(Airway),
    Sid(Sid),
    Star(Star),
    UnknownAirway(String),
    UnknownSid(String),
    UnknownStar(String),
    Direct,
}

impl NavProc {
    pub fn kind(&self) -> LegKind {
        match self {
            Self::Airway(_) | Self::UnknownAirway(_) => LegKind::Airway,
            Self::Sid(_) | Self::UnknownSid(_) => LegKind::Sid,
            Self::Star(_) | Self::UnknownStar(_) => LegKind::Star,
            Self::Direct => LegKind::Direct,
        }
    }

    pub fn identifier(&self) -> &str {
        match self {
            Self::Airway(proc) => proc.identifier.as_str(),
            Self::Sid(proc) => proc.identifier.as_str(),
            Self::Star(proc) => proc.identifier.as_str(),
            Self::UnknownAirway(ident) => ident.as_str(),
            Self::UnknownSid(ident) => ident.as_str(),
            Self::UnknownStar(ident) => ident.as_str(),
            Self::Direct => "DCT",
        }
    }

    pub fn legs(&self) -> &[ResolvedLeg] {
        match self {
            Self::Airway(proc) => &proc.legs,
            Self::Sid(_) => &[],
            Self::Star(_) => &[],
            Self::UnknownAirway(_) => &[],
            Self::UnknownSid(_) => &[],
            Self::UnknownStar(_) => &[],
            Self::Direct => &[],
        }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(
            self,
            NavProc::UnknownAirway(_) | NavProc::UnknownSid(_) | NavProc::UnknownStar(_)
        )
    }
}
