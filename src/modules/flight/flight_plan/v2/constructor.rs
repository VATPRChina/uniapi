use super::{CandidateWithState, IdentCandidate, SolvedIdent};
use crate::modules::navdata::models::{DirectionRestriction, NavProc, ResolvedLeg};

type PathEntry<'a, 'c, 's> = (&'a SolvedIdent<'c, 's>, &'a CandidateWithState<'c>);

/// A logical route leg and the exact procedure selected by the solver.
/// Direct connections have no procedure; named procedures retain their loaded legs.
/// Procedures are borrowed from the resolved candidates, not copied into each leg.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstructedLeg<'c> {
    pub leg: ResolvedLeg,
    pub procedure: Option<&'c NavProc>,
}

pub struct Constructor<'c, 's> {
    idents: Vec<SolvedIdent<'c, 's>>,
}

impl<'c, 's> Constructor<'c, 's> {
    pub fn new(idents: Vec<SolvedIdent<'c, 's>>) -> Self {
        Self { idents }
    }

    /// Construct the first complete surviving path in solver order, following
    /// saved predecessor indices rather than selecting each entry independently.
    /// Empty or incomplete input yields no legs. Named connections retain their
    /// route identifier and selected procedure for subsequent expansion.
    /// ResolvedLeg cannot represent the parsed speed/level or flight-rule amendments.
    pub fn construct(&self) -> Vec<ConstructedLeg<'c>> {
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

    fn path(&self, candidate_index: usize) -> Option<Vec<PathEntry<'_, 'c, 's>>> {
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

fn construct_path<'c>(path: &[PathEntry<'_, 'c, '_>]) -> Option<Vec<ConstructedLeg<'c>>> {
    if !matches!(path.first()?.1.candidate, IdentCandidate::Fix(_)) {
        return None;
    }
    let fixes: Vec<_> = path
        .iter()
        .enumerate()
        .filter_map(|(index, (_, candidate))| match candidate.candidate {
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
                [(ident, candidate)] => match candidate.candidate {
                    IdentCandidate::Leg(NavProc::Direct) => (None, None),
                    IdentCandidate::Leg(leg) => {
                        (Some(ident.ident.identifier().to_owned()), Some(leg))
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
                    is_unknown: procedure.is_some_and(NavProc::is_unknown),
                    is_sid: matches!(procedure, Some(NavProc::Sid(_) | NavProc::UnknownSid(_))),
                    is_star: matches!(procedure, Some(NavProc::Star(_) | NavProc::UnknownStar(_))),
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
    use crate::modules::flight::flight_plan::v2::{
        Expander, IdentWithCandidate, Lexer, Parser, Solver,
    };
    use crate::modules::navdata::models::{Airway, AnyFix, GeoPoint};

    #[test]
    fn reconstructs_a_known_predecessor_path_instead_of_a_recovery_path() {
        let from = AnyFix::GeoPoint(GeoPoint::new(30., 110.));
        let via = AnyFix::GeoPoint(GeoPoint::new(30., 111.));
        let to = AnyFix::GeoPoint(GeoPoint::new(30., 112.));
        let published: Vec<_> = [(from.clone(), via.clone()), (via, to.clone())]
            .into_iter()
            .map(|(from, to)| ResolvedLeg {
                from,
                to,
                identifier: Some("A1".to_owned()),
                is_unknown: false,
                is_sid: false,
                is_star: false,
                direction_restriction: DirectionRestriction::None,
            })
            .collect();
        let candidates = [
            vec![IdentCandidate::Fix(from)],
            vec![
                IdentCandidate::Leg(NavProc::UnknownAirway("A1".to_owned())),
                IdentCandidate::Leg(NavProc::Airway(Airway {
                    identifier: "A1".try_into().unwrap(),
                    legs: published.clone(),
                })),
            ],
            vec![IdentCandidate::Fix(to)],
        ];
        let resolved: Vec<_> = Parser::new(Lexer::new("AAAA A1 BBBB").parse_all().collect())
            .parse()
            .zip(candidates)
            .map(|(ident, candidates)| IdentWithCandidate { ident, candidates })
            .collect();
        let constructed = Constructor::new(Solver::new(&resolved).solve()).construct();
        assert_eq!(constructed.len(), 1);
        assert!(!constructed[0].leg.is_unknown);
        assert!(matches!(constructed[0].procedure, Some(NavProc::Airway(_))));
        assert_eq!(Expander::new(constructed).expand().unwrap(), published);
    }
}
