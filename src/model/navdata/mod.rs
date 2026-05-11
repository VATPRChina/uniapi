mod airport;
mod geo_point;
mod leg;
mod ndb;
mod vhf;
mod waypoint;

pub use airport::Airport;
pub use geo_point::GeoPoint;
pub use leg::{DirectionRestriction, FixRef, Leg, ResolvedLeg};
pub use ndb::{Ndb, NdbKind};
pub use vhf::Vhf;
pub use waypoint::{Waypoint, WaypointKind};

pub trait Fix {
    fn latitude(&self) -> f64;
    fn longitude(&self) -> f64;
}

pub trait Identifiable {
    fn icao_code(&self) -> &str;
    fn identifier(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnyFix {
    Airport(Airport),
    GeoPoint(GeoPoint),
    Ndb(Ndb),
    Vhf(Vhf),
    Waypoint(Waypoint),
}

impl Fix for AnyFix {
    fn latitude(&self) -> f64 {
        match self {
            AnyFix::Airport(airport) => airport.latitude(),
            AnyFix::GeoPoint(geo_point) => geo_point.latitude(),
            AnyFix::Ndb(ndb) => ndb.latitude(),
            AnyFix::Vhf(vhf) => vhf.latitude(),
            AnyFix::Waypoint(waypoint) => waypoint.latitude(),
        }
    }

    fn longitude(&self) -> f64 {
        match self {
            AnyFix::Airport(airport) => airport.longitude(),
            AnyFix::GeoPoint(geo_point) => geo_point.longitude(),
            AnyFix::Ndb(ndb) => ndb.longitude(),
            AnyFix::Vhf(vhf) => vhf.longitude(),
            AnyFix::Waypoint(waypoint) => waypoint.longitude(),
        }
    }
}

impl AnyFix {
    pub fn icao_code(&self) -> Option<&str> {
        match self {
            AnyFix::Airport(airport) => Some(airport.icao_code()),
            AnyFix::GeoPoint(_) => None,
            AnyFix::Ndb(ndb) => Some(ndb.icao_code()),
            AnyFix::Vhf(vhf) => Some(vhf.icao_code()),
            AnyFix::Waypoint(waypoint) => Some(waypoint.icao_code()),
        }
    }

    pub fn identifier(&self) -> Option<&str> {
        match self {
            AnyFix::Airport(airport) => Some(airport.identifier()),
            AnyFix::GeoPoint(_) => None,
            AnyFix::Ndb(ndb) => Some(ndb.identifier()),
            AnyFix::Vhf(vhf) => Some(vhf.identifier()),
            AnyFix::Waypoint(waypoint) => Some(waypoint.identifier()),
        }
    }
}
