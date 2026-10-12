use arrayvec::ArrayString;

use crate::modules::navdata::models::{Fix, Identifiable};
use crate::utils::geo::{Latitude, Longitude};

#[derive(Debug, Clone, PartialEq, Eq)]
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
