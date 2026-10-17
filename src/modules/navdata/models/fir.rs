//! Horizontal mainland FIR coverage loaded from ARINC 424 UF records.

use super::{Fix, ResolvedLeg};

#[derive(Debug, Clone)]
pub struct MainlandFirBoundaries {
    pub(crate) rings: Vec<Vec<[f64; 2]>>,
}

impl MainlandFirBoundaries {
    /// Display classification includes boundary-crossing legs when either fix
    /// lies in mainland FIRs. Missing coordinates do not imply membership.
    pub fn contains_either_endpoint(&self, leg: &ResolvedLeg) -> bool {
        [&leg.from, &leg.to].into_iter().any(|fix| {
            fix.position()
                .is_some_and(|position| self.contains(position))
        })
    }

    /// Positions use the same (latitude, longitude) order as `Fix::position`.
    pub fn contains(&self, (latitude, longitude): (f64, f64)) -> bool {
        self.rings
            .iter()
            .any(|ring| ring_location([longitude, latitude], ring) != Location::Outside)
    }

    #[cfg(test)]
    pub(crate) fn fixture() -> Self {
        Self {
            rings: vec![vec![[100., 20.], [120., 20.], [120., 40.], [100., 40.]]],
        }
    }
}

#[derive(PartialEq, Eq)]
enum Location {
    Outside,
    Inside,
    Boundary,
}

fn ring_location([x, y]: [f64; 2], ring: &[[f64; 2]]) -> Location {
    let mut inside = false;
    for (a, b) in ring
        .iter()
        .zip(ring.iter().cycle().skip(1))
        .take(ring.len())
    {
        let [ax, ay] = *a;
        let [bx, by] = *b;
        let dx = bx - ax;
        let dy = by - ay;
        // Coordinate equality elsewhere in navdata is at 0.01 arcsecond.
        let tolerance = 0.01 / 3600.;
        if ((x - ax) * dy - (y - ay) * dx).abs() <= tolerance * dx.hypot(dy)
            && x >= ax.min(bx) - tolerance
            && x <= ax.max(bx) + tolerance
            && y >= ay.min(by) - tolerance
            && y <= ay.max(by) + tolerance
        {
            return Location::Boundary;
        }
        if (ay > y) != (by > y) && x < ax + (y - ay) * dx / dy {
            inside = !inside;
        }
    }
    if inside {
        Location::Inside
    } else {
        Location::Outside
    }
}
