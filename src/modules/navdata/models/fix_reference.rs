use crate::modules::navdata::models::{Airport, AnyFix, Fix, GeoPoint, Ndb, Vhf, Waypoint};

#[derive(Debug, Clone, PartialEq)]
pub struct FixReference {
    fix: FixReferenceBase,
    heading: u16,
    distance: u16,
}

impl FixReference {
    pub fn new(fix: impl Into<FixReferenceBase>, heading: u16, distance: u16) -> Self {
        Self {
            fix: fix.into(),
            heading,
            distance,
        }
    }

    fn destination(&self) -> (f64, f64) {
        let fix: &dyn Fix = match &self.fix {
            FixReferenceBase::Airport(fix) => fix,
            FixReferenceBase::Ndb(fix) => fix,
            FixReferenceBase::Vhf(fix) => fix,
            FixReferenceBase::Waypoint(fix) => fix,
            FixReferenceBase::Unknown(_) => return (0., 0.),
        };
        let point =
            GeoPoint::new(fix.latitude(), fix.longitude()).destination(self.heading, self.distance);
        (point.latitude, point.longitude)
    }
}

impl Fix for FixReference {
    fn latitude(&self) -> f64 {
        self.destination().0
    }

    fn longitude(&self) -> f64 {
        self.destination().1
    }
}

impl From<FixReference> for AnyFix {
    fn from(value: FixReference) -> Self {
        AnyFix::FixReference(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FixReferenceBase {
    Airport(Airport),
    Ndb(Ndb),
    Vhf(Vhf),
    Waypoint(Waypoint),
    Unknown(String),
}

impl From<Airport> for FixReferenceBase {
    fn from(value: Airport) -> Self {
        FixReferenceBase::Airport(value)
    }
}

impl From<Ndb> for FixReferenceBase {
    fn from(value: Ndb) -> Self {
        FixReferenceBase::Ndb(value)
    }
}

impl From<Vhf> for FixReferenceBase {
    fn from(value: Vhf) -> Self {
        FixReferenceBase::Vhf(value)
    }
}

impl From<Waypoint> for FixReferenceBase {
    fn from(value: Waypoint) -> Self {
        FixReferenceBase::Waypoint(value)
    }
}
