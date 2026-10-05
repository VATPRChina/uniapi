use sqlx::FromRow;

use super::{InvalidNavdataError, NavdataResult};
use crate::modules::navdata::models::{Ndb, NdbKind, Vhf, Waypoint, WaypointKind};

#[derive(FromRow)]
pub(super) struct FixRecord {
    pub icao_code: String,
    pub identifier: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl FixRecord {
    pub fn position(&self) -> NavdataResult<(f64, f64)> {
        Ok((
            self.latitude
                .ok_or(InvalidNavdataError::InvalidNavaidNullLatLong)?,
            self.longitude
                .ok_or(InvalidNavdataError::InvalidNavaidNullLatLong)?,
        ))
    }

    pub fn into_waypoint(self, kind: WaypointKind) -> NavdataResult<Waypoint> {
        let (latitude, longitude) = self.position()?;
        Ok(Waypoint {
            icao_code: self.icao_code.as_str().try_into()?,
            identifier: self.identifier.as_str().try_into()?,
            latitude,
            longitude,
            kind,
        })
    }

    pub fn into_ndb(self, kind: NdbKind) -> NavdataResult<Ndb> {
        let (latitude, longitude) = self.position()?;
        Ok(Ndb {
            icao_code: self.icao_code.as_str().try_into()?,
            identifier: self.identifier.as_str().try_into()?,
            latitude,
            longitude,
            kind,
        })
    }

    pub fn into_vhf(self) -> NavdataResult<Vhf> {
        let (latitude, longitude) = self.position()?;
        Ok(Vhf {
            icao_code: self.icao_code.as_str().try_into()?,
            identifier: self.identifier.as_str().try_into()?,
            latitude,
            longitude,
        })
    }
}
