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

use crate::modules::navdata::models::{ChinaMainlandBoundary, Identifiable};
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

    pub fn is_china_or_unknown(&self, boundary: &ChinaMainlandBoundary) -> bool {
        // Airports have no separate ICAO region code in navdata. Their airport
        // identifier still provides country classification without FIR geometry.
        let icao = match self {
            Self::Airport(airport) => Some(airport.identifier()),
            _ => self.icao_code(),
        };
        icao.is_some_and(|icao| {
            icao.starts_with("Z") && !icao.starts_with("ZM") && !icao.starts_with("ZK")
        }) || self
            .latitude()
            .and_then(|lat| self.longitude().map(|lon| (*lat, *lon)))
            .is_some_and(|coord| boundary.contains(coord))
            || self.is_unknown()
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
