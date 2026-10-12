mod airport;
mod fix_reference;
mod geo_point;
mod ndb;
mod vhf;
mod waypoint;

pub use airport::Airport;
pub use fix_reference::FixReference;
pub use geo_point::GeoPoint;
pub use ndb::{Ndb, NdbKind};
pub use vhf::Vhf;
pub use waypoint::{Waypoint, WaypointKind};

use crate::modules::navdata::models::Identifiable;
use crate::utils::geo::{Latitude, Longitude};

pub trait Fix {
    fn latitude(&self) -> Option<Latitude>;
    fn longitude(&self) -> Option<Longitude>;

    fn valid_latitude_or(&self, default: f64) -> f64 {
        self.latitude().map_or(default, |latitude| *latitude)
    }

    fn valid_longitude_or(&self, default: f64) -> f64 {
        self.longitude().map_or(default, |longitude| *longitude)
    }

    fn position(&self) -> Option<(f64, f64)> {
        let latitude = self.latitude()?;
        let longitude = self.longitude()?;
        Some((*latitude, *longitude))
    }
}

#[derive(Debug, Clone)]
pub enum AnyFix {
    Airport(Airport),
    GeoPoint(GeoPoint),
    Ndb(Ndb),
    Vhf(Vhf),
    Waypoint(Waypoint),
    FixReference(FixReference),
    Unknown(String),
}

impl Fix for AnyFix {
    fn latitude(&self) -> Option<Latitude> {
        match self {
            AnyFix::Airport(airport) => airport.latitude(),
            AnyFix::GeoPoint(geo_point) => geo_point.latitude(),
            AnyFix::Ndb(ndb) => ndb.latitude(),
            AnyFix::Vhf(vhf) => vhf.latitude(),
            AnyFix::Waypoint(waypoint) => waypoint.latitude(),
            AnyFix::FixReference(fix_ref) => fix_ref.latitude(),
            AnyFix::Unknown(_) => None,
        }
    }

    fn longitude(&self) -> Option<Longitude> {
        match self {
            AnyFix::Airport(airport) => airport.longitude(),
            AnyFix::GeoPoint(geo_point) => geo_point.longitude(),
            AnyFix::Ndb(ndb) => ndb.longitude(),
            AnyFix::Vhf(vhf) => vhf.longitude(),
            AnyFix::Waypoint(waypoint) => waypoint.longitude(),
            AnyFix::FixReference(fix_ref) => fix_ref.longitude(),
            AnyFix::Unknown(_) => None,
        }
    }
}

impl AnyFix {
    #[allow(unused)]
    pub fn icao_code(&self) -> Option<&str> {
        match self {
            AnyFix::Airport(airport) => Some(airport.icao_code()),
            AnyFix::GeoPoint(_) => None,
            AnyFix::Ndb(ndb) => Some(ndb.icao_code()),
            AnyFix::Vhf(vhf) => Some(vhf.icao_code()),
            AnyFix::Waypoint(waypoint) => Some(waypoint.icao_code()),
            AnyFix::FixReference(_) => None,
            AnyFix::Unknown(_) => None,
        }
    }

    pub fn is_china(&self) -> bool {
        // TODO: geo point check within China
        self.icao_code().is_none_or(|icao| {
            icao.starts_with("Z") && !icao.starts_with("ZM") && !icao.starts_with("ZK")
        })
    }

    pub fn identifier(&self) -> Option<&str> {
        match self {
            AnyFix::Airport(airport) => Some(airport.identifier()),
            AnyFix::GeoPoint(_) => None,
            AnyFix::Ndb(ndb) => Some(ndb.identifier()),
            AnyFix::Vhf(vhf) => Some(vhf.identifier()),
            AnyFix::Waypoint(waypoint) => Some(waypoint.identifier()),
            AnyFix::FixReference(_) => None,
            AnyFix::Unknown(str) => Some(str.as_str()),
        }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, AnyFix::Unknown(_))
    }
}
#[cfg(test)]
mod tests {
    use super::{AnyFix, GeoPoint, Waypoint, WaypointKind};
    use crate::utils::geo::{Latitude, Longitude};

    #[test]
    fn unknown_fixes_have_no_coordinates() {
        use super::Fix;
        use super::fix_reference::{FixReference, FixReferenceBase};

        let unknown = AnyFix::Unknown("UNKNOWN".to_owned());
        let reference = AnyFix::FixReference(FixReference::new(
            FixReferenceBase::Unknown("UNKNOWN".to_owned()),
            90,
            10,
        ));
        for fix in [unknown, reference] {
            assert_eq!(fix.latitude(), None);
            assert_eq!(fix.longitude(), None);
            assert_eq!(fix.position(), None);
            assert_eq!(fix.valid_latitude_or(30.), 30.);
            assert_eq!(fix.valid_longitude_or(110.), 110.);
        }
    }

    #[test]
    fn fix_equality_uses_coordinate_precision() {
        let waypoint = Waypoint {
            icao_code: "ZG".try_into().unwrap(),
            identifier: "TEST".try_into().unwrap(),
            latitude: Latitude::new(30.).unwrap(),
            longitude: Longitude::new(110.).unwrap(),
            kind: WaypointKind::Enroute,
        };
        let mut same = waypoint.clone();
        same.latitude = Latitude::new(*same.latitude + 0.004 / 3600.).unwrap();
        same.longitude = Longitude::new(*same.longitude + 0.004 / 3600.).unwrap();
        assert_eq!(AnyFix::Waypoint(waypoint.clone()), AnyFix::Waypoint(same));
        let mut different = waypoint.clone();
        different.longitude = Longitude::new(*different.longitude + 0.006 / 3600.).unwrap();
        assert_ne!(AnyFix::Waypoint(waypoint), AnyFix::Waypoint(different));

        let point = AnyFix::GeoPoint(GeoPoint::new(30., 110.).unwrap());
        assert_eq!(
            point,
            AnyFix::GeoPoint(GeoPoint::new(30. + 0.004 / 3600., 110.).unwrap())
        );
        assert_ne!(
            point,
            AnyFix::GeoPoint(GeoPoint::new(30., 110. + 0.006 / 3600.).unwrap())
        );
    }
}
