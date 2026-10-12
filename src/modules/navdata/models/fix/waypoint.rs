use arrayvec::ArrayString;

use crate::modules::navdata::models::{Fix, Identifiable};
use crate::utils::geo::{Latitude, Longitude};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Waypoint {
    pub icao_code: ArrayString<4>,
    pub identifier: ArrayString<5>,
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub kind: WaypointKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointKind {
    Enroute,
    #[allow(unused)]
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
    fn latitude(&self) -> Option<Latitude> {
        Some(self.latitude)
    }

    fn longitude(&self) -> Option<Longitude> {
        Some(self.longitude)
    }
}
