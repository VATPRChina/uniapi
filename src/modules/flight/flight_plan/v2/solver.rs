use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;
use ordered_float::OrderedFloat;
use sea_query::IndexType::BTree;

use crate::modules::{
    flight::flight_plan::v2::{
        FixCandidate, Ident,
        IdentCandidate::{self, Fix},
        IdentWithCandidate, LegCandidate,
    },
    navdata::service::NavdataService,
};

pub struct Solver<'s, 'n> {
    idents: Vec<IdentWithCandidate<'s>>,
    navdata: &'n NavdataService,
}

#[derive(Debug, PartialEq)]
struct State {
    last_token: StateToken,
    /// Recovery interpretations anywhere in the predecessor path.
    fallbacks: usize,
    distance: f64,
    position_lat: f64,
    position_lon: f64,
}

#[derive(Debug, PartialEq)]
enum StateToken {
    Fix(FixCandidate),
    Leg(FixCandidate, LegCandidate),
}

#[derive(Debug, PartialEq)]
pub struct SolvedIdent<'s> {
    pub ident: Ident<'s>,
    pub candidates: Vec<CandidateWithState>,
}

#[derive(Debug, PartialEq)]
pub struct CandidateWithState {
    pub candidate: IdentCandidate,
    state: State,
    pub last_candidate_idx: usize,
}

impl CandidateWithState {
    pub(super) fn distance(&self) -> f64 {
        self.state.distance
    }
}

impl<'s, 'n> Solver<'s, 'n> {
    pub fn new(idents: Vec<IdentWithCandidate<'s>>, navdata: &'n NavdataService) -> Self {
        Self { idents, navdata }
    }

    pub fn solve(self) -> impl IntoIterator<Item = SolvedIdent<'s>> {
        let mut solved_idents = vec![];

        if let Some(first) = self.idents.first() {
            solved_idents.push(Self::solve_init_ident(first));
        } else {
            return solved_idents;
        }

        for ident in self.idents.iter().skip(1) {
            let solved = Self::solve_ident(ident, solved_idents.last().unwrap());
            solved_idents.push(solved);
        }
        solved_idents
    }

    fn solve_init_ident(ident: &IdentWithCandidate<'s>) -> SolvedIdent<'s> {
        let candidates: Vec<_> = ident
            .candidates
            .iter()
            .sorted_by_key(|c| c.priority())
            .flat_map(|candidate| match candidate {
                IdentCandidate::Fix(fix) => Some(CandidateWithState {
                    candidate: candidate.clone(),
                    state: State {
                        last_token: StateToken::Fix(fix.clone()),
                        fallbacks: usize::from(matches!(fix, FixCandidate::UnknownWaypoint)),
                        distance: 0.,
                        position_lat: fix.latitude().unwrap_or_default(),
                        position_lon: fix.longitude().unwrap_or_default(),
                    },
                    last_candidate_idx: 0,
                }),
                IdentCandidate::Leg(_) => None,
            })
            .collect();

        let candidates = CandidateSortPruneState::new(candidates.iter()).handle(candidates);

        SolvedIdent {
            ident: ident.ident.clone(),
            candidates,
        }
    }

    fn solve_ident(
        ident: &IdentWithCandidate<'s>,
        last_solved: &SolvedIdent<'s>,
    ) -> SolvedIdent<'s> {
        let candidates: Vec<_> = last_solved
            .candidates
            .iter()
            .enumerate()
            .flat_map(|(last_candidate_idx, last_candidate)| {
                Self::solve_ident_with_last_candidate(ident, last_candidate, last_candidate_idx)
            })
            .collect();

        let candidates = CandidateSortPruneState::new(candidates.iter()).handle(candidates);

        SolvedIdent {
            ident: ident.ident.clone(),
            candidates,
        }
    }

    fn solve_ident_with_last_candidate(
        ident: &IdentWithCandidate<'s>,
        last_candidate: &CandidateWithState,
        last_candidate_idx: usize,
    ) -> impl IntoIterator<Item = CandidateWithState> {
        ident.candidates.iter().flat_map(move |candidate| {
            Self::solve_ident_candidate_with_last_candidate(
                ident,
                candidate,
                last_candidate,
                last_candidate_idx,
            )
        })
    }

    fn solve_ident_candidate_with_last_candidate(
        ident: &IdentWithCandidate<'s>,
        candidate: &IdentCandidate,
        last_candidate: &CandidateWithState,
        last_candidate_idx: usize,
    ) -> impl IntoIterator<Item = CandidateWithState> {
        (last_candidate.state)
            .next_state(candidate)
            .map(|state| CandidateWithState {
                candidate: candidate.clone(),
                state,
                last_candidate_idx,
            })
    }
}

