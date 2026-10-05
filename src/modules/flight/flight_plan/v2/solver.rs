use itertools::Itertools;
use ordered_float::OrderedFloat;

use crate::modules::flight::flight_plan::v2::{Ident, IdentCandidate, IdentWithCandidate};
use crate::modules::navdata::models::{AnyFix, Fix, NavProc, Ndb, NdbKind, Waypoint, WaypointKind};

/// Resolved candidates own navigation data; search states only borrow it.
pub struct Solver<'c, 's> {
    idents: &'c [IdentWithCandidate<'s>],
}

#[derive(Debug, PartialEq)]
struct State<'c> {
    last_token: StateToken<'c>,
    /// Recovery interpretations anywhere in the predecessor path.
    fallbacks: usize,
    distance: f64,
    position_lat: f64,
    position_lon: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StateToken<'c> {
    Fix(&'c AnyFix),
    Leg(&'c AnyFix, &'c NavProc),
}

#[derive(Debug, PartialEq)]
pub struct SolvedIdent<'c, 's> {
    pub ident: &'c Ident<'s>,
    pub candidates: Vec<CandidateWithState<'c>>,
}

#[derive(Debug, PartialEq)]
pub struct CandidateWithState<'c> {
    pub candidate: &'c IdentCandidate,
    state: State<'c>,
    pub last_candidate_idx: usize,
}

impl CandidateWithState<'_> {
    pub(super) fn distance(&self) -> f64 {
        self.state.distance
    }
}

impl<'c, 's> Solver<'c, 's> {
    pub fn new(idents: &'c [IdentWithCandidate<'s>]) -> Self {
        Self { idents }
    }

    pub fn solve(self) -> Vec<SolvedIdent<'c, 's>> {
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

    fn solve_init_ident(ident: &'c IdentWithCandidate<'s>) -> SolvedIdent<'c, 's> {
        let candidates: Vec<_> = ident
            .candidates
            .iter()
            .sorted_by_key(|c| c.priority())
            .flat_map(|candidate| match candidate {
                IdentCandidate::Fix(fix) => Some(CandidateWithState {
                    candidate,
                    state: State {
                        last_token: StateToken::Fix(fix),
                        fallbacks: usize::from(fix.is_unknown()),
                        distance: 0.,
                        position_lat: fix.valid_latitude_or(0.),
                        position_lon: fix.valid_longitude_or(0.),
                    },
                    last_candidate_idx: 0,
                }),
                IdentCandidate::Leg(_) => None,
            })
            .collect();

        let candidates = CandidateSortPruneState::new(candidates.iter()).handle(candidates);

        SolvedIdent {
            ident: &ident.ident,
            candidates,
        }
    }

    fn solve_ident(
        ident: &'c IdentWithCandidate<'s>,
        last_solved: &SolvedIdent<'c, 's>,
    ) -> SolvedIdent<'c, 's> {
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
            ident: &ident.ident,
            candidates,
        }
    }

    fn solve_ident_with_last_candidate(
        ident: &'c IdentWithCandidate<'s>,
        last_candidate: &CandidateWithState<'c>,
        last_candidate_idx: usize,
    ) -> impl IntoIterator<Item = CandidateWithState<'c>> {
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
        _ident: &'c IdentWithCandidate<'s>,
        candidate: &'c IdentCandidate,
        last_candidate: &CandidateWithState<'c>,
        last_candidate_idx: usize,
    ) -> impl IntoIterator<Item = CandidateWithState<'c>> {
        (last_candidate.state)
            .next_state(candidate)
            .map(|state| CandidateWithState {
                candidate,
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
            IdentCandidate::Leg(NavProc::Direct) => 1,
            IdentCandidate::Leg(NavProc::Airway(_)) => 2,
            IdentCandidate::Fix(AnyFix::GeoPoint(_)) => 3,
            IdentCandidate::Fix(AnyFix::Airport(_)) => 4,
            IdentCandidate::Fix(AnyFix::Vhf(_)) => 5,
            IdentCandidate::Fix(AnyFix::Ndb(Ndb {
                kind: NdbKind::Enroute,
                ..
            })) => 6,
            IdentCandidate::Fix(AnyFix::Waypoint(Waypoint {
                kind: WaypointKind::Enroute,
                ..
            })) => 7,
            IdentCandidate::Leg(NavProc::Sid(_)) => 8,
            IdentCandidate::Leg(NavProc::Star(_)) => 9,
            IdentCandidate::Fix(AnyFix::Ndb(Ndb {
                kind: NdbKind::Terminal,
                ..
            })) => 10,
            IdentCandidate::Fix(AnyFix::Waypoint(Waypoint {
                kind: WaypointKind::Terminal,
                ..
            })) => 11,
            IdentCandidate::Leg(NavProc::UnknownSid(_)) => 101,
            IdentCandidate::Leg(NavProc::UnknownStar(_)) => 102,
            IdentCandidate::Leg(NavProc::UnknownAirway(_)) => 103,
            IdentCandidate::Fix(AnyFix::Unknown(_)) => 104,
            IdentCandidate::Fix(AnyFix::FixReference(_)) => 255, // unreachable
        }
    }
}

