use crate::modules::navdata::models::{AnyFix, Fix};
use crate::utils::geo::{Latitude, Longitude, NonFiniteCoordinateError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoPoint {
    pub latitude: Latitude,
    pub longitude: Longitude,
}

impl GeoPoint {
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, NonFiniteCoordinateError> {
        Ok(Self {
            latitude: latitude.try_into()?,
            longitude: longitude.try_into()?,
        })
    }

    /// Destination at a bearing in degrees and a distance in nautical miles.
    pub fn destination(
        &self,
        heading: u16,
        distance: u16,
    ) -> Result<Self, NonFiniteCoordinateError> {
        if distance == 0 {
            return Ok(self.clone());
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
    fn latitude(&self) -> Option<Latitude> {
        Some(self.latitude)
    }

    fn longitude(&self) -> Option<Longitude> {
        Some(self.longitude)
    }
}

impl From<GeoPoint> for AnyFix {
    fn from(val: GeoPoint) -> Self {
        AnyFix::GeoPoint(val)
    }
}
