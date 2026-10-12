use arrayvec::ArrayString;

use crate::modules::navdata::models::{Fix, Identifiable};
use crate::utils::geo::{Latitude, Longitude};

#[derive(Debug, Clone)]
pub struct Ndb {
    pub icao_code: ArrayString<4>,
    pub identifier: ArrayString<4>,
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub kind: NdbKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NdbKind {
    Enroute,
    Terminal,
}

impl Identifiable for Ndb {
    fn icao_code(&self) -> &str {
        &self.icao_code
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl Fix for Ndb {
    fn latitude(&self) -> Option<Latitude> {
        Some(self.latitude)
    }

    fn longitude(&self) -> Option<Longitude> {
        Some(self.longitude)
    }
}

impl PartialEq for Ndb {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
            && self.latitude == other.latitude
            && self.longitude == other.longitude
            && self.kind == other.kind
    }
}

impl Eq for Ndb {}
