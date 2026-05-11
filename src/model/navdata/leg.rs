use crate::model::navdata::AnyFix;

#[derive(Debug, Clone, PartialEq)]
pub struct Leg {
    from: FixRef,
    to: FixRef,
    identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixRef {
    identifier: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLeg {
    from: AnyFix,
    to: AnyFix,
    identifier: Option<String>,
    direction_restriction: DirectionRestriction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectionRestriction {
    None,
    Forward,
    Backward,
}
