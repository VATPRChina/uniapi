//! Horizontal mainland FIR coverage loaded from ARINC 424 UF records.

use geo::{
    BoundingRect, Centroid, Contains, InteriorPoint, LineString, Orient, Polygon, Rect,
    orient::Direction, unary_union,
};

// Coordinate equality elsewhere in navdata is at 0.01 arcsecond.
const BOUNDARY_TOLERANCE: f64 = 0.01 / 3600.;

#[derive(Debug, Clone)]
pub struct ChinaMainlandBoundary {
    /// One closed outer loop; internal FIR seam gaps are included in the airspace.
    boundary: Vec<(f64, f64)>,
    bbox: Rect<f64>,
    inner_bbox: Option<Rect<f64>>,
}

impl ChinaMainlandBoundary {
    pub fn from_rings(rings: Vec<Vec<(f64, f64)>>) -> Result<Self, &'static str> {
        let polygons: Vec<_> = rings
            .into_iter()
            .map(|ring| Polygon::new(LineString::from(ring), Vec::new()).orient(Direction::Default))
            .collect();
        let mut airspace = unary_union(&polygons).0;
        if airspace.len() != 1 {
            return Err("mainland FIRs must merge into one connected airspace");
        }
        let polygon = airspace.pop().unwrap();
        let bbox = polygon.bounding_rect().ok_or("empty mainland boundary")?;
        let (exterior, _) = polygon.into_inner();
        let inner_bbox = inner_bbox(&Polygon::new(exterior.clone(), Vec::new()), bbox);
        let boundary = exterior
            .0
            .into_iter()
            .map(|point| (point.x, point.y))
            .collect();
        Ok(Self {
            boundary,
            bbox,
            inner_bbox,
        })
    }

    /// Positions use the same (latitude, longitude) order as `Fix::position`.
    pub fn contains(&self, (latitude, longitude): (f64, f64)) -> bool {
        if self.inner_bbox.is_some_and(|bbox| {
            longitude >= bbox.min().x
                && longitude <= bbox.max().x
                && latitude >= bbox.min().y
                && latitude <= bbox.max().y
        }) {
            return true;
        }

        let bbox = self.bbox;
        if longitude < bbox.min().x - BOUNDARY_TOLERANCE
            || longitude > bbox.max().x + BOUNDARY_TOLERANCE
            || latitude < bbox.min().y - BOUNDARY_TOLERANCE
            || latitude > bbox.max().y + BOUNDARY_TOLERANCE
        {
            return false;
        }

        ring_location((longitude, latitude), &self.boundary) != Location::Outside
    }
}

/// Find a conservative inner rectangle once at load time, then grow each side.
/// Full polygon containment prevents the rectangle from crossing concave edges.
fn inner_bbox(polygon: &Polygon<f64>, outer: Rect<f64>) -> Option<Rect<f64>> {
    if polygon.contains(&outer) {
        return Some(outer);
    }
    let center = polygon
        .centroid()
        .filter(|point| polygon.contains(point))
        .or_else(|| polygon.interior_point())?;
    if !polygon.contains(&center) {
        return None;
    }
    let mut inner = grow_bbox(polygon, Rect::new(center.0, center.0), outer);
    for side in 0..4 {
        let min = (inner.min().x, inner.min().y);
        let max = (inner.max().x, inner.max().y);
        let target = match side {
            0 => Rect::new((outer.min().x, min.1), max),
            1 => Rect::new(min, (outer.max().x, max.1)),
            2 => Rect::new((min.0, outer.min().y), max),
            _ => Rect::new(min, (max.0, outer.max().y)),
        };
        inner = grow_bbox(polygon, inner, target);
    }
    Some(inner)
}

fn grow_bbox(polygon: &Polygon<f64>, initial: Rect<f64>, target: Rect<f64>) -> Rect<f64> {
    if polygon.contains(&target) {
        return target;
    }
    let mut inner = initial;
    let (mut low, mut high) = (0., 1.);
    // A fixed number of iterations bounds startup cost; every accepted box is contained.
    for _ in 0..24 {
        let fraction = (low + high) / 2.;
        let candidate = Rect::new(
            (
                initial.min().x + (target.min().x - initial.min().x) * fraction,
                initial.min().y + (target.min().y - initial.min().y) * fraction,
            ),
            (
                initial.max().x + (target.max().x - initial.max().x) * fraction,
                initial.max().y + (target.max().y - initial.max().y) * fraction,
            ),
        );
        if polygon.contains(&candidate) {
            inner = candidate;
            low = fraction;
        } else {
            high = fraction;
        }
    }
    inner
}

#[derive(PartialEq, Eq)]
enum Location {
    Outside,
    Inside,
    Boundary,
}

fn ring_location((x, y): (f64, f64), ring: &[(f64, f64)]) -> Location {
    let mut inside = false;
    for (a, b) in ring.iter().zip(ring.iter().skip(1)) {
        let (ax, ay) = *a;
        let (bx, by) = *b;
        let dx = bx - ax;
        let dy = by - ay;
        let tolerance = BOUNDARY_TOLERANCE;
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
