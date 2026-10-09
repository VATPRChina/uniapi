use std::collections::HashSet;

use arrayvec::ArrayString;
use futures::{FutureExt, future::OptionFuture};
use itertools::Itertools;
use sqlx::FromRow;

use super::{InvalidNavdataError, NavdataResult, NavdataService};
use crate::modules::navdata::models::TerminalProcedure;

impl NavdataService {
    /// Load exact and abbreviated SID/STAR identifiers, skipping invalid procedures.
    /// Exact matches precede six-character aliases for published names longer than six.
    pub async fn find_procedure_by_ident(
        &self,
        ident: &str,
        mode: FindProcedureMode,
    ) -> NavdataResult<Vec<TerminalProcedure>> {
        let primary = abbreviations(ident);
        let alternative = abbreviations_alternative(ident);

        let (mut results, extra) = tokio::try_join!(
            self.find_procedures(
                "procedure_identifier",
                primary.as_deref().unwrap_or(ident),
                mode,
            ),
            OptionFuture::from(
                alternative
                    .as_deref()
                    .map(|ident| { self.find_procedures("procedure_identifier", ident, mode) })
            )
            .map(Option::transpose),
        )?;

        results.extend(extra.into_iter().flatten());
        Ok(results)
    }

    /// Load all valid procedures at an airport, ordered by procedure identifier.
    pub async fn find_procedure_by_airport(
        &self,
        airport: &str,
        mode: FindProcedureMode,
    ) -> NavdataResult<Vec<TerminalProcedure>> {
        self.find_procedures("airport_identifier", airport, mode)
            .await
    }

    async fn find_procedures(
        &self,
        column: &'static str,
        value: &str,
        mode: FindProcedureMode,
    ) -> NavdataResult<Vec<TerminalProcedure>> {
        let table = match mode {
            FindProcedureMode::Sid => "tbl_pd_sids",
            FindProcedureMode::Star => "tbl_pe_stars",
        };
        let legs: Vec<ProcedureLegRecord> = sqlx::query_as(&format!(
            "SELECT airport_identifier, procedure_identifier, route_type,
                    transition_identifier, seqno, path_termination, waypoint_identifier
             FROM {table} WHERE {column} = $1
             ORDER BY airport_identifier, procedure_identifier, seqno",
        ))
        .bind(value)
        .fetch_all(&self.db)
        .await?;
        Ok(legs
            .into_iter()
            .chunk_by(|leg| (leg.airport_identifier.clone(), leg.procedure_identifier.clone()))
            .into_iter()
            .filter_map(|((airport, identifier), legs)| {
                match TerminalProcedure::try_from(ProcedureRecord { mode, legs: legs.collect() }) {
                    Ok(procedure) => Some(procedure),
                    Err(error) => {
                        tracing::warn!(airport, identifier, ?mode, %error, "ignoring invalid terminal procedure");
                        None
                    }
                }
            })
            .collect())
    }
}

fn abbreviations(ident: &str) -> Option<String> {
    if ident.len() != 7 {
        return None;
    }

    let validity_indicator = ident.chars().nth(5).unwrap();
    let route_indicator = ident.chars().nth(6).unwrap();

    if !validity_indicator.is_ascii_digit() {
        return None;
    }

    if !route_indicator.is_ascii_alphanumeric() {
        return None;
    }

    let dedup: String = ident
        .chars()
        .take(6)
        .tuple_windows()
        .flat_map(|(l, r)| if l == r { None } else { Some(l) })
        .collect();
    if dedup.len() == 4 {
        return Some(format!(
            "{}{}{}",
            dedup, validity_indicator, route_indicator
        ));
    }

    Some(format!(
        "{}{}{}",
        ident.chars().take(4).collect::<String>(),
        validity_indicator,
        route_indicator
    ))
}

#[cfg(test)]
#[test]
fn test_abbr() {
    assert_eq!(abbreviations("SASAN1"), None);
    assert_eq!(abbreviations("ALPHABETA"), None);
    assert_eq!(abbreviations("SASAN1A"), Some("SASA1A".to_string()));
    assert_eq!(abbreviations("COTTO2B"), Some("COTO2B".to_string()));
    assert_eq!(abbreviations("AKDIK6K"), Some("AKDI6K".to_string()));
}

