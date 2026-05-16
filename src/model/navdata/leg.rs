use crate::model::navdata::AnyFix;

#[derive(Debug, Clone, PartialEq)]
pub struct Leg {
    pub from: FixRef,
    pub to: FixRef,
    pub identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixRef {
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLeg {
    pub from: AnyFix,
    pub to: AnyFix,
    pub identifier: Option<String>,
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
            direction_restriction: match self.direction_restriction {
                DirectionRestriction::None => DirectionRestriction::None,
                DirectionRestriction::Forward => DirectionRestriction::Backward,
                DirectionRestriction::Backward => DirectionRestriction::Forward,
            },
        }
    }
}
