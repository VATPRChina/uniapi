use crate::modules::navdata::models::{Airport, AnyFix, Fix, Ndb, Vhf, Waypoint};

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
        let latitude = fix.latitude().to_radians();
        let longitude = fix.longitude().to_radians();
        let bearing = f64::from(self.heading).to_radians();
        // Distance is in nautical miles; use the mean Earth radius in metres.
        let angle = f64::from(self.distance) * 1852.0 / 6_371_008.8;
        let lat = (latitude.sin() * angle.cos() + latitude.cos() * angle.sin() * bearing.cos())
            .clamp(-1., 1.)
            .asin();
        let lon = longitude
            + (bearing.sin() * angle.sin() * latitude.cos())
                .atan2(angle.cos() - latitude.sin() * lat.sin());
        (
            lat.to_degrees(),
            (lon.to_degrees() + 180.).rem_euclid(360.) - 180.,
        )
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
