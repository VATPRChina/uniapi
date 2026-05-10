use arrayvec::ArrayString;

use crate::model::navdata::{Fix, Identifiable};

#[derive(Debug, Clone, PartialEq)]
pub struct Airport {
    identifier: ArrayString<4>,
    latitude: f64,
    longitude: f64,
}

impl Identifiable for Airport {
    fn icao_code(&self) -> &str {
        ""
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl Fix for Airport {
    fn latitude(&self) -> f64 {
        self.latitude
    }

    fn longitude(&self) -> f64 {
        self.longitude
    }
}
