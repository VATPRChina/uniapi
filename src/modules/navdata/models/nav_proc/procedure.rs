use crate::modules::navdata::models::ResolvedLeg;
use arrayvec::ArrayString;

#[derive(Debug, Clone, PartialEq)]
pub struct Sid {
    pub airport: ArrayString<4>,
    pub identifier: ArrayString<8>,
    /// Published common-route legs only; runway/enroute transitions are excluded.
    pub legs: Vec<ResolvedLeg>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Star {
    pub airport: ArrayString<4>,
    pub identifier: ArrayString<8>,
    /// Published common-route legs only; runway/enroute transitions are excluded.
    pub legs: Vec<ResolvedLeg>,
}
