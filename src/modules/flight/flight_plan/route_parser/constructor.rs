use super::{CandidateWithState, IdentCandidate, SolvedIdent};
use crate::modules::navdata::models::{DirectionRestriction, LegKind, NavProc, ResolvedLeg};

type PathEntry<'a, 's> = (&'a SolvedIdent<'s>, &'a CandidateWithState);

/// A logical route leg and the exact procedure selected by the solver.
/// Direct connections have no procedure; named procedures retain their loaded legs.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstructedLeg {
    pub leg: ResolvedLeg,
    pub procedure: Option<NavProc>,
}

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
    /// route identifier and selected procedure for subsequent expansion.
    /// ResolvedLeg cannot represent the parsed speed/level or flight-rule amendments.
    pub fn construct(&self) -> Vec<ConstructedLeg> {
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

fn construct_path(path: &[PathEntry<'_, '_>]) -> Option<Vec<ConstructedLeg>> {
    if !matches!(path.first()?.1.candidate, IdentCandidate::Fix(_)) {
        return None;
    }
    let fixes: Vec<_> = path
        .iter()
        .enumerate()
        .filter_map(|(index, (_, candidate))| match &candidate.candidate {
            IdentCandidate::Fix(fix) => Some((index, fix)),
            IdentCandidate::Leg(_) => None,
        })
        .collect();
    fixes
        .windows(2)
        .map(|pair| {
            let [(from_index, from), (to_index, to)] = pair else {
                unreachable!("windows contain two fixes")
            };
            let (identifier, procedure) = match &path[from_index + 1..*to_index] {
                [] => (None, None),
                [(ident, candidate)] => match &candidate.candidate {
                    IdentCandidate::Leg(NavProc::Direct) => (None, None),
                    IdentCandidate::Leg(leg) => {
                        (Some(ident.ident.identifier().to_owned()), Some(leg.clone()))
                    }
                    IdentCandidate::Fix(_) => return None,
                },
                _ => return None,
            };
            Some(ConstructedLeg {
                leg: ResolvedLeg {
                    from: (*from).to_owned(),
                    to: (*to).to_owned(),
                    identifier,
                    is_unknown: procedure.as_ref().is_some_and(NavProc::is_unknown),
                    kind: procedure.as_ref().map_or(LegKind::Direct, NavProc::kind),
                    direction_restriction: DirectionRestriction::None,
                },
                procedure,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::flight::dto::FlightRouteLeg;
    use crate::modules::flight::flight_plan::route_parser::{
        Expander, IdentWithCandidate, LexGrouper, Lexer, Solver,
    };
    use crate::modules::navdata::models::{Airport, Airway, AnyFix, GeoPoint, Sid, Star};

    #[test]
    fn leg_kind_survives_construction_expansion_and_dto_conversion() {
        let airport = |identifier: &str, longitude| -> AnyFix {
            AnyFix::Airport(Airport {
                identifier: identifier.try_into().unwrap(),
                latitude: 30.,
                longitude,
            })
        };
        let from = airport("AAAA", 110.);
        let to = airport("BBBB", 112.);
        let via: AnyFix = GeoPoint::new(30., 111.).into();
        let published = |kind| {
            [(from.clone(), via.clone()), (via.clone(), to.clone())]
                .into_iter()
                .map(|(from, to)| ResolvedLeg {
                    from,
                    to,
                    identifier: Some("A1".to_owned()),
                    is_unknown: false,
                    kind,
                    direction_restriction: DirectionRestriction::None,
                })
                .collect()
        };
        let cases = [
            (NavProc::Direct, LegKind::Direct, false),
            (
                NavProc::Airway(Airway {
                    identifier: "A1".try_into().unwrap(),
                    legs: published(LegKind::Airway),
                }),
                LegKind::Airway,
                false,
            ),
            (
                NavProc::Sid(Sid {
                    airport: "AAAA".try_into().unwrap(),
                    identifier: "A1".try_into().unwrap(),
                    legs: published(LegKind::Sid),
                }),
                LegKind::Sid,
                false,
            ),
            (
                NavProc::Star(Star {
                    airport: "BBBB".try_into().unwrap(),
                    identifier: "A1".try_into().unwrap(),
                    legs: published(LegKind::Star),
                }),
                LegKind::Star,
                false,
            ),
            (
                NavProc::UnknownAirway("A1".to_owned()),
                LegKind::Airway,
                true,
            ),
            (NavProc::UnknownSid("A1".to_owned()), LegKind::Sid, true),
            (NavProc::UnknownStar("A1".to_owned()), LegKind::Star, true),
        ];
        for (procedure, kind, is_unknown) in cases {
            let candidates = [
                vec![IdentCandidate::Fix(from.clone())],
                vec![IdentCandidate::Leg(procedure)],
                vec![IdentCandidate::Fix(to.clone())],
            ];
            let resolved = LexGrouper::new(Lexer::new("AAAA A1 BBBB").parse_all().collect())
                .parse()
                .zip(candidates)
                .map(|(ident, candidates)| IdentWithCandidate { ident, candidates })
                .collect();
            let constructed =
                Constructor::new(Solver::new(resolved).solve().into_iter().collect()).construct();
            assert_eq!(constructed.len(), 1);
            assert_eq!(constructed[0].leg.kind, kind);
            assert_eq!(constructed[0].leg.is_unknown, is_unknown);
            let expanded = Expander::new(constructed).expand().unwrap();
            let expected_segments = if kind == LegKind::Direct || is_unknown {
                1
            } else {
                2
            };
            assert_eq!(expanded.len(), expected_segments);
            assert_eq!(expanded.first().unwrap().from, from);
            assert_eq!(expanded.last().unwrap().to, to);
            for leg in expanded {
                assert_eq!(leg.kind, kind);
                let dto = FlightRouteLeg::from(leg);
                assert_eq!(dto.is_unknown, is_unknown);
                assert_eq!(dto.is_sid, kind == LegKind::Sid);
                assert_eq!(dto.is_star, kind == LegKind::Star);
            }
        }
    }
}
