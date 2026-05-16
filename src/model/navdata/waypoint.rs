use arrayvec::ArrayString;

use crate::model::navdata::{Fix, Identifiable};

#[derive(Debug, Clone, PartialEq)]
pub struct Waypoint {
    pub icao_code: ArrayString<4>,
    pub identifier: ArrayString<5>,
    pub latitude: f64,
    pub longitude: f64,
    pub kind: WaypointKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointKind {
    Enroute,
    Terminal,
}

impl Identifiable for Waypoint {
    fn icao_code(&self) -> &str {
        &self.icao_code
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl Fix for Waypoint {
    fn latitude(&self) -> f64 {
        self.latitude
    }

    fn longitude(&self) -> f64 {
        self.longitude
    }
}
