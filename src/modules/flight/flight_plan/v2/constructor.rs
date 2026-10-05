use super::{CandidateWithState, IdentCandidate, SolvedIdent};
use crate::modules::navdata::models::{DirectionRestriction, NavProc, ResolvedLeg};

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
            let (identifier, is_unknown, is_sid, is_star) = match &path[from_index + 1..*to_index] {
                [] => (None, false, false, false),
                [(ident, candidate)] => match &candidate.candidate {
                    IdentCandidate::Leg(NavProc::Direct) => (None, false, false, false),
                    IdentCandidate::Leg(leg) => (
                        Some(ident.ident.identifier().to_owned()),
                        leg.is_unknown(),
                        matches!(leg, NavProc::Sid(_) | NavProc::UnknownSid(_)),
                        matches!(leg, NavProc::Star(_) | NavProc::UnknownStar(_)),
                    ),
                    IdentCandidate::Fix(_) => return None,
                },
                _ => return None,
            };
            Some(ResolvedLeg {
                from: (*from).to_owned(),
                to: (*to).to_owned(),
                identifier,
                is_unknown,
                is_sid,
                is_star,
                direction_restriction: DirectionRestriction::None,
            })
        })
        .collect()
}
