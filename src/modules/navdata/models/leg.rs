use crate::modules::navdata::models::AnyFix;

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLeg {
    pub from: AnyFix,
    pub to: AnyFix,
    pub identifier: Option<String>,
    /// The connection was recovered as an unknown leg rather than navdata.
    pub is_unknown: bool,
    pub direction_restriction: DirectionRestriction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectionRestriction {
    None,
    Forward,
    Backward,
}

impl ResolvedLeg {
    pub fn into_reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            identifier: self.identifier,
            is_unknown: self.is_unknown,
            direction_restriction: match self.direction_restriction {
                DirectionRestriction::None => DirectionRestriction::None,
                DirectionRestriction::Forward => DirectionRestriction::Backward,
                DirectionRestriction::Backward => DirectionRestriction::Forward,
            },
        }
    }
}
