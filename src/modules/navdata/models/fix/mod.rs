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

#[derive(Debug, Clone, PartialEq, Eq)]
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

    fn airport(identifier: &str, latitude: f64, longitude: f64) -> super::Airport {
        super::Airport {
            identifier: identifier.try_into().unwrap(),
            latitude: Latitude::new(latitude).unwrap(),
            longitude: Longitude::new(longitude).unwrap(),
        }
    }

    #[test]
    fn airports_compare_only_identifiers() {
        let original = airport("ZBAA", 40., 116.);
        let moved = airport("ZBAA", 30., 110.);
        assert_eq!(original, moved);
        assert_eq!(AnyFix::Airport(original.clone()), AnyFix::Airport(moved));
        assert_ne!(original, airport("ZSPD", 40., 116.));
    }

    #[test]
    fn named_fixes_ignore_icao_but_require_identifier_position_and_kind() {
        use super::{Ndb, NdbKind, Vhf};

        let ndb = Ndb {
            icao_code: "ZG".try_into().unwrap(),
            identifier: "TEST".try_into().unwrap(),
            latitude: Latitude::new(30.).unwrap(),
            longitude: Longitude::new(110.).unwrap(),
            kind: NdbKind::Enroute,
        };
        let vhf = Vhf {
            icao_code: ndb.icao_code,
            identifier: ndb.identifier,
            latitude: ndb.latitude,
            longitude: ndb.longitude,
        };
        let waypoint = Waypoint {
            icao_code: ndb.icao_code,
            identifier: "TEST".try_into().unwrap(),
            latitude: ndb.latitude,
            longitude: ndb.longitude,
            kind: WaypointKind::Enroute,
        };
        let cases = [
            (
                AnyFix::Ndb(ndb.clone()),
                AnyFix::Ndb(Ndb {
                    icao_code: "RJ".try_into().unwrap(),
                    ..ndb
                }),
            ),
            (
                AnyFix::Vhf(vhf.clone()),
                AnyFix::Vhf(Vhf {
                    icao_code: "RJ".try_into().unwrap(),
                    ..vhf
                }),
            ),
            (
                AnyFix::Waypoint(waypoint.clone()),
                AnyFix::Waypoint(Waypoint {
                    icao_code: "RJ".try_into().unwrap(),
                    ..waypoint
                }),
            ),
        ];
        for (original, same) in cases {
            assert_eq!(original, same);
            assert_eq!(same, original);
            let mut different = original.clone();
            match &mut different {
                AnyFix::Ndb(fix) => fix.identifier = "NEXT".try_into().unwrap(),
                AnyFix::Vhf(fix) => fix.identifier = "NEXT".try_into().unwrap(),
                AnyFix::Waypoint(fix) => fix.identifier = "NEXT".try_into().unwrap(),
                _ => unreachable!(),
            }
            assert_ne!(original, different);
            let mut different_kind = original.clone();
            match &mut different_kind {
                AnyFix::Ndb(fix) => fix.kind = NdbKind::Terminal,
                AnyFix::Waypoint(fix) => fix.kind = WaypointKind::Terminal,
                _ => {}
            }
            if !matches!(original, AnyFix::Vhf(_)) {
                assert_ne!(original, different_kind);
            }
            for change_latitude in [true, false] {
                let mut different = original.clone();
                let (latitude, longitude) = match &mut different {
                    AnyFix::Ndb(fix) => (&mut fix.latitude, &mut fix.longitude),
                    AnyFix::Vhf(fix) => (&mut fix.latitude, &mut fix.longitude),
                    AnyFix::Waypoint(fix) => (&mut fix.latitude, &mut fix.longitude),
                    _ => unreachable!(),
                };
                if change_latitude {
                    *latitude = Latitude::new(31.).unwrap();
                } else {
                    *longitude = Longitude::new(111.).unwrap();
                }
                assert_ne!(original, different);
            }
        }
    }

    #[test]
    fn references_compare_base_fix_heading_and_distance() {
        use super::fix_reference::{FixReference, FixReferenceBase};

        let original = FixReference::new(airport("ZBAA", 40., 116.), 90, 10);
        let moved = FixReference::new(airport("ZBAA", 30., 110.), 90, 10);
        assert_eq!(original, moved);
        assert_eq!(
            AnyFix::FixReference(original.clone()),
            AnyFix::FixReference(moved)
        );
        assert_ne!(
            original,
            FixReference::new(airport("ZSPD", 40., 116.), 90, 10)
        );
        assert_ne!(
            original,
            FixReference::new(airport("ZBAA", 40., 116.), 91, 10)
        );
        assert_ne!(
            original,
            FixReference::new(airport("ZBAA", 40., 116.), 90, 11)
        );
        assert_ne!(
            FixReferenceBase::Airport(airport("ZBAA", 40., 116.)),
            FixReferenceBase::Unknown("ZBAA".to_owned())
        );
    }

    #[test]
    fn unknowns_compare_identifiers_and_variants_must_match() {
        let unknown = AnyFix::Unknown("ZBAA".to_owned());
        assert_eq!(unknown, AnyFix::Unknown("ZBAA".to_owned()));
        assert_ne!(unknown, AnyFix::Unknown("ZSPD".to_owned()));
        assert_ne!(unknown, AnyFix::Airport(airport("ZBAA", 40., 116.)));
    }

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
