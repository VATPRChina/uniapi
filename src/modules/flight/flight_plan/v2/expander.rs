use itertools::Itertools;

use super::parser::{ParsedLeg, ParsedRoute};
use crate::modules::navdata::models::{
    AnyFix, DirectionRestriction, Fix, ProcedureKind, ProcedureSegment, ResolvedLeg,
};
use crate::modules::navdata::service::{InvalidNavdataError, NavdataService};

#[derive(Debug, thiserror::Error)]
pub enum RouteExpanderError {
    #[error("navdata error: {0}")]
    Navdata(#[from] InvalidNavdataError),
    #[error("no connected sequence found for {0}")]
    MissingPath(String),
    #[error("unsupported procedure route type: {0}")]
    UnsupportedRouteType(String),
}

type ExpansionResult<T> = Result<T, RouteExpanderError>;

pub struct RouteExpander<'s> {
    navdata: &'s NavdataService,
    legs: Vec<ParsedLeg>,
    departure_runway: Option<String>,
    arrival_runway: Option<String>,
}

impl<'s> RouteExpander<'s> {
    pub fn new(navdata: &'s NavdataService, legs: Vec<ParsedLeg>) -> Self {
        Self {
            navdata,
            legs,
            departure_runway: None,
            arrival_runway: None,
        }
    }

    /// Runway identifiers use the navdata form, for example `RW11`.
    pub fn with_departure_runway(mut self, runway: impl Into<String>) -> Self {
        self.departure_runway = Some(runway.into());
        self
    }

    pub fn with_arrival_runway(mut self, runway: impl Into<String>) -> Self {
        self.arrival_runway = Some(runway.into());
        self
    }

    pub async fn expand(&self) -> ExpansionResult<Vec<ResolvedLeg>> {
        let mut result = Vec::new();
        for parsed in &self.legs {
            let expanded = match &parsed.route {
                ParsedRoute::Direct => vec![parsed.leg.clone()],
                ParsedRoute::Airway(identifier) => {
                    self.expand_airway(&parsed.leg, identifier).await?
                }
                ParsedRoute::Sid(airport, identifier) => {
                    self.expand_procedure(
                        &parsed.leg,
                        ProcedureKind::Sid,
                        airport,
                        identifier,
                        self.departure_runway.as_deref(),
                    )
                    .await?
                }
                ParsedRoute::Star(airport, identifier) => {
                    self.expand_procedure(
                        &parsed.leg,
                        ProcedureKind::Star,
                        airport,
                        identifier,
                        self.arrival_runway.as_deref(),
                    )
                    .await?
                }
            };
            result.extend(expanded);
        }
        Ok(result)
    }

    async fn expand_airway(
        &self,
        leg: &ResolvedLeg,
        identifier: &str,
    ) -> ExpansionResult<Vec<ResolvedLeg>> {
        let missing = || RouteExpanderError::MissingPath(identifier.to_owned());
        let from = leg.from.identifier().ok_or_else(missing)?;
        let to = leg.to.identifier().ok_or_else(missing)?;
        if same_fix(&leg.from, &leg.to) {
            return Ok(Vec::new());
        }
        let legs = self
            .navdata
            .list_airway_legs_between(identifier, from, to)
            .await?;
        let legs = if legs
            .first()
            .is_some_and(|first| same_fix(&first.from, &leg.to))
        {
            legs.into_iter()
                .rev()
                .map(ResolvedLeg::into_reversed)
                .collect()
        } else {
            legs
        };
        if !legs
            .first()
            .is_some_and(|first| same_fix(&first.from, &leg.from))
            || !legs.last().is_some_and(|last| same_fix(&last.to, &leg.to))
            || !legs
                .windows(2)
                .all(|pair| same_fix(&pair[0].to, &pair[1].from))
        {
            return Err(missing());
        }
        Ok(legs)
    }

    async fn expand_procedure(
        &self,
        leg: &ResolvedLeg,
        kind: ProcedureKind,
        airport: &str,
        identifier: &str,
        runway: Option<&str>,
    ) -> ExpansionResult<Vec<ResolvedLeg>> {
        let segments = self
            .navdata
            .list_procedure_segments(kind, airport, identifier)
            .await?;
        let paths = procedure_paths(&segments, runway)?;
        let mut matches = paths
            .into_iter()
            .filter_map(|points| trim_procedure(points, leg, kind, airport))
            .map(|points| make_legs(points, identifier));
        let first = matches
            .next()
            .ok_or_else(|| RouteExpanderError::MissingPath(identifier.to_owned()))?;
        if matches.next().is_some() {
            return Ok(vec![leg.clone()]);
        }
        Ok(first)
    }
}