trait Priority {
    fn priority(&self) -> u8;
}

impl Priority for IdentCandidate {
    fn priority(&self) -> u8 {
        match self {
            IdentCandidate::Leg(LegCandidate::Direct) => 1,
            IdentCandidate::Leg(LegCandidate::Airway) => 2,
            IdentCandidate::Fix(FixCandidate::Geo { .. }) => 3,
            IdentCandidate::Fix(FixCandidate::Airport { .. }) => 4,
            IdentCandidate::Fix(FixCandidate::EnrouteVor { .. }) => 5,
            IdentCandidate::Fix(FixCandidate::EnrouteNdb { .. }) => 6,
            IdentCandidate::Fix(FixCandidate::EnrouteWaypoint { .. }) => 7,
            IdentCandidate::Leg(LegCandidate::Sid { .. }) => 8,
            IdentCandidate::Leg(LegCandidate::Star { .. }) => 9,
            IdentCandidate::Fix(FixCandidate::TerminalVor { .. }) => 10,
            IdentCandidate::Fix(FixCandidate::TerminalNdb { .. }) => 11,
            IdentCandidate::Fix(FixCandidate::TerminalWaypoint { .. }) => 12,
            IdentCandidate::Leg(LegCandidate::UnknownAirway) => 101,
            IdentCandidate::Fix(FixCandidate::UnknownWaypoint) => 102,
        }
    }
}

struct CandidateSortPruneState {
    item_kind_count: BTreeMap<u8, usize>,
}

impl CandidateSortPruneState {
    pub fn new<'c>(candiates: impl Iterator<Item = &'c CandidateWithState>) -> Self {
        let item_kind_count = candiates.fold(BTreeMap::new(), |mut acc, candidate| {
            *acc.entry(candidate.candidate.priority()).or_insert(0) += 1;
            acc
        });
        CandidateSortPruneState { item_kind_count }
    }

    pub fn handle(&mut self, candiates: Vec<CandidateWithState>) -> Vec<CandidateWithState> {
        let has_known_fix = candiates.iter().any(|c| {
            matches!(c.candidate, IdentCandidate::Fix(_))
                && !matches!(
                    c.candidate,
                    IdentCandidate::Fix(FixCandidate::UnknownWaypoint),
                )
        });
        let has_known_leg = candiates.iter().any(|c| {
            matches!(c.candidate, IdentCandidate::Leg(_))
                && !matches!(
                    c.candidate,
                    IdentCandidate::Leg(LegCandidate::UnknownAirway),
                )
        });

        candiates
            .into_iter()
            .into_group_map_by(|c| c.candidate.priority())
            .into_iter()
            .flat_map(|(p, g)| {
                g.into_iter()
                    // Prefer a known history before comparing its distance. A
                    // cheap unknown point must not displace a published airway.
                    .sorted_by_key(|c| (c.state.fallbacks, OrderedFloat(c.state.distance)))
                    .next()
            })
            .filter(|c| {
                !matches!(
                    c.candidate,
                    IdentCandidate::Fix(FixCandidate::UnknownWaypoint)
                ) || !has_known_fix
            })
            .filter(|c| {
                !matches!(
                    c.candidate,
                    IdentCandidate::Leg(LegCandidate::UnknownAirway)
                ) || !has_known_leg
            })
            .sorted_by_key(|c| c.candidate.priority())
            .collect()
    }
}

