use tracing::warn;

use super::ConstructedLeg;
use crate::modules::navdata::models::{AnyFix, LegKind, NavProc, ResolvedLeg};
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
    /// Fix endpoints use their equality implementations. Airways allow reverse
    /// traversal, flipping the direction restrictions. Published endpoint records
    /// are preserved, including their ICAO regions when equivalent fixes differ.
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
    if leg.is_unknown || leg.from.is_unknown() || leg.to.is_unknown() || leg.from == leg.to {
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
    Ok(find_path(&edges, &leg.from, &leg.to)
        .filter(|path| !path.is_empty())
        .unwrap_or_else(|| {
            if leg.kind == LegKind::Airway {
                warn!(
                    "failed to find a route on {:?} from {:?} to {:?}",
                    leg.identifier, leg.from, leg.to
                );
            }
            vec![leg.clone()]
        }))
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
fn find_path(edges: &[ResolvedLeg], from: &AnyFix, to: &AnyFix) -> Option<Vec<ResolvedLeg>> {
    let first = Search {
        frontier: vec![Path {
            point: from.clone(),
            legs: Vec::new(),
        }],
        visited: vec![from.clone()],
    };
    std::iter::successors(Some(first), |search| {
        if search.frontier.iter().any(|path| &path.point == to) {
            return None;
        }
        let frontier = search
            .frontier
            .iter()
            .flat_map(|path| {
                edges
                    .iter()
                    .filter(|edge| {
                        path.point == edge.from
                            && !search.visited.iter().any(|point| point == &edge.to)
                    })
                    .map(|edge| Path {
                        point: edge.to.clone(),
                        legs: path.legs.iter().cloned().chain([edge.clone()]).collect(),
                    })
            })
            .fold(Vec::<Path>::new(), |paths, next| {
                if paths.iter().any(|path| path.point == next.point) {
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
            .find(|path| &path.point == to)
            .map(|path| path.legs)
    })
}

#[cfg(test)]
mod tests {
    use super::find_path;
    use crate::modules::navdata::models::{
        AnyFix, DirectionRestriction, GeoPoint, LegKind, ResolvedLeg, Waypoint, WaypointKind,
    };

    #[test]
    fn path_search_matches_fix_identity_instead_of_identifier_or_metadata() {
        let from = Waypoint {
            icao_code: "ZG".try_into().unwrap(),
            identifier: "START".try_into().unwrap(),
            latitude: 30.0.try_into().unwrap(),
            longitude: 110.0.try_into().unwrap(),
            kind: WaypointKind::Enroute,
        };
        let to = AnyFix::GeoPoint(GeoPoint::new(31., 111.).unwrap());
        let edge = ResolvedLeg {
            from: AnyFix::Waypoint(from.clone()),
            to: to.clone(),
            identifier: Some("A1".to_owned()),
            is_unknown: false,
            kind: LegKind::Airway,
            direction_restriction: DirectionRestriction::Forward,
        };
        let edges = [edge.clone()];
        let same = AnyFix::Waypoint(Waypoint {
            icao_code: "RJ".try_into().unwrap(),
            ..from.clone()
        });
        assert_eq!(find_path(&edges, &same, &to), Some(vec![edge]));
        let different_position = AnyFix::Waypoint(Waypoint {
            longitude: 111.0.try_into().unwrap(),
            ..from
        });
        assert!(find_path(&edges, &different_position, &to).is_none());
        let different_variant = AnyFix::GeoPoint(GeoPoint::new(30., 110.).unwrap());
        assert!(find_path(&edges, &different_variant, &to).is_none());
    }
    #[test]
    fn expansion_preserves_published_tamot_region_and_mainland_cutoff() {
        use super::{ConstructedLeg, Expander};
        use crate::modules::flight::flight_plan::{RouteSegmenter, SegmentKind};
        use crate::modules::navdata::models::{Airway, ChinaMainlandBoundary, NavProc};

        let fix = |identifier: &str, region: &str, latitude: f64, longitude: f64| {
            AnyFix::Waypoint(Waypoint {
                icao_code: region.try_into().unwrap(),
                identifier: identifier.try_into().unwrap(),
                latitude: latitude.try_into().unwrap(),
                longitude: longitude.try_into().unwrap(),
                kind: WaypointKind::Enroute,
            })
        };
        let wyn = fix("WYN", "ZG", 24.348611, 114.113056);
        let nlg = fix("NLG", "ZG", 22.530556, 113.563056);
        let tamot_zg = fix("TAMOT", "ZG", 22.358333, 113.866667);
        let tamot_vh = fix("TAMOT", "VH", 22.358333, 113.866667);
        let alley = fix("ALLEY", "VH", 21.086431, 113.785972);
        let leg = |from, to| ResolvedLeg {
            from,
            to,
            identifier: Some("W18".into()),
            is_unknown: false,
            kind: LegKind::Airway,
            direction_restriction: DirectionRestriction::Forward,
        };
        let published = vec![leg(wyn.clone(), nlg.clone()), leg(nlg, tamot_zg)];
        let expanded = Expander::new(vec![
            ConstructedLeg {
                leg: leg(wyn, tamot_vh.clone()),
                procedure: Some(NavProc::Airway(Airway {
                    identifier: "W18".try_into().unwrap(),
                    legs: published,
                })),
            },
            ConstructedLeg {
                leg: ResolvedLeg {
                    identifier: None,
                    kind: LegKind::Direct,
                    ..leg(tamot_vh, alley)
                },
                procedure: None,
            },
        ])
        .expand()
        .unwrap();
        assert_eq!(expanded.len(), 3);
        assert_eq!(expanded[1].to.identifier(), Some("TAMOT"));
        assert_eq!(expanded[1].to.icao_code(), Some("ZG"));
        assert_eq!(expanded[2].from.icao_code(), Some("VH"));
        // Equivalent fix records still connect without discarding airway metadata.
        assert_eq!(expanded[1].to, expanded[2].from);
        let boundary = ChinaMainlandBoundary::default();
        let segments = RouteSegmenter::new(&boundary).segment(&expanded);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].kind, SegmentKind::Mainland);
        assert_eq!(segments[0].route.end_index(), 2);
        assert_eq!(
            segments[0].route.legs.last().unwrap().to.identifier(),
            Some("TAMOT")
        );
        assert_eq!(segments[1].kind, SegmentKind::International);
        assert_eq!(segments[1].route.start_index, 2);
    }
}
