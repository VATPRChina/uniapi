use crate::model::navdata::Fix;

#[derive(Debug, Clone, PartialEq)]
pub struct GeoPoint {
    latitude: f64,
    longitude: f64,
}

impl Fix for GeoPoint {
    fn latitude(&self) -> f64 {
        self.latitude
    }

    fn longitude(&self) -> f64 {
        self.longitude
    }
}