impl State {
    pub fn next_state(&self, candidate: &IdentCandidate) -> Option<State> {
        match (&self.last_token, candidate) {
            (StateToken::Fix(last_fix), IdentCandidate::Fix(cur_fix)) => {
                self.next_state_fix_fix(last_fix, cur_fix)
            }
            (StateToken::Fix(last_fix), IdentCandidate::Leg(cur_leg)) => {
                self.next_state_fix_leg(last_fix, cur_leg)
            }
            (StateToken::Leg(last_fix, last_leg), IdentCandidate::Fix(cur_fix)) => {
                self.next_state_leg_fix(last_leg, last_fix, cur_fix)
            }
            (StateToken::Leg(last_fix, last_leg), IdentCandidate::Leg(cur_leg)) => {
                self.next_state_leg_leg(last_leg, last_fix, cur_leg)
            }
        }
    }

    pub fn next_state_fix_fix(&self, last: &FixCandidate, cur: &FixCandidate) -> Option<State> {
        Some(State {
            last_token: StateToken::Fix(cur.clone()),
            fallbacks: self.fallbacks + usize::from(matches!(cur, FixCandidate::UnknownWaypoint)),
            distance: self.distance
                + cur.position().map_or(1000., |(lat, lon)| {
                    distance_nm(lat, lon, self.position_lat, self.position_lon)
                }),
            position_lat: cur.latitude().unwrap_or(self.position_lat),
            position_lon: cur.longitude().unwrap_or(self.position_lon),
        })
    }

    pub fn next_state_fix_leg(&self, last: &FixCandidate, cur: &LegCandidate) -> Option<State> {
        // TODO: if fix not on leg return None
        Some(State {
            last_token: StateToken::Leg(last.clone(), cur.clone()),
            fallbacks: self.fallbacks + usize::from(matches!(cur, LegCandidate::UnknownAirway)),
            distance: self.distance,
            position_lat: self.position_lat,
            position_lon: self.position_lon,
        })
    }

    pub fn next_state_leg_fix(
        &self,
        last: &LegCandidate,
        last_fix: &FixCandidate,
        cur: &FixCandidate,
    ) -> Option<State> {
        // TODO: if fix not on leg return None
        Some(State {
            last_token: StateToken::Fix(cur.clone()),
            fallbacks: self.fallbacks + usize::from(matches!(cur, FixCandidate::UnknownWaypoint)),
            // TODO: use real leg distance
            distance: self.distance
                + cur.position().map_or(1000., |(lat, lon)| {
                    distance_nm(lat, lon, self.position_lat, self.position_lon)
                })
                + matches!(last, LegCandidate::UnknownAirway)
                    .then(|| 1000.)
                    .unwrap_or(0.),
            position_lat: cur.latitude().unwrap_or(self.position_lat),
            position_lon: cur.longitude().unwrap_or(self.position_lon),
        })
    }

    pub fn next_state_leg_leg(
        &self,
        last: &LegCandidate,
        last_fix: &FixCandidate,
        cur: &LegCandidate,
    ) -> Option<State> {
        // TODO: support leg-leg error recovery
        None
    }
}

fn distance_nm(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_NM: f64 = 3440.065;

    let lat1 = lat1.to_radians();
    let lon1 = lon1.to_radians();
    let lat2 = lat2.to_radians();
    let lon2 = lon2.to_radians();

    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_NM * c
}

