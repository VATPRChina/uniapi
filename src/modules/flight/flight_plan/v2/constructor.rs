use arrayvec::ArrayString;

use super::{
    CandidateWithState, FixCandidate, IdentCandidate, LegCandidate, Lexer, LexerTokenValue,
    SolvedIdent,
};
use crate::modules::navdata::models::{
    Airport, AnyFix, DirectionRestriction, GeoPoint, Ndb, NdbKind, ResolvedLeg, Vhf, Waypoint,
    WaypointKind,
};

type PathEntry<'a, 's> = (&'a SolvedIdent<'s>, &'a CandidateWithState);

pub struct Constructor<'s> {
    idents: Vec<SolvedIdent<'s>>,
}

impl<'s> Constructor<'s> {
    pub fn new(idents: Vec<SolvedIdent<'s>>) -> Self {
        Self { idents }
    }

    /// Construct the first complete surviving path in solver order, following
    /// saved predecessor indices rather than selecting each entry independently.
    /// Empty or incomplete input yields no legs. Named connections retain their
    /// route identifier; this synchronous step does not expand navigation data.
    /// ResolvedLeg cannot represent the parsed speed/level or flight-rule amendments.
    pub fn construct(&self) -> Vec<ResolvedLeg> {
        self.idents
            .last()
            .into_iter()
            .flat_map(|ident| ident.candidates.iter().enumerate())
            .filter(|(_, candidate)| {
                matches!(candidate.candidate, IdentCandidate::Fix(_))
                    && candidate.distance().is_finite()
            })
            .find_map(|(index, _)| self.path(index).and_then(|path| construct_path(&path)))
            .unwrap_or_default()
    }

    fn path(&self, candidate_index: usize) -> Option<Vec<PathEntry<'_, 's>>> {
        let ident_index = self.idents.len().checked_sub(1)?;
        let candidate = self.idents[ident_index].candidates.get(candidate_index)?;
        let backwards: Vec<_> =
            std::iter::successors(Some((ident_index, candidate)), |(index, candidate)| {
                let previous = index.checked_sub(1)?;
                self.idents[previous]
                    .candidates
                    .get(candidate.last_candidate_idx)
                    .map(|candidate| (previous, candidate))
            })
            .collect();
        (backwards.len() == self.idents.len()).then(|| {
            backwards
                .into_iter()
                .rev()
                .map(|(index, candidate)| (&self.idents[index], candidate))
                .collect()
        })
    }
}

fn construct_path(path: &[PathEntry<'_, '_>]) -> Option<Vec<ResolvedLeg>> {
    if !matches!(path.first()?.1.candidate, IdentCandidate::Fix(_)) {
        return None;
    }
    let fixes: Vec<_> = path
        .iter()
        .enumerate()
        .filter_map(|(index, (ident, candidate))| match &candidate.candidate {
            IdentCandidate::Fix(fix) => Some((index, to_fix(ident.ident.identifier(), fix))),
            IdentCandidate::Leg(_) => None,
        })
        .collect();
    fixes
        .windows(2)
        .map(|pair| {
            let [(from_index, from), (to_index, to)] = pair else {
                unreachable!("windows contain two fixes")
            };
            let identifier = match &path[from_index + 1..*to_index] {
                [] => None,
                [(ident, candidate)] => match &candidate.candidate {
                    IdentCandidate::Leg(LegCandidate::Direct) => None,
                    IdentCandidate::Leg(_) => Some(ident.ident.identifier().to_owned()),
                    IdentCandidate::Fix(_) => return None,
                },
                _ => return None,
            };
            Some(ResolvedLeg {
                from: from.clone(),
                to: to.clone(),
                identifier,
                direction_restriction: DirectionRestriction::None,
            })
        })
        .collect()
}

fn to_fix(identifier: &str, candidate: &FixCandidate) -> AnyFix {
    // Reference candidates already contain the projected coordinates. Treat them
    // as points to avoid projecting twice or fitting the reference text into the
    // fixed-size identifier fields of the shared navigation models.
    if let Some((lat, lon)) = candidate.position()
        && matches!(
            Lexer::new(identifier)
                .parse_all()
                .next()
                .map(|token| token.value),
            Some(LexerTokenValue::IdentifierReference { .. })
        )
    {
        return AnyFix::GeoPoint(GeoPoint::new(lat, lon));
    }
    typed_fix(identifier, candidate).unwrap_or_else(|| AnyFix::Unknown(identifier.to_owned()))
}

