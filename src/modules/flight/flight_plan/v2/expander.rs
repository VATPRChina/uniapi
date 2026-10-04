use crate::modules::navdata::{
    models::{AnyFix, DirectionRestriction, Fix, ProcedureKind, ProcedureSegment, ResolvedLeg},
    service::{InvalidNavdataError, NavdataResult, NavdataService},
};
use futures::{StreamExt, TryStreamExt, stream};

pub struct Expander {
    route: Vec<ResolvedLeg>,
}

impl Expander {
    pub fn new(route: Vec<ResolvedLeg>) -> Self {
        Self { route }
    }

    /// Expand named legs into connected published segments, in route order.
    /// Direct legs and legs without a matching path retain their original form.
    /// Airway reversals retain direction restrictions for subsequent validation.
    /// Procedures use fixed endpoints only; runway selection and vector geometry
    /// are not represented by the constructed route.
    pub async fn expand(&self, navdata: &NavdataService) -> NavdataResult<Vec<ResolvedLeg>> {
        stream::iter(self.route.iter().cloned().map(Ok::<_, InvalidNavdataError>))
            .and_then(|leg| async move { expand_leg(navdata, &leg).await })
            .boxed()
            .try_concat()
            .await
    }
}

async fn expand_leg(
    navdata: &NavdataService,
    leg: &ResolvedLeg,
) -> NavdataResult<Vec<ResolvedLeg>> {
    let Some(identifier) = leg.identifier.as_deref() else {
        return Ok(vec![leg.clone()]);
    };
    if leg.is_unknown
        || !known_fix(&leg.from)
        || !known_fix(&leg.to)
        || same_fix(&leg.from, &leg.to)
    {
        return Ok(vec![leg.clone()]);
    }
    let procedure = match (&leg.from, &leg.to) {
        (AnyFix::Airport(airport), _) => {
            Some((ProcedureKind::Sid, airport.identifier.as_str(), &leg.from))
        }
        (_, AnyFix::Airport(airport)) => {
            Some((ProcedureKind::Star, airport.identifier.as_str(), &leg.to))
        }
        _ => None,
    };
    let edges = if let Some((kind, airport, point)) = procedure {
        let segments = navdata
            .list_procedure_segments(kind, airport, identifier)
            .await?;
        if segments.is_empty() {
            airway_edges(navdata, identifier).await?
        } else {
            procedure_edges(&segments, kind, point, identifier)
        }
    } else {
        airway_edges(navdata, identifier).await?
    };
    if edges
        .iter()
        .any(|edge| !known_fix(&edge.from) || !known_fix(&edge.to))
    {
        return Err(InvalidNavdataError::InternalError(
            "invalid expansion coordinates",
        ));
    }
    Ok(find_path(&edges, &leg.from, &leg.to)
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
        .unwrap_or_else(|| vec![leg.clone()]))
}

async fn airway_edges(
    navdata: &NavdataService,
    identifier: &str,
) -> NavdataResult<Vec<ResolvedLeg>> {
    Ok(navdata
        .list_airway_segments(identifier)
        .await?
        .into_iter()
        .flat_map(|segment| [segment.clone(), segment.into_reversed()])
        .collect())
}

fn procedure_edges(
    segments: &[ProcedureSegment],
    kind: ProcedureKind,
    airport: &AnyFix,
    identifier: &str,
) -> Vec<ResolvedLeg> {
    let runway = |segment: &&ProcedureSegment| match kind {
        ProcedureKind::Sid => matches!(segment.route_type.as_str(), "1" | "4"),
        ProcedureKind::Star => matches!(segment.route_type.as_str(), "3" | "6"),
    };
    let has_runway = segments
        .iter()
        .filter(runway)
        .any(|segment| !segment.fixes.is_empty());
    let connectors =
        segments
            .iter()
            .filter(|segment| {
                if has_runway {
                    runway(segment)
                } else {
                    matches!(segment.route_type.as_str(), "2" | "5")
                }
            })
            .filter_map(|segment| match kind {
                ProcedureKind::Sid => segment.fixes.first().map(|fix| {
                    named_segment(airport.clone(), fix.clone(), identifier, true, false)
                }),
                ProcedureKind::Star => segment.fixes.last().map(|fix| {
                    named_segment(fix.clone(), airport.clone(), identifier, false, true)
                }),
            });
    segments
        .iter()
        .flat_map(|segment| {
            segment.fixes.windows(2).map(|pair| {
                named_segment(
                    pair[0].clone(),
                    pair[1].clone(),
                    identifier,
                    kind == ProcedureKind::Sid,
                    kind == ProcedureKind::Star,
                )
            })
        })
        .chain(connectors)
        .collect()
}

fn named_segment(
    from: AnyFix,
    to: AnyFix,
    identifier: &str,
    is_sid: bool,
    is_star: bool,
) -> ResolvedLeg {
    ResolvedLeg {
        from,
        to,
        identifier: Some(identifier.to_owned()),
        is_unknown: false,
        is_sid,
        is_star,
        direction_restriction: DirectionRestriction::None,
    }
}

fn known_fix(fix: &AnyFix) -> bool {
    !matches!(fix, AnyFix::Unknown(_))
        && fix.latitude().is_finite()
        && fix.longitude().is_finite()
        && (-90. ..=90.).contains(&fix.latitude())
        && (-180. ..=180.).contains(&fix.longitude())
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
/// record order breaking ties. Visited physical fixes prevent loops.
fn find_path(edges: &[ResolvedLeg], from: &AnyFix, to: &AnyFix) -> Option<Vec<ResolvedLeg>> {
    let first = Search {
        frontier: vec![Path {
            point: from.clone(),
            legs: Vec::new(),
        }],
        visited: vec![from.clone()],
    };
    std::iter::successors(Some(first), |search| {
        if search.frontier.iter().any(|path| same_fix(&path.point, to)) {
            return None;
        }
        let frontier = search
            .frontier
            .iter()
            .flat_map(|path| {
                edges
                    .iter()
                    .filter(|edge| {
                        same_fix(&path.point, &edge.from)
                            && !search.visited.iter().any(|point| same_fix(point, &edge.to))
                    })
                    .map(|edge| Path {
                        point: edge.to.clone(),
                        legs: path.legs.iter().cloned().chain([edge.clone()]).collect(),
                    })
            })
            .fold(Vec::<Path>::new(), |paths, next| {
                if paths.iter().any(|path| same_fix(&path.point, &next.point)) {
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
            .find(|path| same_fix(&path.point, to))
            .map(|path| path.legs)
    })
}
