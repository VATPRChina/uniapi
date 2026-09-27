use crate::modules::navdata::models::{AnyFix, Fix};

#[derive(Debug, Clone, PartialEq)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
}

impl GeoPoint {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }
}

impl Fix for GeoPoint {
    fn latitude(&self) -> f64 {
        self.latitude
    }

    fn longitude(&self) -> f64 {
        self.longitude
    }
}

impl From<GeoPoint> for AnyFix {
    fn from(val: GeoPoint) -> Self {
        AnyFix::GeoPoint(val)
    }
}
