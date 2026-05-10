mod airport;
mod geo_point;
mod ndb;
mod vhf;
mod waypoint;

pub use airport::Airport;
pub use geo_point::GeoPoint;
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
