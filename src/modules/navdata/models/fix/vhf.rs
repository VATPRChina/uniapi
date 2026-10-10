use arrayvec::ArrayString;

use crate::modules::navdata::models::{Fix, Identifiable};
use crate::utils::geo::{Latitude, Longitude};

#[derive(Clone, PartialEq)]
pub struct Vhf {
    pub icao_code: ArrayString<4>,
    pub identifier: ArrayString<4>,
    pub latitude: f64,
    pub longitude: f64,
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
    fn latitude(&self) -> f64 {
        self.latitude
    }

    fn longitude(&self) -> f64 {
        self.longitude
    }
}

impl std::fmt::Debug for Vhf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vhf")
            .field("icao_code", &self.icao_code)
            .field("identifier", &self.identifier)
            .field("latitude", &Latitude(self.latitude))
            .field("longitude", &Longitude(self.longitude))
            .finish()
    }
}
