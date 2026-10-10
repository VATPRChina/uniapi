use arrayvec::ArrayString;

use crate::modules::navdata::models::{Fix, Identifiable};
use crate::utils::geo::{Latitude, Longitude};

#[derive(Clone, PartialEq)]
pub struct Airport {
    pub identifier: ArrayString<4>,
    pub latitude: f64,
    pub longitude: f64,
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

impl std::fmt::Debug for Airport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Airport")
            .field("identifier", &self.identifier)
            .field("latitude", &Latitude(self.latitude))
            .field("longitude", &Longitude(self.longitude))
            .finish()
    }
}
