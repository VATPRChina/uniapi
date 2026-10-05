use super::ConstructedLeg;
use crate::modules::navdata::models::{NavProc, ResolvedLeg};

mod search;
use search::{find_path, same_fix};

#[derive(Debug, thiserror::Error)]
pub enum ExpansionError {
    #[error("invalid expansion coordinates on procedure data")]
    InvalidCoordinates,
}

pub struct Expander<'c> {
    route: Vec<ConstructedLeg<'c>>,
}

impl<'c> Expander<'c> {
    pub fn new(route: Vec<ConstructedLeg<'c>>) -> Self {
        Self { route }
    }

    /// Expand named legs into connected published segments, in route order.
    /// Direct legs and legs without a matching path retain their original form.
    /// Airways allow reverse traversal, flipping the direction restrictions.
    /// SID/STAR common legs retain their published direction.
    /// Procedures use fixed endpoints only; runway selection and vector geometry
    /// are not represented by the constructed route.
    pub fn expand(self) -> Result<Vec<ResolvedLeg>, ExpansionError> {
        self.route
            .into_iter()
            .map(expand_leg)
            .collect::<Result<Vec<_>, _>>()
            .map(|legs| legs.into_iter().flatten().collect())
    }
}

fn expand_leg(constructed: ConstructedLeg<'_>) -> Result<Vec<ResolvedLeg>, ExpansionError> {
    let ConstructedLeg { leg, procedure } = constructed;
    let Some(procedure) = procedure else {
        return Ok(vec![leg]);
    };
    if leg.is_unknown
        || leg.from.is_unknown()
        || leg.to.is_unknown()
        || same_fix(&leg.from, &leg.to)
    {
        return Ok(vec![leg]);
    }
    let edges = procedure.legs();
    if edges
        .iter()
        .any(|edge| edge.from.is_unknown() || edge.to.is_unknown())
    {
        return Err(ExpansionError::InvalidCoordinates);
    }
    let Some(path) = find_path(
        edges,
        matches!(procedure, NavProc::Airway(_)),
        &leg.from,
        &leg.to,
    )
    .filter(|path| !path.is_empty()) else {
        return Ok(vec![leg]);
    };
    Ok(path
        .iter()
        .enumerate()
        .map(|(index, traversal)| {
            let from = if index == 0 {
                &leg.from
            } else {
                path[index - 1].to()
            };
            let to = if index + 1 == path.len() {
                &leg.to
            } else {
                traversal.to()
            };
            traversal.resolve(from.clone(), to.clone())
        })
        .collect())
}

#[cfg(test)]
mod tests;
