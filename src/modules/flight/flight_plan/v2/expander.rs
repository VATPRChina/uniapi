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
    if !known_fix(&leg.from) || !known_fix(&leg.to) || same_fix(&leg.from, &leg.to) {
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
    let connectors = segments
        .iter()
        .filter(|segment| {
            if has_runway {
                runway(segment)
            } else {
                matches!(segment.route_type.as_str(), "2" | "5")
            }
        })
        .filter_map(|segment| match kind {
            ProcedureKind::Sid => segment
                .fixes
                .first()
                .map(|fix| named_segment(airport.clone(), fix.clone(), identifier)),
            ProcedureKind::Star => segment
                .fixes
                .last()
                .map(|fix| named_segment(fix.clone(), airport.clone(), identifier)),
        });
    segments
        .iter()
        .flat_map(|segment| {
            segment
                .fixes
                .windows(2)
                .map(|pair| named_segment(pair[0].clone(), pair[1].clone(), identifier))
        })
        .chain(connectors)
        .collect()
}

fn named_segment(from: AnyFix, to: AnyFix, identifier: &str) -> ResolvedLeg {
    ResolvedLeg {
        from,
        to,
        identifier: Some(identifier.to_owned()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::navdata::models::{Airport, Waypoint, WaypointKind};

    fn fix(identifier: &str, lon: f64) -> AnyFix {
        AnyFix::Waypoint(Waypoint {
            identifier: identifier.try_into().unwrap(),
            icao_code: "ZG".try_into().unwrap(),
            latitude: 0.,
            longitude: lon,
            kind: WaypointKind::Enroute,
        })
    }

    fn airport(identifier: &str) -> AnyFix {
        AnyFix::Airport(Airport {
            identifier: identifier.try_into().unwrap(),
            latitude: 0.,
            longitude: 0.,
        })
    }

    fn identifiers(legs: &[ResolvedLeg]) -> Vec<&str> {
        legs.first()
            .into_iter()
            .filter_map(|leg| leg.from.identifier())
            .chain(legs.iter().filter_map(|leg| leg.to.identifier()))
            .collect()
    }

    async fn navdata() -> NavdataService {
        let navdata =
            NavdataService::with_preferred_routes_path(":memory:", "assets/test/routes.csv")
                .await
                .unwrap();
        sqlx::raw_sql(
            r#"
            CREATE TABLE tbl_er_enroute_airways (
                area_code TEXT, direction_restriction TEXT, icao_code TEXT,
                route_identifier TEXT, seqno INTEGER, waypoint_description_code TEXT,
                waypoint_identifier TEXT, waypoint_latitude REAL, waypoint_longitude REAL,
                waypoint_ref_table TEXT
            );
            INSERT INTO tbl_er_enroute_airways VALUES
                ('A', '', 'ZG', 'A1', 1, '', 'AAA', 0, 0, 'EA'),
                ('A', 'F', 'ZG', 'A1', 2, '', 'BBB', 0, 1, 'EA'),
                ('A', 'B', 'ZG', 'A1', 3, '', 'CCC', 0, 2, 'EA'),
                ('A', '', 'ZG', 'AREA', 1, '', 'AAA', 0, 0, 'EA'),
                ('B', '', 'ZG', 'AREA', 2, '', 'CCC', 0, 2, 'EA'),
                ('A', '', 'ZG', 'END', 1, '', 'AAA', 0, 0, 'EA'),
                ('A', '', 'ZG', 'END', 2, 'EE', 'BBB', 0, 1, 'EA'),
                ('A', '', 'ZG', 'END', 3, '', 'CCC', 0, 2, 'EA'),
                ('A', '', 'ZY', 'FAR', 1, '', 'AAA', 50, 0, 'EA'),
                ('A', '', 'ZY', 'FAR', 2, '', 'CCC', 50, 2, 'EA'),
                ('A', '', 'ZG', 'BAD', 1, '', 'AAA', 0, 0, 'EA'),
                ('A', '', 'ZG', 'BAD', 2, '', 'CCC', 0, 2, 'ZZ');
            CREATE TABLE tbl_pd_sids (
                airport_identifier TEXT, procedure_identifier TEXT, route_type TEXT,
                transition_identifier TEXT, seqno INTEGER, waypoint_identifier TEXT,
                waypoint_icao_code TEXT, waypoint_ref_table TEXT,
                waypoint_latitude REAL, waypoint_longitude REAL
            );
            CREATE TABLE tbl_pe_stars AS SELECT * FROM tbl_pd_sids;
            INSERT INTO tbl_pd_sids VALUES
                ('ADEP', 'SID1', '4', 'RW01', 10, 'RWPT', 'ZG', 'PC', 0, 0.1),
                ('ADEP', 'SID1', '4', 'RW01', 20, 'JUNC', 'ZG', 'PC', 0, 0.2),
                ('ADEP', 'SID1', '5', 'ALL', 30, 'JUNC', 'ZG', 'PC', 0, 0.2),
                ('ADEP', 'SID1', '5', 'ALL', 40, 'ENDPT', 'ZG', 'PC', 0, 0.3),
                ('ADEP', 'SID1', '6', 'EXIT', 50, 'ENDPT', 'ZG', 'PC', 0, 0.3),
                ('ADEP', 'SID1', '6', 'EXIT', 60, 'EXIT', 'ZG', 'EA', 0, 0.4),
                ('ADEP', 'SID1', '6', 'OTHER', 50, 'ENDPT', 'ZG', 'PC', 0, 0.3),
                ('ADEP', 'SID1', '6', 'OTHER', 60, 'OTHER', 'ZG', 'EA', 0, 5);
            INSERT INTO tbl_pe_stars VALUES
                ('AARR', 'STAR1', '4', 'EXIT', 10, 'EXIT', 'ZG', 'EA', 0, 0.4),
                ('AARR', 'STAR1', '4', 'EXIT', 20, 'ENDPT', 'ZG', 'PC', 0, 0.3),
                ('AARR', 'STAR1', '5', 'ALL', 30, 'ENDPT', 'ZG', 'PC', 0, 0.3),
                ('AARR', 'STAR1', '5', 'ALL', 40, 'JUNC', 'ZG', 'PC', 0, 0.2),
                ('AARR', 'STAR1', '6', 'RW01', 50, 'JUNC', 'ZG', 'PC', 0, 0.2),
                ('AARR', 'STAR1', '6', 'RW01', 60, 'RWPT', 'ZG', 'PC', 0, 0.1);
        "#,
        )
        .execute(&navdata.db)
        .await
        .unwrap();
        navdata
    }

    #[tokio::test]
    async fn expands_airways_and_keeps_route_order_endpoints_and_restrictions() {
        let navdata = navdata().await;
        let airway = named_segment(fix("AAA", 0.), fix("CCC", 2.), "A1");
        let direct = ResolvedLeg {
            from: airway.to.clone(),
            to: fix("DDD", 3.),
            identifier: None,
            direction_restriction: DirectionRestriction::None,
        };
        let expanded = Expander::new(vec![airway.clone(), direct.clone()])
            .expand(&navdata)
            .await
            .unwrap();
        assert_eq!(identifiers(&expanded), ["AAA", "BBB", "CCC", "DDD"]);
        assert_eq!(expanded[0].from, airway.from);
        assert_eq!(expanded[1].to, airway.to);
        assert_eq!(
            expanded[0].direction_restriction,
            DirectionRestriction::Forward
        );
        assert_eq!(
            expanded[1].direction_restriction,
            DirectionRestriction::Backward
        );
        assert_eq!(expanded[2], direct);
        assert!(expanded.windows(2).all(|pair| pair[0].to == pair[1].from));
    }

    #[tokio::test]
    async fn reverses_airway_segments_and_direction_restrictions() {
        let navdata = navdata().await;
        let expanded = Expander::new(vec![named_segment(fix("CCC", 2.), fix("AAA", 0.), "A1")])
            .expand(&navdata)
            .await
            .unwrap();
        assert_eq!(identifiers(&expanded), ["CCC", "BBB", "AAA"]);
        assert_eq!(
            expanded[0].direction_restriction,
            DirectionRestriction::Forward
        );
        assert_eq!(
            expanded[1].direction_restriction,
            DirectionRestriction::Backward
        );
    }

    #[tokio::test]
    async fn retains_missing_disconnected_and_ambiguous_named_legs() {
        let navdata = navdata().await;
        let route: Vec<_> = ["MISSING", "AREA", "END", "FAR"]
            .into_iter()
            .map(|identifier| named_segment(fix("AAA", 0.), fix("CCC", 2.), identifier))
            .collect();
        assert_eq!(
            Expander::new(route.clone()).expand(&navdata).await.unwrap(),
            route
        );
    }

    #[tokio::test]
    async fn expands_sid_and_star_through_connected_transitions() {
        let navdata = navdata().await;
        let departure = named_segment(airport("ADEP"), fix("EXIT", 0.4), "SID1");
        let arrival = named_segment(fix("EXIT", 0.4), airport("AARR"), "STAR1");
        let expanded = Expander::new(vec![departure.clone(), arrival.clone()])
            .expand(&navdata)
            .await
            .unwrap();
        assert_eq!(
            identifiers(&expanded),
            [
                "ADEP", "RWPT", "JUNC", "ENDPT", "EXIT", "ENDPT", "JUNC", "RWPT", "AARR"
            ]
        );
        assert_eq!(expanded.first().unwrap().from, departure.from);
        assert_eq!(expanded.last().unwrap().to, arrival.to);
        assert!(
            expanded[..4]
                .iter()
                .all(|leg| leg.identifier.as_deref() == Some("SID1"))
        );
        assert!(
            expanded[4..]
                .iter()
                .all(|leg| leg.identifier.as_deref() == Some("STAR1"))
        );
        assert!(expanded.windows(2).all(|pair| pair[0].to == pair[1].from));
    }

    #[tokio::test]
    async fn direct_unknown_and_empty_routes_do_not_query_navdata() {
        let navdata = navdata().await;
        navdata.db.close().await;
        let route = vec![
            ResolvedLeg {
                from: fix("AAA", 0.),
                to: fix("BBB", 1.),
                identifier: None,
                direction_restriction: DirectionRestriction::None,
            },
            named_segment(AnyFix::Unknown("NOFIX".to_owned()), fix("CCC", 2.), "A1"),
        ];
        assert_eq!(
            Expander::new(route.clone()).expand(&navdata).await.unwrap(),
            route
        );
        assert!(
            Expander::new(Vec::new())
                .expand(&navdata)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(matches!(
            Expander::new(vec![named_segment(fix("AAA", 0.), fix("CCC", 2.), "A1")])
                .expand(&navdata)
                .await,
            Err(InvalidNavdataError::DatabaseError(_))
        ));
    }

    #[tokio::test]
    async fn invalid_airway_records_return_errors_instead_of_panicking() {
        let navdata = navdata().await;
        let result = Expander::new(vec![named_segment(fix("AAA", 0.), fix("CCC", 2.), "BAD")])
            .expand(&navdata)
            .await;
        assert!(matches!(result, Err(InvalidNavdataError::InternalError(_))));
    }

    #[test]
    fn search_handles_cycles_and_chooses_a_connected_shortest_segment_path() {
        let edges = vec![
            named_segment(fix("AAA", 0.), fix("BBB", 1.), "A1"),
            named_segment(fix("BBB", 1.), fix("AAA", 0.), "A1"),
            named_segment(fix("AAA", 0.), fix("CCC", 2.), "A1"),
            named_segment(fix("CCC", 2.), fix("DDD", 3.), "A1"),
            named_segment(fix("BBB", 1.), fix("CCC", 2.), "A1"),
        ];
        assert_eq!(
            identifiers(&find_path(&edges, &fix("AAA", 0.), &fix("DDD", 3.)).unwrap()),
            ["AAA", "CCC", "DDD"]
        );
        assert!(find_path(&edges, &fix("AAA", 0.), &fix("NONE", 4.)).is_none());
    }

    #[test]
    fn procedures_without_runway_segments_use_common_route_connectors() {
        let segments = vec![ProcedureSegment {
            route_type: "5".to_owned(),
            transition: "ALL".to_owned(),
            fixes: vec![fix("AAA", 0.1), fix("BBB", 0.2)],
        }];
        let departure = procedure_edges(&segments, ProcedureKind::Sid, &airport("ADEP"), "SID1");
        let arrival = procedure_edges(&segments, ProcedureKind::Star, &airport("AARR"), "STAR1");
        assert_eq!(
            identifiers(&find_path(&departure, &airport("ADEP"), &fix("BBB", 0.2)).unwrap()),
            ["ADEP", "AAA", "BBB"]
        );
        assert_eq!(
            identifiers(&find_path(&arrival, &fix("AAA", 0.1), &airport("AARR")).unwrap()),
            ["AAA", "BBB", "AARR"]
        );
        assert!(find_path(&arrival, &airport("AARR"), &fix("AAA", 0.1)).is_none());
    }

    #[tokio::test]
    async fn expands_a_real_constructed_route() {
        use crate::modules::flight::flight_plan::v2::{
            CandidateResolver, Constructor, Lexer, Parser, Solver,
        };
        let navdata = NavdataService::with_preferred_routes_path(
            "data/NavigraphDFDv2-2604.1.0.db?mode=ro",
            "data/Route-Server.csv",
        )
        .await
        .unwrap();
        let parsed = Parser::new(Lexer::new("ZBAA ELKUR W40 YQG ZSPD").parse_all().collect())
            .parse()
            .collect();
        let candidates = CandidateResolver::new(parsed)
            .resolve_candidates(&navdata)
            .await
            .unwrap()
            .collect();
        let constructed = Constructor::new(
            Solver::new(candidates, &navdata)
                .solve()
                .into_iter()
                .collect(),
        )
        .construct();
        let expanded = Expander::new(constructed.clone())
            .expand(&navdata)
            .await
            .unwrap();
        assert!(expanded.len() > constructed.len());
        assert_eq!(
            expanded.first().unwrap().from,
            constructed.first().unwrap().from
        );
        assert_eq!(expanded.last().unwrap().to, constructed.last().unwrap().to);
        assert!(identifiers(&expanded).contains(&"PANKI"));
        assert!(expanded.windows(2).all(|pair| pair[0].to == pair[1].from));
    }
}
