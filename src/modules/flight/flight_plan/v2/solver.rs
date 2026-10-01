use itertools::Itertools;
use ordered_float::OrderedFloat;

use crate::modules::{
    flight::flight_plan::v2::{
        FixCandidate, Ident, IdentCandidate, IdentWithCandidate, LegCandidate,
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
    ident: Ident<'s>,
    candidates: Vec<CandidateWithState>,
}

#[derive(Debug, PartialEq)]
struct CandidateWithState {
    candidate: IdentCandidate,
    state: State,
}

impl<'s, 'n> Solver<'s, 'n> {
    pub fn new(idents: Vec<IdentWithCandidate<'s>>, navdata: &'n NavdataService) -> Self {
        Self { idents, navdata }
    }

    pub fn solve(self) -> impl IntoIterator<Item = SolvedIdent<'s>> {
        let mut solved_idents = vec![];
        let first_pos = self
            .idents
            .first()
            .unwrap()
            .candidates
            .iter()
            .find_map(|c| match c {
                IdentCandidate::Fix(fix_candidate) => fix_candidate.position(),
                IdentCandidate::Leg(leg_candidate) => None,
            })
            .unwrap_or_default();
        for ident in self.idents.iter() {
            let solved = Self::solve_ident(
                ident,
                solved_idents.last().unwrap_or(&SolvedIdent {
                    ident: Ident {
                        ident: "",
                        amendments: vec![],
                        errors: vec![],
                    },
                    candidates: vec![CandidateWithState {
                        candidate: IdentCandidate::Fix(FixCandidate::UnknownWaypoint),
                        state: State {
                            last_token: StateToken::Fix(FixCandidate::UnknownWaypoint),
                            distance: 0.,
                            position_lat: first_pos.0,
                            position_lon: first_pos.1,
                        },
                    }],
                }),
            );
            solved_idents.push(solved);
        }
        solved_idents
    }

    fn solve_ident(
        ident: &IdentWithCandidate<'s>,
        last_solved: &SolvedIdent<'s>,
    ) -> SolvedIdent<'s> {
        let candidates = last_solved
            .candidates
            .iter()
            .flat_map(|last_candidate| Self::solve_ident_with_last_candidate(ident, last_candidate))
            .sorted_by_key(|c| OrderedFloat(c.state.distance))
            .take(3)
            .collect();
        SolvedIdent {
            ident: ident.ident.clone(),
            candidates,
        }
    }

    fn solve_ident_with_last_candidate(
        ident: &IdentWithCandidate<'s>,
        last_candidate: &CandidateWithState,
    ) -> impl IntoIterator<Item = CandidateWithState> {
        ident.candidates.iter().flat_map(|candidate| {
            Self::solve_ident_candidate_with_last_candidate(ident, candidate, last_candidate)
        })
    }

    fn solve_ident_candidate_with_last_candidate(
        ident: &IdentWithCandidate<'s>,
        candidate: &IdentCandidate,
        last_candidate: &CandidateWithState,
    ) -> impl IntoIterator<Item = CandidateWithState> {
        let state: Option<State> = match (&last_candidate.state.last_token, candidate) {
            (StateToken::Fix(last_fix), IdentCandidate::Fix(cur_fix)) => {
                Self::compute_state_fix_fix(&last_candidate.state, last_fix, cur_fix)
            }
            (StateToken::Fix(last_fix), IdentCandidate::Leg(cur_leg)) => {
                Self::compute_state_fix_leg(&last_candidate.state, last_fix, cur_leg)
            }
            (StateToken::Leg(last_fix, last_leg), IdentCandidate::Fix(cur_fix)) => {
                Self::compute_state_leg_fix(&last_candidate.state, last_leg, last_fix, cur_fix)
            }
            (StateToken::Leg(last_fix, last_leg), IdentCandidate::Leg(cur_leg)) => {
                Self::compute_state_leg_leg(&last_candidate.state, last_leg, last_fix, cur_leg)
            }
        };
        state.map(|state| CandidateWithState {
            candidate: candidate.clone(),
            state,
        })
    }

    fn compute_state_fix_fix(
        state: &State,
        last: &FixCandidate,
        cur: &FixCandidate,
    ) -> Option<State> {
        Some(State {
            last_token: StateToken::Fix(cur.clone()),
            distance: state.distance
                + cur.position().map_or(1000., |(lat, lon)| {
                    distance_nm(lat, lon, state.position_lat, state.position_lon)
                }),
            position_lat: cur.latitude().unwrap_or(state.position_lat),
            position_lon: cur.longitude().unwrap_or(state.position_lon),
        })
    }

    fn compute_state_fix_leg(
        state: &State,
        last: &FixCandidate,
        cur: &LegCandidate,
    ) -> Option<State> {
        // TODO: if fix not on leg return None
        Some(State {
            last_token: StateToken::Leg(last.clone(), cur.clone()),
            distance: state.distance,
            position_lat: state.position_lat,
            position_lon: state.position_lon,
        })
    }

    fn compute_state_leg_fix(
        state: &State,
        last: &LegCandidate,
        last_fix: &FixCandidate,
        cur: &FixCandidate,
    ) -> Option<State> {
        // TODO: if fix not on leg return None
        Some(State {
            last_token: StateToken::Fix(cur.clone()),
            // TODO: use real leg distance
            distance: state.distance
                + cur.position().map_or(1000., |(lat, lon)| {
                    distance_nm(lat, lon, state.position_lat, state.position_lon)
                })
                + matches!(last, LegCandidate::UnknownAirway)
                    .then(|| 1000.)
                    .unwrap_or(0.),
            position_lat: cur.latitude().unwrap_or(state.position_lat),
            position_lon: cur.longitude().unwrap_or(state.position_lon),
        })
    }

    fn compute_state_leg_leg(
        state: &State,
        last: &LegCandidate,
        last_fix: &FixCandidate,
        cur: &LegCandidate,
    ) -> Option<State> {
        // TODO: support leg-leg error recovery
        None
    }
}

// TODO: improve candidate sorting
// TODO: improve candidate pruning
// TODO: improve candidate validation

pub fn distance_nm(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
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
                        "  Candidate {}: {:?}\n    State: {:?}\n    Distance: {:.2} NM\n    Position: ({:.6}, {:.6})",
                        index + 1,
                        candidate.candidate,
                        state.last_token,
                        state.distance,
                        state.position_lat,
                        state.position_lon,
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
