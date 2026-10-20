use crate::modules::navdata::models::{AnyFix, ChinaMainlandBoundary};

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLeg {
    pub from: AnyFix,
    pub to: AnyFix,
    pub identifier: Option<String>,
    /// The connection was recovered as an unknown leg rather than navdata.
    pub is_unknown: bool,
    pub kind: LegKind,
    pub direction_restriction: DirectionRestriction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectionRestriction {
    None,
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegKind {
    Airway,
    Sid,
    Star,
    Direct,
}

impl ResolvedLeg {
    pub fn into_reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            identifier: self.identifier,
            is_unknown: self.is_unknown,
            kind: self.kind,
            direction_restriction: match self.direction_restriction {
                DirectionRestriction::None => DirectionRestriction::None,
                DirectionRestriction::Forward => DirectionRestriction::Backward,
                DirectionRestriction::Backward => DirectionRestriction::Forward,
            },
        }
    }

    pub fn is_china(&self, china_boundary: &ChinaMainlandBoundary) -> bool {
        self.from.is_china_or_unknown(china_boundary) || self.to.is_china_or_unknown(china_boundary)
    }
}
