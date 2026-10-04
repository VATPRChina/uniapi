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
                        fallbacks: usize::from(fix.is_unknown()),
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
            IdentCandidate::Leg(LegCandidate::UnknownSid) => 101,
            IdentCandidate::Leg(LegCandidate::UnknownStar) => 102,
            IdentCandidate::Leg(LegCandidate::UnknownAirway) => 103,
            IdentCandidate::Fix(FixCandidate::UnknownWaypoint) => 104,
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
        let has_known_fix = candiates
            .iter()
            .any(|c| matches!(c.candidate, IdentCandidate::Fix(_)) && !c.candidate.is_unknown());
        let has_known_leg = candiates
            .iter()
            .any(|c| matches!(c.candidate, IdentCandidate::Leg(_)) && !c.candidate.is_unknown());

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
            .filter(|c| match &c.candidate {
                Fix(fix_candidate) => !fix_candidate.is_unknown() || !has_known_fix,
                // do not prune leg as leg depends on future fix
                IdentCandidate::Leg(leg_candidate) => true,
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
            fallbacks: self.fallbacks + usize::from(cur.is_unknown()),
            distance: self.distance
                + cur.position().map_or(1000., |(lat, lon)| {
                    distance_nm(lat, lon, self.position_lat, self.position_lon)
                }),
            position_lat: cur.latitude().unwrap_or(self.position_lat),
            position_lon: cur.longitude().unwrap_or(self.position_lon),
        })
    }

    pub fn next_state_fix_leg(&self, last: &FixCandidate, cur: &LegCandidate) -> Option<State> {
        if !matches!(last, FixCandidate::Airport { .. }) && matches!(cur, LegCandidate::UnknownSid)
        {
            return None;
        }
        if let LegCandidate::Sid { airport: sid_aprt } = cur {
            if let FixCandidate::Airport { airport, .. } = last {
                if airport != sid_aprt {
                    return None;
                }
            } else {
                return None;
            }
        }
        // TODO: if fix not on leg return None
        Some(State {
            last_token: StateToken::Leg(last.clone(), cur.clone()),
            fallbacks: self.fallbacks + usize::from(cur.is_unknown()),
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
        if matches!(last, LegCandidate::UnknownStar) && !matches!(cur, FixCandidate::Airport { .. })
        {
            return None;
        }
        if let LegCandidate::Star { airport: star_aprt } = last {
            if let FixCandidate::Airport { airport, .. } = cur {
                if airport != star_aprt {
                    return None;
                }
            } else {
                return None;
            }
        }
        // TODO: if fix not on leg return None
        Some(State {
            last_token: StateToken::Fix(cur.clone()),
            fallbacks: self.fallbacks + usize::from(cur.is_unknown()),
            // TODO: use real leg distance
            distance: self.distance
                + cur.position().map_or(1000., |(lat, lon)| {
                    distance_nm(lat, lon, self.position_lat, self.position_lon)
                })
                + if last.is_unknown() { 1000. } else { 0. },
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
