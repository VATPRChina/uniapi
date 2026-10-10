use crate::modules::navdata::models::{AnyFix, Fix};
use crate::utils::geo::{Latitude, Longitude};

#[derive(Clone, PartialEq)]
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

    /// Destination at a bearing in degrees and a distance in nautical miles.
    pub fn destination(&self, heading: u16, distance: u16) -> Self {
        if distance == 0 {
            return self.clone();
        }
        let latitude = self.latitude.to_radians();
        let longitude = self.longitude.to_radians();
        let bearing = f64::from(heading).to_radians();
        let angle = f64::from(distance) * 1852.0 / 6_371_008.8;
        let lat = (latitude.sin() * angle.cos() + latitude.cos() * angle.sin() * bearing.cos())
            .clamp(-1., 1.)
            .asin();
        let lon = longitude
            + (bearing.sin() * angle.sin() * latitude.cos())
                .atan2(angle.cos() - latitude.sin() * lat.sin());
        Self::new(
            lat.to_degrees(),
            (lon.to_degrees() + 180.).rem_euclid(360.) - 180.,
        )
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

impl std::fmt::Debug for GeoPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeoPoint")
            .field("latitude", &Latitude(self.latitude))
            .field("longitude", &Longitude(self.longitude))
            .finish()
    }
}

impl From<GeoPoint> for AnyFix {
    fn from(val: GeoPoint) -> Self {
        AnyFix::GeoPoint(val)
    }
}
