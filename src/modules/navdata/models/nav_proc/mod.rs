mod airway;
mod procedure;

pub use airway::Airway;
pub use procedure::{Sid, Star};

use crate::modules::navdata::models::ResolvedLeg;

#[derive(Debug, Clone, PartialEq)]
pub enum NavProc {
    Airway(Airway),
    Sid(Sid),
    Star(Star),
}

impl NavProc {
    pub fn identifier(&self) -> &str {
        match self {
            Self::Airway(proc) => proc.identifier.as_str(),
            Self::Sid(proc) => proc.identifier.as_str(),
            Self::Star(proc) => proc.identifier.as_str(),
        }
    }

    pub fn legs(&self) -> &[ResolvedLeg] {
        match self {
            Self::Airway(proc) => &proc.legs,
            Self::Sid(proc) => &proc.legs,
            Self::Star(proc) => &proc.legs,
        }
    }
}
