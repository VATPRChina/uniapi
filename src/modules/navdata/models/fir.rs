//! Horizontal mainland FIR coverage loaded from ARINC 424 UF records.

#[derive(Debug, Clone)]
pub struct MainlandFirBoundaries {
    rings: Vec<Vec<(f64, f64)>>,
}

impl MainlandFirBoundaries {
    pub fn from_rings(rings: Vec<Vec<(f64, f64)>>) -> Self {
        Self { rings }
    }

    /// Positions use the same (latitude, longitude) order as `Fix::position`.
    pub fn contains(&self, (latitude, longitude): (f64, f64)) -> bool {
        self.rings
            .iter()
            .any(|ring| ring_location((longitude, latitude), ring) != Location::Outside)
    }
}

#[derive(PartialEq, Eq)]
enum Location {
    Outside,
    Inside,
    Boundary,
}

fn ring_location((x, y): (f64, f64), ring: &[(f64, f64)]) -> Location {
    let mut inside = false;
    for (a, b) in ring
        .iter()
        .zip(ring.iter().cycle().skip(1))
        .take(ring.len())
    {
        let (ax, ay) = *a;
        let (bx, by) = *b;
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
