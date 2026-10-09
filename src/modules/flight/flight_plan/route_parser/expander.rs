use tracing::warn;

use super::ConstructedLeg;
use crate::modules::navdata::models::{AnyFix, Fix, NavProc, ResolvedLeg};
use std::borrow::Cow;

#[derive(Debug, thiserror::Error)]
pub enum ExpansionError {
    #[error("invalid expansion coordinates on procedure data")]
    InvalidCoordinates,
}

pub struct Expander {
    route: Vec<ConstructedLeg>,
}

impl Expander {
    pub fn new(route: Vec<ConstructedLeg>) -> Self {
        Self { route }
    }

    /// Expand named legs into connected published segments, in route order.
    /// Direct legs and legs without a matching path retain their original form.
    /// Airways match named fixes by identifier and allow reverse traversal,
    /// flipping the direction restrictions.
    /// SID/STAR common legs retain their published direction.
    /// Procedures use fixed endpoints only; runway selection and vector geometry
    /// are not represented by the constructed route.
    pub fn expand(&self) -> Result<Vec<ResolvedLeg>, ExpansionError> {
        self.route
            .iter()
            .map(expand_leg)
            .collect::<Result<Vec<_>, _>>()
            .map(|legs| legs.into_iter().flatten().collect())
    }
}

fn expand_leg(constructed: &ConstructedLeg) -> Result<Vec<ResolvedLeg>, ExpansionError> {
    let leg = &constructed.leg;
    let Some(procedure) = &constructed.procedure else {
        return Ok(vec![leg.clone()]);
    };
    let fix_matches: fn(&AnyFix, &AnyFix) -> bool = match procedure {
        NavProc::Airway(_) => same_airway_fix,
        _ => same_fix,
    };
    if leg.is_unknown
        || leg.from.is_unknown()
        || leg.to.is_unknown()
        || fix_matches(&leg.from, &leg.to)
    {
        return Ok(vec![leg.clone()]);
    }
    let edges = procedure.legs();
    if edges
        .iter()
        .any(|edge| edge.from.is_unknown() || edge.to.is_unknown())
    {
        return Err(ExpansionError::InvalidCoordinates);
    }
    let edges: Cow<'_, [ResolvedLeg]> = match procedure {
        NavProc::Airway(_) => Cow::Owned(
            edges
                .iter()
                .cloned()
                .flat_map(|edge| [edge.clone(), edge.into_reversed()])
                .collect(),
        ),
        _ => Cow::Borrowed(edges),
    };
    Ok(find_path(&edges, &leg.from, &leg.to, fix_matches)
        .filter(|path| !path.is_empty())
        .map(|path| {
            path.iter()
                .enumerate()
                .map(|(index, segment)| ResolvedLeg {
                    from: if index == 0 {
                        leg.from.clone()
                    } else {
                        path[index - 1].to.clone()
                    },
                    to: if index + 1 == path.len() {
                        leg.to.clone()
                    } else {
                        segment.to.clone()
                    },
                    ..segment.clone()
                })
                .collect()
        })
        .unwrap_or_else(|| {
            warn!(
                "failed to find a route on {:?} from {:?} to {:?}",
                leg.identifier, leg.from, leg.to
            );
            vec![leg.clone()]
        }))
}

fn same_airway_fix(left: &AnyFix, right: &AnyFix) -> bool {
    match (left.identifier(), right.identifier()) {
        (Some(left), Some(right)) => left.eq_ignore_ascii_case(right),
        _ => same_fix(left, right),
    }
}

fn same_fix(left: &AnyFix, right: &AnyFix) -> bool {
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

#[derive(Clone)]
struct Path {
    point: AnyFix,
    legs: Vec<ResolvedLeg>,
}

struct Search {
    frontier: Vec<Path>,
    visited: Vec<AnyFix>,
}

/// Breadth-first search picks the fewest published segments, with navigation
/// record order breaking ties. Visited fixes use the same matching rule as edges.
fn find_path(
    edges: &[ResolvedLeg],
    from: &AnyFix,
    to: &AnyFix,
    fix_matches: fn(&AnyFix, &AnyFix) -> bool,
) -> Option<Vec<ResolvedLeg>> {
    let first = Search {
        frontier: vec![Path {
            point: from.clone(),
            legs: Vec::new(),
        }],
        visited: vec![from.clone()],
    };
    std::iter::successors(Some(first), |search| {
        if search
            .frontier
            .iter()
            .any(|path| fix_matches(&path.point, to))
        {
            return None;
        }
        let frontier = search
            .frontier
            .iter()
            .flat_map(|path| {
                edges
                    .iter()
                    .filter(|edge| {
                        fix_matches(&path.point, &edge.from)
                            && !search
                                .visited
                                .iter()
                                .any(|point| fix_matches(point, &edge.to))
                    })
                    .map(|edge| Path {
                        point: edge.to.clone(),
                        legs: path.legs.iter().cloned().chain([edge.clone()]).collect(),
                    })
            })
            .fold(Vec::<Path>::new(), |paths, next| {
                if paths
                    .iter()
                    .any(|path| fix_matches(&path.point, &next.point))
                {
                    paths
                } else {
                    paths.into_iter().chain([next]).collect()
                }
            });
        (!frontier.is_empty()).then(|| Search {
            visited: search
                .visited
                .iter()
                .cloned()
                .chain(frontier.iter().map(|path| path.point.clone()))
                .collect(),
            frontier,
        })
    })
    .find_map(|search| {
        search
            .frontier
            .into_iter()
            .find(|path| fix_matches(&path.point, to))
            .map(|path| path.legs)
    })
}