struct CandidateSortPruneState {}

impl CandidateSortPruneState {
    pub fn new<'a, 'c: 'a>(_: impl Iterator<Item = &'a CandidateWithState<'c>>) -> Self {
        CandidateSortPruneState {}
    }

    pub fn handle<'c>(
        &self,
        candiates: Vec<CandidateWithState<'c>>,
    ) -> Vec<CandidateWithState<'c>> {
        let has_known_fix = candiates
            .iter()
            .any(|c| matches!(c.candidate, IdentCandidate::Fix(_)) && !c.candidate.is_unknown());

        candiates
            .into_iter()
            .into_group_map_by(|c| c.candidate.priority())
            .into_values()
            .flat_map(|g| {
                g.into_iter()
                    // Prefer a known history before comparing its distance. A
                    // cheap unknown point must not displace a published airway.
                    .sorted_by_key(|c| (c.state.fallbacks, OrderedFloat(c.state.distance)))
                    .next()
            })
            .filter(|c| match &c.candidate {
                IdentCandidate::Fix(fix_candidate) => !fix_candidate.is_unknown() || !has_known_fix,
                // do not prune leg as leg depends on future fix
                IdentCandidate::Leg(_) => true,
            })
            .sorted_by_key(|c| c.candidate.priority())
            .collect()
    }
}

impl<'c> State<'c> {
    pub fn next_state(&self, candidate: &'c IdentCandidate) -> Option<State<'c>> {
        match (self.last_token, candidate) {
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

    pub fn next_state_fix_fix(&self, _last: &'c AnyFix, cur: &'c AnyFix) -> Option<State<'c>> {
        Some(State {
            last_token: StateToken::Fix(cur),
            fallbacks: self.fallbacks + usize::from(cur.is_unknown()),
            distance: self.distance
                + cur.position().map_or(1000., |(lat, lon)| {
                    distance_nm(lat, lon, self.position_lat, self.position_lon)
                }),
            position_lat: cur.valid_latitude_or(self.position_lat),
            position_lon: cur.valid_longitude_or(self.position_lon),
        })
    }

    pub fn next_state_fix_leg(&self, last: &'c AnyFix, cur: &'c NavProc) -> Option<State<'c>> {
        if !matches!(last, AnyFix::Airport { .. }) && matches!(cur, NavProc::UnknownSid(_)) {
            return None;
        }
        if let NavProc::Sid(sid) = cur {
            if let AnyFix::Airport(airport) = last {
                if airport.identifier != sid.airport {
                    return None;
                }
            } else {
                return None;
            }
        }
        // TODO: if fix not on leg return None
        Some(State {
            last_token: StateToken::Leg(last, cur),
            fallbacks: self.fallbacks + usize::from(cur.is_unknown()),
            distance: self.distance,
            position_lat: self.position_lat,
            position_lon: self.position_lon,
        })
    }

    pub fn next_state_leg_fix(
        &self,
        last: &'c NavProc,
        _last_fix: &'c AnyFix,
        cur: &'c AnyFix,
    ) -> Option<State<'c>> {
        if matches!(last, NavProc::UnknownStar(_)) && !matches!(cur, AnyFix::Airport(_)) {
            return None;
        }
        if let NavProc::Star(star) = last {
            if let AnyFix::Airport(airport) = cur {
                if airport.identifier != star.airport {
                    return None;
                }
            } else {
                return None;
            }
        }
        // TODO: if fix not on leg return None
        Some(State {
            last_token: StateToken::Fix(cur),
            fallbacks: self.fallbacks + usize::from(cur.is_unknown()),
            // TODO: use real leg distance
            distance: self.distance
                + cur.position().map_or(1000., |(lat, lon)| {
                    distance_nm(lat, lon, self.position_lat, self.position_lon)
                }),
            position_lat: cur.valid_latitude_or(self.position_lat),
            position_lon: cur.valid_longitude_or(self.position_lon),
        })
    }

    pub fn next_state_leg_leg(
        &self,
        _last: &'c NavProc,
        _last_fix: &'c AnyFix,
        _cur: &'c NavProc,
    ) -> Option<State<'c>> {
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
