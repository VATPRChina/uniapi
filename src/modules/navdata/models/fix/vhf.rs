use arrayvec::ArrayString;

use crate::modules::navdata::models::{Fix, Identifiable};
use crate::utils::geo::{Latitude, Longitude};

#[derive(Debug, Clone)]
pub struct Vhf {
    pub icao_code: ArrayString<4>,
    pub identifier: ArrayString<4>,
    pub latitude: Latitude,
    pub longitude: Longitude,
}

impl Identifiable for Vhf {
    fn icao_code(&self) -> &str {
        &self.icao_code
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl Fix for Vhf {
    fn latitude(&self) -> Option<Latitude> {
        Some(self.latitude)
    }

    fn longitude(&self) -> Option<Longitude> {
        Some(self.longitude)
    }
}

impl PartialEq for Vhf {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
            && self.latitude == other.latitude
            && self.longitude == other.longitude
    }
}

impl Eq for Vhf {}