fn typed_fix(identifier: &str, candidate: &FixCandidate) -> Option<AnyFix> {
    let (latitude, longitude) = candidate.position()?;
    Some(match candidate {
        FixCandidate::Airport { airport, .. } => AnyFix::Airport(Airport {
            identifier: *airport,
            latitude,
            longitude,
        }),
        FixCandidate::Geo { .. } => AnyFix::GeoPoint(GeoPoint::new(latitude, longitude)),
        FixCandidate::EnrouteWaypoint { icao_code, .. } => AnyFix::Waypoint(Waypoint {
            icao_code: ArrayString::from(icao_code.as_str()).ok()?,
            identifier: ArrayString::from(identifier).ok()?,
            latitude,
            longitude,
            kind: WaypointKind::Enroute,
        }),
        // Terminal candidates retain an airport scope, but no ICAO region.
        // Shared models cannot carry that scope; leave the region empty.
        FixCandidate::TerminalWaypoint { .. } => AnyFix::Waypoint(Waypoint {
            icao_code: ArrayString::new(),
            identifier: ArrayString::from(identifier).ok()?,
            latitude,
            longitude,
            kind: WaypointKind::Terminal,
        }),
        FixCandidate::EnrouteNdb { icao_code, .. } => AnyFix::Ndb(Ndb {
            icao_code: ArrayString::from(icao_code.as_str()).ok()?,
            identifier: ArrayString::from(identifier).ok()?,
            latitude,
            longitude,
            kind: NdbKind::Enroute,
        }),
        FixCandidate::TerminalNdb { .. } => AnyFix::Ndb(Ndb {
            icao_code: ArrayString::new(),
            identifier: ArrayString::from(identifier).ok()?,
            latitude,
            longitude,
            kind: NdbKind::Terminal,
        }),
        FixCandidate::EnrouteVor { icao_code, .. } => AnyFix::Vhf(Vhf {
            icao_code: ArrayString::from(icao_code.as_str()).ok()?,
            identifier: ArrayString::from(identifier).ok()?,
            latitude,
            longitude,
        }),
        FixCandidate::TerminalVor { .. } => AnyFix::Vhf(Vhf {
            icao_code: ArrayString::new(),
            identifier: ArrayString::from(identifier).ok()?,
            latitude,
            longitude,
        }),
        FixCandidate::UnknownWaypoint => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::{
        flight::flight_plan::v2::{FixCandidate, Ident, IdentWithCandidate, Solver},
        navdata::{
            models::{AnyFix, Fix, GeoPoint, Ndb, NdbKind, Vhf, Waypoint, WaypointKind},
            service::NavdataService,
        },
    };

    fn entry(identifier: &str, candidates: Vec<IdentCandidate>) -> IdentWithCandidate<'_> {
        IdentWithCandidate {
            ident: Ident {
                ident: identifier,
                amendments: Vec::new(),
                errors: Vec::new(),
            },
            candidates,
        }
    }

    fn airport(identifier: &str, longitude: f64) -> IdentCandidate {
        IdentCandidate::Fix(FixCandidate::Airport {
            airport: identifier.try_into().unwrap(),
            lat: 0.,
            lon: longitude,
        })
    }

    fn geo(longitude: f64) -> IdentCandidate {
        IdentCandidate::Fix(FixCandidate::Geo {
            lat: 0.,
            lon: longitude,
        })
    }

    async fn solve(entries: Vec<IdentWithCandidate<'_>>) -> Vec<SolvedIdent<'_>> {
        let navdata = NavdataService::with_preferred_routes_path(
            "data/NavigraphDFDv2-2604.1.0.db?mode=ro",
            "data/Route-Server.csv",
        )
        .await
        .unwrap();
        Solver::new(entries, &navdata).solve().into_iter().collect()
    }

    #[tokio::test]
    async fn follows_predecessors_in_solver_order() {
        let solved = solve(vec![
            entry("ADEP", vec![geo(20.), airport("ADEP", 0.)]),
            entry("FIX", vec![airport("AFIX", 1.)]),
            entry("AARR", vec![geo(30.), airport("AARR", 2.)]),
        ])
        .await;
        // Candidate priority orders the distant Geo before the cheaper Airport.
        assert!(matches!(
            solved[2].candidates[0].candidate,
            IdentCandidate::Fix(FixCandidate::Geo { .. })
        ));
        assert_eq!(solved[1].candidates[0].last_candidate_idx, 1);
        let legs = Constructor::new(solved).construct();
        assert_eq!(legs.len(), 2);
        assert_eq!(legs[0].from.identifier(), Some("ADEP"));
        assert_eq!(legs[0].from.longitude(), 0.);
        assert_eq!(legs[0].to, legs[1].from);
        assert_eq!(legs[1].to, AnyFix::GeoPoint(GeoPoint::new(0., 30.)));
        assert!(legs.iter().all(|leg| leg.identifier.is_none()));
    }

    #[tokio::test]
    async fn connects_explicit_direct_and_named_legs() {
        let connections = [
            ("DCT", LegCandidate::Direct, None),
            ("A1", LegCandidate::Airway, Some("A1")),
            (
                "SID1",
                LegCandidate::Sid {
                    airport: "ADEP".try_into().unwrap(),
                },
                Some("SID1"),
            ),
            (
                "STAR1",
                LegCandidate::Star {
                    airport: "AARR".try_into().unwrap(),
                },
                Some("STAR1"),
            ),
            ("MISSING", LegCandidate::UnknownAirway, Some("MISSING")),
        ];
        for (identifier, candidate, expected) in connections {
            let solved = solve(vec![
                entry("ADEP", vec![airport("ADEP", 0.)]),
                entry(identifier, vec![IdentCandidate::Leg(candidate)]),
                entry("AARR", vec![airport("AARR", 1.)]),
            ])
            .await;
            let legs = Constructor::new(solved).construct();
            assert_eq!(legs.len(), 1);
            assert_eq!(legs[0].identifier.as_deref(), expected);
            assert_eq!(legs[0].direction_restriction, DirectionRestriction::None);
            assert_eq!(legs[0].from.identifier(), Some("ADEP"));
            assert_eq!(legs[0].to.identifier(), Some("AARR"));
        }
    }

    #[tokio::test]
    async fn keeps_unknown_fixes_and_continuity() {
        let solved = solve(vec![
            entry("ADEP", vec![airport("ADEP", 0.)]),
            entry(
                "MISSING",
                vec![IdentCandidate::Fix(FixCandidate::UnknownWaypoint)],
            ),
            entry("AARR", vec![airport("AARR", 1.)]),
        ])
        .await;
        let legs = Constructor::new(solved).construct();
        assert_eq!(legs.len(), 2);
        assert_eq!(legs[0].to, AnyFix::Unknown("MISSING".to_owned()));
        assert_eq!(legs[0].to, legs[1].from);
    }

    #[tokio::test]
    async fn empty_single_and_incomplete_routes_have_no_legs() {
        assert!(Constructor::new(Vec::new()).construct().is_empty());
        let single = solve(vec![entry("ADEP", vec![airport("ADEP", 0.)])]).await;
        assert!(Constructor::new(single).construct().is_empty());
        let trailing_leg = solve(vec![
            entry("ADEP", vec![airport("ADEP", 0.)]),
            entry("DCT", vec![IdentCandidate::Leg(LegCandidate::Direct)]),
        ])
        .await;
        assert!(Constructor::new(trailing_leg).construct().is_empty());
        let exhausted = solve(vec![
            entry("ADEP", vec![airport("ADEP", 0.)]),
            entry("NONE", Vec::new()),
            entry("AARR", vec![airport("AARR", 1.)]),
        ])
        .await;
        assert!(Constructor::new(exhausted).construct().is_empty());
    }

    #[tokio::test]
    async fn rejects_broken_predecessor_links_without_returning_a_partial_route() {
        let solved = solve(vec![
            entry("ADEP", vec![airport("ADEP", 0.)]),
            entry("AARR", vec![airport("AARR", 1.)]),
        ])
        .await;
        let broken = solved
            .into_iter()
            .enumerate()
            .map(|(index, ident)| {
                if index == 0 {
                    SolvedIdent {
                        candidates: Vec::new(),
                        ..ident
                    }
                } else {
                    ident
                }
            })
            .collect();
        assert!(Constructor::new(broken).construct().is_empty());
    }

    #[tokio::test]
    async fn uses_the_next_complete_path_when_the_preferred_history_is_broken() {
        let solved = solve(vec![
            entry("ADEP", vec![geo(0.), airport("ADEP", 20.)]),
            entry("FIX", vec![geo(21.1), airport("AFIX", 1.)]),
            entry("AARR", vec![geo(21.2), airport("AARR", 2.)]),
        ])
        .await;
        let broken = solved
            .into_iter()
            .enumerate()
            .map(|(index, ident)| {
                if index == 0 {
                    SolvedIdent {
                        candidates: ident.candidates.into_iter().take(1).collect(),
                        ident: ident.ident,
                    }
                } else {
                    ident
                }
            })
            .collect();
        let legs = Constructor::new(broken).construct();
        assert_eq!(legs.len(), 2);
        assert_eq!(legs[0].from.longitude(), 0.);
        assert_eq!(legs[0].to.longitude(), 1.);
        assert_eq!(legs[1].to.longitude(), 2.);
    }

    #[tokio::test]
    async fn constructs_a_real_route_through_the_full_v2_pipeline() {
        use crate::modules::flight::flight_plan::v2::{CandidateResolver, Lexer, Parser};

        let navdata = NavdataService::with_preferred_routes_path(
            "data/NavigraphDFDv2-2604.1.0.db?mode=ro",
            "data/Route-Server.csv",
        )
        .await
        .unwrap();
        let tokens = Lexer::new("ZBAA ELKUR W40 YQG ZSPD").parse_all().collect();
        let parsed = Parser::new(tokens).parse().collect();
        let candidates = CandidateResolver::new(parsed)
            .resolve_candidates(&navdata)
            .await
            .unwrap()
            .collect();
        let solved = Solver::new(candidates, &navdata)
            .solve()
            .into_iter()
            .collect();
        let legs = Constructor::new(solved).construct();
        assert_eq!(legs.first().unwrap().from.identifier(), Some("ZBAA"));
        assert_eq!(legs.last().unwrap().to.identifier(), Some("ZSPD"));
        assert!(
            legs.iter()
                .any(|leg| leg.identifier.as_deref() == Some("W40"))
        );
        assert!(legs.windows(2).all(|pair| pair[0].to == pair[1].from));
    }

    #[test]
    fn converts_all_fix_types_without_losing_known_coordinates() {
        let candidates = [
            FixCandidate::Airport {
                airport: "TEST".try_into().unwrap(),
                lat: 12.,
                lon: 34.,
            },
            FixCandidate::Geo { lat: 12., lon: 34. },
            FixCandidate::EnrouteWaypoint {
                icao_code: "ZG".try_into().unwrap(),
                lat: 12.,
                lon: 34.,
            },
            FixCandidate::TerminalWaypoint {
                airport: "TEST".try_into().unwrap(),
                lat: 12.,
                lon: 34.,
            },
            FixCandidate::EnrouteNdb {
                icao_code: "ZG".try_into().unwrap(),
                lat: 12.,
                lon: 34.,
            },
            FixCandidate::TerminalNdb {
                airport: "TEST".try_into().unwrap(),
                lat: 12.,
                lon: 34.,
            },
            FixCandidate::EnrouteVor {
                icao_code: "ZG".try_into().unwrap(),
                lat: 12.,
                lon: 34.,
            },
            FixCandidate::TerminalVor {
                airport: "TEST".try_into().unwrap(),
                lat: 12.,
                lon: 34.,
            },
        ];
        for candidate in candidates {
            let fix = to_fix("FIX", &candidate);
            assert_eq!((fix.latitude(), fix.longitude()), (12., 34.));
            match candidate {
                FixCandidate::Airport { .. } => assert!(matches!(fix, AnyFix::Airport(_))),
                FixCandidate::Geo { .. } => assert!(matches!(fix, AnyFix::GeoPoint(_))),
                FixCandidate::EnrouteWaypoint { .. } => {
                    assert_eq!(fix.icao_code(), Some("ZG"));
                    assert!(matches!(
                        fix,
                        AnyFix::Waypoint(Waypoint {
                            kind: WaypointKind::Enroute,
                            ..
                        })
                    ));
                }
                FixCandidate::TerminalWaypoint { .. } => assert!(matches!(
                    fix,
                    AnyFix::Waypoint(Waypoint {
                        kind: WaypointKind::Terminal,
                        ..
                    })
                )),
                FixCandidate::EnrouteNdb { .. } => {
                    assert_eq!(fix.icao_code(), Some("ZG"));
                    assert!(matches!(
                        fix,
                        AnyFix::Ndb(Ndb {
                            kind: NdbKind::Enroute,
                            ..
                        })
                    ));
                }
                FixCandidate::TerminalNdb { .. } => assert!(matches!(
                    fix,
                    AnyFix::Ndb(Ndb {
                        kind: NdbKind::Terminal,
                        ..
                    })
                )),
                FixCandidate::EnrouteVor { .. } => {
                    assert_eq!(fix.icao_code(), Some("ZG"));
                    assert!(matches!(fix, AnyFix::Vhf(_)));
                }
                FixCandidate::TerminalVor { .. } => assert!(matches!(fix, AnyFix::Vhf(_))),
                FixCandidate::UnknownWaypoint => unreachable!(),
            }
        }
    }

    #[test]
    fn references_use_projected_coordinates_without_projecting_again() {
        let fix = to_fix(
            "ABCDE090100",
            &FixCandidate::EnrouteWaypoint {
                icao_code: "ZG".try_into().unwrap(),
                lat: 12.,
                lon: 34.,
            },
        );
        assert_eq!(fix, AnyFix::GeoPoint(GeoPoint::new(12., 34.)));
        assert_eq!(
            to_fix("ABCDE090100", &FixCandidate::UnknownWaypoint),
            AnyFix::Unknown("ABCDE090100".to_owned())
        );
    }

    #[test]
    fn oversized_identifiers_recover_without_panicking() {
        assert_eq!(
            to_fix(
                "TOOLONG",
                &FixCandidate::EnrouteWaypoint {
                    icao_code: "ZG".try_into().unwrap(),
                    lat: 12.,
                    lon: 34.,
                }
            ),
            AnyFix::Unknown("TOOLONG".to_owned())
        );
    }
}
