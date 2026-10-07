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
