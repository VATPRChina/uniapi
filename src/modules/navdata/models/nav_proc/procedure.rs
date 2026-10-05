use crate::modules::navdata::models::AnyFix;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcedureKind {
    Sid,
    Star,
}

/// One published runway, common-route, or enroute portion, in sequence order.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureSegment {
    pub route_type: String,
    pub transition: String,
    pub fixes: Vec<AnyFix>,
}
