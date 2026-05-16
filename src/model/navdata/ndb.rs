use arrayvec::ArrayString;

use crate::model::navdata::{Fix, Identifiable};

#[derive(Debug, Clone, PartialEq)]
pub struct Ndb {
    pub icao_code: ArrayString<4>,
    pub identifier: ArrayString<4>,
    pub latitude: f64,
    pub longitude: f64,
    pub kind: NdbKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NdbKind {
    Enroute,
    #[allow(unused)]
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
