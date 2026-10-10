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

pub trait Fix {
    fn latitude(&self) -> f64;
    fn longitude(&self) -> f64;

    fn valid_latitude_or(&self, default: f64) -> f64 {
        if self.latitude().is_finite() {
            self.latitude()
        } else {
            default
        }
    }

    fn valid_longitude_or(&self, default: f64) -> f64 {
        if self.longitude().is_finite() {
            self.longitude()
        } else {
            default
        }
    }

    fn position(&self) -> Option<(f64, f64)> {
        if !self.latitude().is_finite() {
            return None;
        }
        if !self.longitude().is_finite() {
            return None;
        }
        Some((self.latitude(), self.longitude()))
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    fn latitude(&self) -> f64 {
        match self {
            AnyFix::Airport(airport) => airport.latitude(),
            AnyFix::GeoPoint(geo_point) => geo_point.latitude(),
            AnyFix::Ndb(ndb) => ndb.latitude(),
            AnyFix::Vhf(vhf) => vhf.latitude(),
            AnyFix::Waypoint(waypoint) => waypoint.latitude(),
            AnyFix::FixReference(fix_ref) => fix_ref.latitude(),
            AnyFix::Unknown(_) => f64::NAN,
        }
    }

    fn longitude(&self) -> f64 {
        match self {
            AnyFix::Airport(airport) => airport.longitude(),
            AnyFix::GeoPoint(geo_point) => geo_point.longitude(),
            AnyFix::Ndb(ndb) => ndb.longitude(),
            AnyFix::Vhf(vhf) => vhf.longitude(),
            AnyFix::Waypoint(waypoint) => waypoint.longitude(),
            AnyFix::FixReference(fix_ref) => fix_ref.longitude(),
            AnyFix::Unknown(_) => f64::NAN,
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