/// Print every interpretation separately so its state can be followed between entries.
#[cfg(test)]
fn pretty_print_idents(idents: &[SolvedIdent<'_>]) {
    let output = idents
        .iter()
        .enumerate()
        .map(|(index, solved)| {
            let candidates = solved
                .candidates
                .iter()
                .enumerate()
                .map(|(index, candidate)| {
                    let state = &candidate.state;
                    format!(
                        "  Candidate {}: {:?}\n    State: {:?}\n    Distance: {:.2} NM\n    Position: ({:.6}, {:.6})\n    Last: Candidate {}",
                        index + 1,
                        candidate.candidate,
                        state.last_token,
                        state.distance,
                        state.position_lat,
                        state.position_lon,
                        candidate.last_candidate_idx,
                    )
                })
                .join("\n");
            format!(
                "[{index}] {}\n{}",
                solved.ident.identifier(),
                if candidates.is_empty() {
                    "  (no candidates)"
                } else {
                    &candidates
                },
            )
        })
        .join("\n\n");
    println!(
        "{}",
        if output.is_empty() {
            "(no idents)"
        } else {
            &output
        }
    );
}

#[cfg(test)]
#[test]
fn prefers_known_predecessors_over_shorter_recovery_paths() {
    let entry = |ident, candidates| IdentWithCandidate {
        ident: Ident {
            ident,
            amendments: Vec::new(),
            errors: Vec::new(),
        },
        candidates,
    };
    let vor = |lon| {
        IdentCandidate::Fix(FixCandidate::EnrouteVor {
            icao_code: "ZG".try_into().unwrap(),
            lat: 0.,
            lon,
        })
    };
    let departure = Solver::solve_init_ident(&entry(
        "ADEP",
        vec![IdentCandidate::Fix(FixCandidate::UnknownWaypoint)],
    ));
    let fix = Solver::solve_ident(
        &entry(
            "LYA",
            vec![vor(150.), IdentCandidate::Leg(LegCandidate::UnknownAirway)],
        ),
        &departure,
    );
    let airway = Solver::solve_ident(
        &entry(
            "W45",
            vec![
                IdentCandidate::Leg(LegCandidate::Airway),
                IdentCandidate::Fix(FixCandidate::UnknownWaypoint),
            ],
        ),
        &fix,
    );
    let arrival = Solver::solve_ident(&entry("ML", vec![vor(1.)]), &airway);
    let selected = &arrival.candidates[0];
    let predecessor = &airway.candidates[selected.last_candidate_idx];
    let recovery = airway
        .candidates
        .iter()
        .find(|candidate| {
            matches!(
                candidate.candidate,
                IdentCandidate::Fix(FixCandidate::UnknownWaypoint)
            )
        })
        .unwrap();
    // The known detour is much longer than returning via an unknown W45 at
    // the initial position, but W45 must keep its published airway meaning.
    assert!(selected.distance() > recovery.distance() + distance_nm(0., 1., 0., 0.));
    assert_eq!(selected.state.fallbacks, 1); // Only the missing departure.
    assert_eq!(
        predecessor.candidate,
        IdentCandidate::Leg(LegCandidate::Airway)
    );
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use crate::modules::flight::flight_plan::v2::{CandidateResolver, Lexer, Parser};

    let lexed = Lexer::new(
        "ZBAA ELKUR W40 YQG W142 DALIM A593 DPX A470 DALNU W166 ZJ W167 SASAN R343 EKIMU ZSPD",
    )
    .parse_all()
    .collect();
    let parsed = Parser::new(lexed).parse().collect();
    let navdata = NavdataService::with_preferred_routes_path(
        "data/NavigraphDFDv2-2604.1.0.db?mode=ro",
        "data/Route-Server.csv",
    )
    .await
    .unwrap();
    let resolved = CandidateResolver::new(parsed)
        .resolve_candidates(&navdata)
        .await
        .unwrap()
        .collect();
    let solved: Vec<_> = Solver::new(resolved, &navdata)
        .solve()
        .into_iter()
        .collect();
    pretty_print_idents(&solved);
    assert!(solved.is_empty(), "expected no solved identifiers");
}

#[cfg(test)]
#[tokio::test]
async fn test2() {
    use crate::modules::flight::flight_plan::v2::{CandidateResolver, Lexer, Parser};

    let lexed = Lexer::new(
        "ZBAA ELKUR W40 PANKI W158 AR ONAXU ATVIM W127 HFE VILID P37 P321 P206 P468 P179 P262 P599 JDZ P395 P263 P645 OMDEM XLN A470 DOTMI RPLL",
    )
    .parse_all()
    .collect();
    let parsed = Parser::new(lexed).parse().collect();
    let navdata = NavdataService::with_preferred_routes_path(
        "data/NavigraphDFDv2-2604.1.0.db?mode=ro",
        "data/Route-Server.csv",
    )
    .await
    .unwrap();
    let resolved = CandidateResolver::new(parsed)
        .resolve_candidates(&navdata)
        .await
        .unwrap()
        .collect();
    let solved: Vec<_> = Solver::new(resolved, &navdata)
        .solve()
        .into_iter()
        .collect();
    pretty_print_idents(&solved);
    assert!(solved.is_empty(), "expected no solved identifiers");
}
