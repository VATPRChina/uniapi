use crate::modules::navdata::models::ResolvedLeg;
use arrayvec::ArrayString;

#[derive(Debug, Clone, PartialEq)]
pub struct Airway {
    pub identifier: ArrayString<7>,
    pub legs: Vec<ResolvedLeg>,
}
