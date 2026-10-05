mod fix;
mod leg;
mod nav_proc;
mod preferred_route;

pub use fix::{
    Airport, AnyFix, Fix, FixReference, GeoPoint, Ndb, NdbKind, Vhf, Waypoint, WaypointKind,
};
pub use leg::{DirectionRestriction, LegKind, ResolvedLeg};
pub use nav_proc::{Airway, ProcedureKind, ProcedureSegment};
pub use preferred_route::{LevelRestrictionType, PreferredRoute};

pub trait Identifiable {
    #[allow(unused)]
    fn icao_code(&self) -> &str;
    fn identifier(&self) -> &str;
}
