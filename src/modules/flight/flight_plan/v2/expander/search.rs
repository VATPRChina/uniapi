//! Search borrowed published legs and materialize only the selected path.

use crate::modules::navdata::models::{AnyFix, Fix, ResolvedLeg};

pub(super) fn same_fix(left: &AnyFix, right: &AnyFix) -> bool {
    let family = matches!(
        (left, right),
        (AnyFix::Airport(_), AnyFix::Airport(_))
            | (AnyFix::Waypoint(_), AnyFix::Waypoint(_))
            | (AnyFix::Vhf(_), AnyFix::Vhf(_))
            | (AnyFix::Ndb(_), AnyFix::Ndb(_))
            | (AnyFix::GeoPoint(_), AnyFix::GeoPoint(_))
            | (AnyFix::FixReference(_), AnyFix::FixReference(_))
    );
    family
        && left.identifier() == right.identifier()
        && !left
            .icao_code()
            .zip(right.icao_code())
            .is_some_and(|(left, right)| !left.is_empty() && !right.is_empty() && left != right)
        && (left.latitude() - right.latitude()).abs() <= 1. / 3600.
        && (left.longitude() - right.longitude()).abs() <= 1. / 3600.
}

/// An oriented view of a published segment; reversing does not copy its data.
#[derive(Clone, Copy)]
pub(super) struct Traversal<'a> {
    leg: &'a ResolvedLeg,
    reversed: bool,
}

impl<'a> Traversal<'a> {
    fn from(self) -> &'a AnyFix {
        if self.reversed {
            &self.leg.to
        } else {
            &self.leg.from
        }
    }

    pub(super) fn to(self) -> &'a AnyFix {
        if self.reversed {
            &self.leg.from
        } else {
            &self.leg.to
        }
    }

    /// Materialize only segments on the selected path for the owned API result.
    pub(super) fn resolve(self, from: AnyFix, to: AnyFix) -> ResolvedLeg {
        ResolvedLeg {
            from,
            to,
            identifier: self.leg.identifier.clone(),
            is_unknown: self.leg.is_unknown,
            is_sid: self.leg.is_sid,
            is_star: self.leg.is_star,
            direction_restriction: if self.reversed {
                self.leg.direction_restriction.reversed()
            } else {
                self.leg.direction_restriction.clone()
            },
        }
    }
}

fn traversals(edges: &[ResolvedLeg], bidirectional: bool) -> impl Iterator<Item = Traversal<'_>> {
    edges.iter().flat_map(move |leg| {
        [
            Some(Traversal {
                leg,
                reversed: false,
            }),
            bidirectional.then_some(Traversal {
                leg,
                reversed: true,
            }),
        ]
        .into_iter()
        .flatten()
    })
}

struct Path<'a> {
    point: &'a AnyFix,
    legs: Vec<Traversal<'a>>,
}

struct Search<'a> {
    frontier: Vec<Path<'a>>,
    visited: Vec<&'a AnyFix>,
}

/// Breadth-first search picks the fewest published segments, with navigation
/// record order breaking ties. Visited physical fixes prevent loops.
pub(super) fn find_path<'a>(
    edges: &'a [ResolvedLeg],
    bidirectional: bool,
    from: &'a AnyFix,
    to: &AnyFix,
) -> Option<Vec<Traversal<'a>>> {
    let first = Search {
        frontier: vec![Path {
            point: from,
            legs: Vec::new(),
        }],
        visited: vec![from],
    };
    std::iter::successors(Some(first), |search| {
        if search.frontier.iter().any(|path| same_fix(path.point, to)) {
            return None;
        }
        let frontier = search
            .frontier
            .iter()
            .flat_map(|path| {
                traversals(edges, bidirectional)
                    .filter(|edge| {
                        same_fix(path.point, edge.from())
                            && !search
                                .visited
                                .iter()
                                .any(|point| same_fix(point, edge.to()))
                    })
                    .map(|edge| Path {
                        point: edge.to(),
                        legs: path.legs.iter().copied().chain([edge]).collect(),
                    })
            })
            .fold(Vec::<Path<'_>>::new(), |paths, next| {
                if paths.iter().any(|path| same_fix(path.point, next.point)) {
                    paths
                } else {
                    paths.into_iter().chain([next]).collect()
                }
            });
        (!frontier.is_empty()).then(|| Search {
            visited: search
                .visited
                .iter()
                .copied()
                .chain(frontier.iter().map(|path| path.point))
                .collect(),
            frontier,
        })
    })
    .find_map(|search| {
        search
            .frontier
            .into_iter()
            .find(|path| same_fix(path.point, to))
            .map(|path| path.legs)
    })
}
