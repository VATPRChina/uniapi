use arrayvec::ArrayString;

use crate::modules::navdata::models::{Fix, Identifiable};
use crate::utils::geo::{Latitude, Longitude};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Airport {
    pub identifier: ArrayString<4>,
    pub latitude: Latitude,
    pub longitude: Longitude,
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
    fn latitude(&self) -> Option<Latitude> {
        Some(self.latitude)
    }

    fn longitude(&self) -> Option<Longitude> {
        Some(self.longitude)
    }
}