fn abbreviations_alternative(ident: &str) -> Option<String> {
    if ident.len() != 7 {
        return None;
    }

    let chars: Vec<char> = ident.chars().collect();

    let validity_indicator = chars[5];
    let route_indicator = chars[6];

    if !validity_indicator.is_ascii_digit() {
        return None;
    }

    if !route_indicator.is_ascii_alphanumeric() {
        return None;
    }

    if chars[3] != 'I' && chars[3] != 'O' {
        return None;
    }

    Some(
        chars
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| *idx != 3)
            .map(|(_, c)| c)
            .collect(),
    )
}

#[cfg(test)]
#[test]
fn test_abbr_altn() {
    assert_eq!(abbreviations_alternative("SASAN1A"), None);
    assert_eq!(
        abbreviations_alternative("AKDIK6K"),
        Some("AKDK6K".to_string())
    );
}

#[derive(Debug, Clone, Copy)]
pub enum FindProcedureMode {
    Sid,
    Star,
}

/// The complete set of records for one airport/procedure.
struct ProcedureRecord {
    mode: FindProcedureMode,
    legs: Vec<ProcedureLegRecord>,
}

#[derive(FromRow)]
struct ProcedureLegRecord {
    airport_identifier: String,
    procedure_identifier: String,
    route_type: String,
    transition_identifier: Option<String>,
    seqno: u32,
    path_termination: Option<String>,
    waypoint_identifier: Option<String>,
}

impl TryFrom<ProcedureRecord> for TerminalProcedure {
    type Error = InvalidNavdataError;

    fn try_from(value: ProcedureRecord) -> NavdataResult<Self> {
        let first = value
            .legs
            .first()
            .ok_or(InvalidNavdataError::InternalError(
                "procedure has no records",
            ))?;
        let (runway_types, common_types, enroute_types): (&[&str], &[&str], &[&str]) =
            match value.mode {
                FindProcedureMode::Sid => (
                    &["1", "4", "F", "T"],
                    &["2", "5", "M"],
                    &["3", "6", "S", "V"],
                ),
                FindProcedureMode::Star => (
                    &["3", "6", "9", "S"],
                    &["2", "5", "8", "M"],
                    &["1", "4", "7", "F"],
                ),
            };
        let common_legs: Vec<_> = value
            .legs
            .iter()
            .filter(|leg| common_types.contains(&leg.route_type.as_str()))
            .collect();
        let runway_legs: Vec<_> = value
            .legs
            .iter()
            .filter(|leg| runway_types.contains(&leg.route_type.as_str()))
            .collect();
        let runway_transitions = runway_identifiers(
            runway_legs
                .iter()
                .copied()
                .filter_map(ProcedureLegRecord::runway),
        )?;
        let runway_transitions = if runway_transitions.is_empty() {
            runway_identifiers(
                common_legs
                    .iter()
                    .copied()
                    .filter_map(ProcedureLegRecord::runway),
            )?
        } else {
            runway_transitions
        };
        if runway_transitions.is_empty() {
            return Err(InvalidNavdataError::ProcedureMissingTransition("runway"));
        }
        let enroute_legs: Vec<_> = value
            .legs
            .iter()
            .filter(|leg| enroute_types.contains(&leg.route_type.as_str()))
            .collect();
        let (enroute_boundary, connection_boundary) = match value.mode {
            FindProcedureMode::Sid => (Boundary::Start, Boundary::End),
            FindProcedureMode::Star => (Boundary::End, Boundary::Start),
        };
        let enroute_points = branch_endpoints(&enroute_legs, enroute_boundary);
        let common_points = branch_endpoints(&common_legs, connection_boundary);
        let enroute_transitions = if enroute_points.iter().any(|leg| leg.has_fix()) {
            enroute_identifiers(&enroute_points, false)?
        } else if common_points.iter().any(|leg| leg.has_fix()) {
            enroute_identifiers(&common_points, true)?
        } else {
            let runway_points = branch_endpoints(&runway_legs, connection_boundary);
            runway_connection(&runway_points)?
        };
        Ok(Self {
            airport: first.airport_identifier.as_str().try_into()?,
            identifier: first.procedure_identifier.as_str().try_into()?,
            runway_transitions,
            enroute_transitions,
            is_rnav: value
                .legs
                .iter()
                .any(|leg| matches!(leg.route_type.as_str(), "4" | "5" | "6")),
        })
    }
}

impl ProcedureLegRecord {
    fn runway(&self) -> Option<&str> {
        let transition = self.transition_identifier.as_deref()?.trim();
        let runway = transition.strip_prefix("RW").unwrap_or(transition);
        (!runway.is_empty()).then_some(runway)
    }