// DFD route types: conventional, RNAV, FMS, and profile-descent families.
// Each family's 0/1/2 phases are already in flight order for both SIDs and STARs.
fn phase(route_type: &str) -> ExpansionResult<(u8, usize)> {
    match route_type {
        "1" => Ok((0, 0)),
        "2" => Ok((0, 1)),
        "3" => Ok((0, 2)),
        "4" => Ok((1, 0)),
        "5" => Ok((1, 1)),
        "6" => Ok((1, 2)),
        "F" => Ok((2, 0)),
        "M" => Ok((2, 1)),
        "S" => Ok((2, 2)),
        "7" => Ok((3, 0)),
        "8" => Ok((3, 1)),
        "9" => Ok((3, 2)),
        "0" => Ok((4, 1)),
        "T" => Ok((5, 0)),
        "V" => Ok((5, 2)),
        _ => Err(RouteExpanderError::UnsupportedRouteType(
            route_type.to_owned(),
        )),
    }
}

fn procedure_paths(
    segments: &[ProcedureSegment],
    runway: Option<&str>,
) -> ExpansionResult<Vec<Vec<AnyFix>>> {
    let classified = segments
        .iter()
        .map(|segment| Ok((phase(&segment.route_type)?, segment)))
        .collect::<ExpansionResult<Vec<_>>>()?;
    let mut paths = Vec::new();
    for family in classified.iter().map(|((family, _), _)| *family).unique() {
        let stages: Vec<Vec<Option<&ProcedureSegment>>> = (0..3)
            .map(|stage| {
                let candidates: Vec<_> = classified
                    .iter()
                    .filter(|((f, s), _)| *f == family && *s == stage)
                    .map(|(_, segment)| Some(*segment))
                    .collect();
                if candidates.is_empty() {
                    vec![None]
                } else {
                    candidates
                }
            })
            .collect();
        for combination in stages.into_iter().multi_cartesian_product() {
            let segments: Vec<_> = combination.into_iter().flatten().collect();
            let runways: Vec<_> = segments
                .iter()
                .map(|s| s.transition.as_str())
                .filter(|s| s.starts_with("RW") && *s != "RWALL")
                .unique()
                .collect();
            if runways.len() > 1
                || runway.is_some_and(|requested| runways.iter().any(|r| *r != requested))
            {
                continue;
            }
            let mut points = Vec::new();
            for fix in segments.iter().flat_map(|segment| &segment.fixes) {
                if !points.last().is_some_and(|last| same_fix(last, fix)) {
                    points.push(fix.clone());
                }
            }
            if !points.is_empty() {
                paths.push(points);
            }
        }
    }
    Ok(paths)
}

fn trim_procedure(
    mut points: Vec<AnyFix>,
    leg: &ResolvedLeg,
    kind: ProcedureKind,
    airport: &str,
) -> Option<Vec<AnyFix>> {
    let starts_at_airport =
        matches!(&leg.from, AnyFix::Airport(a) if a.identifier.as_str() == airport);
    let ends_at_airport = matches!(&leg.to, AnyFix::Airport(a) if a.identifier.as_str() == airport);
    if kind == ProcedureKind::Sid && starts_at_airport {
        points.insert(0, leg.from.clone());
    }
    if kind == ProcedureKind::Star && ends_at_airport {
        points.push(leg.to.clone());
    }
    let start = points.iter().position(|point| same_fix(point, &leg.from))?;
    let end = points
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, point)| same_fix(point, &leg.to))?
        .0;
    let mut selected = points[start..=end].to_vec();
    *selected.first_mut()? = leg.from.clone();
    *selected.last_mut()? = leg.to.clone();
    Some(selected)
}

fn make_legs(points: Vec<AnyFix>, identifier: &str) -> Vec<ResolvedLeg> {
    points
        .into_iter()
        .tuple_windows()
        .filter(|(from, to)| !same_fix(from, to))
        .map(|(from, to)| ResolvedLeg {
            from,
            to,
            identifier: Some(identifier.to_owned()),
            direction_restriction: DirectionRestriction::Forward,
        })
        .collect()
}

fn same_fix(left: &AnyFix, right: &AnyFix) -> bool {
    left.identifier() == right.identifier()
        && (left.latitude() - right.latitude()).abs() <= 1.0 / 3600.0
        && (left.longitude() - right.longitude()).abs() <= 1.0 / 3600.0
}

#[cfg(test)]
mod tests {
    use super::super::{Lexer, Resolver, RouteParser};
    use super::*;

