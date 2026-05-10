use arrayvec::ArrayString;

use crate::model::navdata::{Fix, Identifiable};

#[derive(Debug, Clone, PartialEq)]
pub struct Ndb {
    icao_code: ArrayString<4>,
    identifier: ArrayString<4>,
    latitude: f64,
    longitude: f64,
    kind: NdbKind,
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
    fn latitude(&self) -> f64 {
        self.latitude
    }

    fn longitude(&self) -> f64 {
        self.longitude
    }
}
