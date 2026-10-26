mod fir;
mod fix;
mod leg;
mod nav_proc;
mod preferred_route;

pub use fir::{ChinaMainlandBoundary, FirBoundary};
pub use fix::{
    Airport, AnyFix, Fix, FixReference, GeoPoint, Ndb, NdbKind, Vhf, Waypoint, WaypointKind,
};
pub use leg::{DirectionRestriction, LegKind, ResolvedLeg};
pub use nav_proc::{Airway, NavProc, Sid, Star, TerminalProcedure};
pub use preferred_route::{CruiseAltitudeType, PreferredRoute};

pub trait Identifiable {
    #[allow(unused)]
    fn icao_code(&self) -> &str;
    fn identifier(&self) -> &str;
}
