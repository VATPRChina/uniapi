use crate::modules::navdata::models::AnyFix;

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLeg {
    pub from: AnyFix,
    pub to: AnyFix,
    pub identifier: Option<String>,
    /// The connection was recovered as an unknown leg rather than navdata.
    pub is_unknown: bool,
    pub is_sid: bool,
    pub is_star: bool,
    pub direction_restriction: DirectionRestriction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectionRestriction {
    None,
    Forward,
    Backward,
}

impl DirectionRestriction {
    pub fn reversed(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LegKind {
    Airway,
    Sid,
    Star,
    Unknown,
    Direct,
}

impl ResolvedLeg {
    pub fn into_reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            identifier: self.identifier,
            is_unknown: self.is_unknown,
            is_sid: self.is_sid,
            is_star: self.is_star,
            direction_restriction: self.direction_restriction.reversed(),
        }
    }
}