    fn terminates_at_fix(&self) -> bool {
        matches!(
            self.path_termination.as_deref().unwrap_or("").trim(),
            "IF" | "TF" | "CF" | "DF" | "AF" | "RF" | "HF"
        )
    }

    fn has_fix(&self) -> bool {
        self.terminates_at_fix()
            && self
                .waypoint_identifier
                .as_deref()
                .is_some_and(|identifier| !identifier.trim().is_empty())
    }

    fn fix(&self, boundary: &'static str) -> NavdataResult<&str> {
        let path = self.path_termination.as_deref().unwrap_or("").trim();
        // DFD path/termination codes that finish at a fix, including a hold to fix.
        if !self.terminates_at_fix() {
            return Err(InvalidNavdataError::ProcedureEndpointNotFix {
                boundary,
                transition: self.transition_identifier.clone().unwrap_or_default(),
                seqno: self.seqno,
                path_termination: path.to_owned(),
            });
        }
        self.waypoint_identifier
            .as_deref()
            .map(str::trim)
            .filter(|identifier| !identifier.is_empty())
            .ok_or(InvalidNavdataError::ProcedureRecordMissingFixIdentifier)
    }
}

#[derive(Clone, Copy)]
enum Boundary {
    Start,
    End,
}

fn branch_endpoints<'a>(
    legs: &[&'a ProcedureLegRecord],
    boundary: Boundary,
) -> Vec<&'a ProcedureLegRecord> {
    legs.iter()
        .copied()
        .into_group_map_by(|leg| {
            (
                leg.route_type.as_str(),
                leg.transition_identifier.as_deref().unwrap_or(""),
            )
        })
        .into_values()
        .filter_map(|branch| match boundary {
            Boundary::Start => branch.into_iter().min_by_key(|leg| leg.seqno),
            Boundary::End => branch.into_iter().max_by_key(|leg| leg.seqno),
        })
        .collect()
}

/// Retain strict validation when a source contains any usable boundary fix.
fn enroute_identifiers(
    endpoints: &[&ProcedureLegRecord],
    common: bool,
) -> NavdataResult<HashSet<ArrayString<5>>> {
    let transitions: Vec<_> = endpoints
        .iter()
        .map(|endpoint| {
            let fix = endpoint.fix(if common {
                "common-route boundary"
            } else {
                "enroute-transition boundary"
            })?;
            if common {
                Ok(fix)
            } else {
                endpoint
                    .transition_identifier
                    .as_deref()
                    .map(str::trim)
                    .filter(|transition| !transition.is_empty())
                    .ok_or(InvalidNavdataError::ProcedureMissingTransition("enroute"))
            }
        })
        .collect::<NavdataResult<_>>()?;
    identifiers(transitions.into_iter())
}

/// A runway fallback is unambiguous only if every branch has the same boundary fix.
fn runway_connection(endpoints: &[&ProcedureLegRecord]) -> NavdataResult<HashSet<ArrayString<5>>> {
    let fixes: Vec<_> = endpoints
        .iter()
        .map(|endpoint| endpoint.fix("runway-transition boundary"))
        .collect::<NavdataResult<_>>()?;
    let first = fixes
        .first()
        .ok_or(InvalidNavdataError::ProcedureMissingTransition("enroute"))?;
    if fixes.iter().any(|fix| fix != first) {
        return Err(InvalidNavdataError::ProcedureRunwayEndpointsDiffer);
    }
    identifiers(std::iter::once(*first))
}

fn identifiers<'a, const N: usize>(
    identifiers: impl Iterator<Item = &'a str>,
) -> NavdataResult<HashSet<ArrayString<N>>> {
    Ok(identifiers
        .map(str::trim)
        .filter(|identifier| !identifier.is_empty())
        .map(ArrayString::try_from)
        .collect::<Result<_, _>>()?)
}

fn runway_identifiers<'a>(
    identifiers: impl Iterator<Item = &'a str>,
) -> NavdataResult<HashSet<ArrayString<3>>> {
    let expanded: Vec<_> = identifiers
        .filter(|identifier| !identifier.is_empty())
        .flat_map(|identifier| {
            let (number, suffixes): (&str, &[&str]) =
                if let Some(number) = identifier.strip_suffix('B') {
                    (number, &["L", "R"])
                } else if let Some(number) = identifier.strip_suffix('A') {
                    (number, &["L", "C", "R"])
                } else {
                    (identifier, &[""])
                };
            suffixes
                .iter()
                .map(move |suffix| format!("{number}{suffix}"))
        })
        .collect();
    self::identifiers(expanded.iter().map(String::as_str))
}