    async fn navdata() -> NavdataService {
        NavdataService::with_preferred_routes_path(
            "data/ng_jeppesen_fwdfd_2401.s3db?mode=ro",
            "assets/test/routes.csv",
        )
        .await
        .unwrap()
    }

    async fn parsed(navdata: &NavdataService, route: &str) -> Vec<ParsedLeg> {
        let tokens = Resolver::new(navdata, Lexer::new(route).parse_all())
            .resolve_tokens()
            .await
            .unwrap();
        RouteParser::new(tokens).parse().unwrap()
    }

    fn identifiers(legs: &[ResolvedLeg]) -> Vec<&str> {
        std::iter::once(legs.first().unwrap().from.identifier().unwrap())
            .chain(legs.iter().map(|leg| leg.to.identifier().unwrap()))
            .collect()
    }

    #[tokio::test]
    async fn expands_airways_in_both_directions() {
        let navdata = navdata().await;
        for (route, expected) in [
            (
                "NEFTU A301 ALPEN",
                vec!["NEFTU", "EMABU", "IMADI", "SAVEM", "MLY", "ALPEN"],
            ),
            (
                "ALPEN A301 NEFTU",
                vec!["ALPEN", "MLY", "SAVEM", "IMADI", "EMABU", "NEFTU"],
            ),
        ] {
            let input = parsed(&navdata, route).await;
            let expanded = RouteExpander::new(&navdata, input).expand().await.unwrap();
            assert_eq!(identifiers(&expanded), expected);
            assert!(
                expanded
                    .iter()
                    .all(|leg| leg.identifier.as_deref() == Some("A301"))
            );
            assert!(
                expanded
                    .windows(2)
                    .all(|legs| same_fix(&legs[0].to, &legs[1].from))
            );
        }
    }

    #[tokio::test]
    async fn expands_sid_star_and_enroute_transition() {
        let navdata = navdata().await;
        for (route, expected) in [
            ("MDLR CHUM2K CHUMA", vec!["MDLR", "LR100", "LR101", "CHUMA"]),
            ("BEROS BERO1B MMIO", vec!["BEROS", "OMEVO", "VOLIK", "MMIO"]),
            ("MDPP POKIL1 PISOR", vec!["MDPP", "POKIL", "PISOR"]),
        ] {
            let input = parsed(&navdata, route).await;
            let expanded = RouteExpander::new(&navdata, input).expand().await.unwrap();
            assert_eq!(identifiers(&expanded), expected, "{route}");
        }
    }

    #[tokio::test]
    async fn leaves_ambiguous_procedure_unchanged_and_continues() {
        let navdata = navdata().await;
        let input = parsed(&navdata, "MKJP ALPEN3 ALPEN A301 NEFTU").await;
        let original = input[0].leg.clone();
        let expanded = RouteExpander::new(&navdata, input).expand().await.unwrap();
        assert_eq!(expanded[0], original);
        assert_eq!(
            identifiers(&expanded[1..]),
            ["ALPEN", "MLY", "SAVEM", "IMADI", "EMABU", "NEFTU"]
        );
        let input = parsed(&navdata, "MKJP ALPEN3 ALPEN").await;
        let expanded = RouteExpander::new(&navdata, input)
            .with_departure_runway("RW12")
            .expand()
            .await
            .unwrap();
        assert!(
            expanded
                .iter()
                .all(|leg| leg.direction_restriction == DirectionRestriction::Forward)
        );
    }

    #[tokio::test]
    async fn preserves_direct_unknown_and_coordinate_legs() {
        let navdata = navdata().await;
        let input = parsed(&navdata, "MBAC UNKNOWN 18N066W DCT VP001").await;
        let expected: Vec<_> = input.iter().map(|parsed| parsed.leg.clone()).collect();
        navdata.db.close().await;
        assert_eq!(
            RouteExpander::new(&navdata, input).expand().await.unwrap(),
            expected
        );
        assert!(
            RouteExpander::new(&navdata, Vec::new())
                .expand()
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn rejects_missing_airway_endpoint_and_invalid_runway() {
        let navdata = navdata().await;
        let input = parsed(&navdata, "NEFTU A301 MBAC").await;
        assert!(matches!(
            RouteExpander::new(&navdata, input).expand().await,
            Err(RouteExpanderError::MissingPath(_))
        ));
        let input = parsed(&navdata, "MDLR CHUM2K CHUMA").await;
        assert!(matches!(
            RouteExpander::new(&navdata, input)
                .with_departure_runway("RW99")
                .expand()
                .await,
            Err(RouteExpanderError::MissingPath(_))
        ));
    }
}
