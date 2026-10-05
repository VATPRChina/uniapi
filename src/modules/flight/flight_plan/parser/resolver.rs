use futures::{StreamExt, TryStreamExt, stream};

use crate::modules::flight::flight_plan::parser::{Ident, Lexer, LexerTokenValue};
use crate::modules::navdata::models::{AnyFix, GeoPoint, NavProc};
use crate::modules::navdata::service::{InvalidNavdataError, NavdataResult, NavdataService};

pub struct CandidateResolver<'s> {
    idents: Vec<Ident<'s>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct IdentWithCandidate<'s> {
    pub(super) ident: Ident<'s>,
    pub(super) candidates: Vec<IdentCandidate>,
}

/// An identifier can denote either a physical fix or a connecting leg.
#[derive(Debug, Clone, PartialEq)]
pub enum IdentCandidate {
    Fix(AnyFix),
    Leg(NavProc),
}

impl IdentCandidate {
    pub fn is_unknown(&self) -> bool {
        match self {
            IdentCandidate::Fix(fix_candidate) => fix_candidate.is_unknown(),
            IdentCandidate::Leg(leg_candidate) => leg_candidate.is_unknown(),
        }
    }
}

impl<'s> CandidateResolver<'s> {
    pub fn new(idents: Vec<Ident<'s>>) -> Self {
        Self { idents }
    }

    /// Resolve all candidates in route order without choosing between ambiguous matches.
    /// Plain identifiers always include unknown route/point fallbacks.
    /// Database failures propagate instead of being treated as unknown identifiers.
    ///
    /// # Panics
    /// Panics if a standalone speed/altitude or flight-rule token reaches resolution.
    pub async fn resolve_candidates(
        self,
        navdata: &NavdataService,
    ) -> NavdataResult<impl Iterator<Item = IdentWithCandidate<'s>>> {
        let resolved: Vec<_> = stream::iter(self.idents)
            .then(|ident| async move {
                let candidates = resolve_ident(navdata, ident.identifier()).await?;
                Ok::<_, InvalidNavdataError>(IdentWithCandidate { ident, candidates })
            })
            .try_collect()
            .await?;
        Ok(resolved.into_iter())
    }
}

async fn resolve_ident(
    navdata: &NavdataService,
    ident: &str,
) -> NavdataResult<Vec<IdentCandidate>> {
    let Some(token) = Lexer::new(ident).parse_all().next() else {
        return Ok(Vec::new());
    };
    match token.value {
        LexerTokenValue::Direct => Ok(vec![IdentCandidate::Leg(NavProc::Direct)]),
        LexerTokenValue::Geo { lat, lon } => {
            Ok(vec![IdentCandidate::Fix(AnyFix::GeoPoint(GeoPoint {
                latitude: lat,
                longitude: lon,
            }))])
        }
        LexerTokenValue::IdentifierReference { ident, .. } => {
            let fixes = find_fix_candidates(navdata, ident).await?;
            if fixes.is_empty() {
                Ok(vec![IdentCandidate::Fix(AnyFix::Unknown(
                    ident.to_string(),
                ))])
            } else {
                Ok(fixes.into_iter().map(IdentCandidate::Fix).collect())
            }
        }

        LexerTokenValue::Identifier => {
            let (routes, fixes) = tokio::try_join!(
                find_route_candidates(navdata, ident),
                find_fix_candidates(navdata, ident),
            )?;
            Ok(routes
                .into_iter()
                .map(IdentCandidate::Leg)
                .chain(fixes.into_iter().map(IdentCandidate::Fix))
                .chain([
                    IdentCandidate::Leg(NavProc::UnknownSid(ident.to_string())),
                    IdentCandidate::Leg(NavProc::UnknownStar(ident.to_string())),
                    IdentCandidate::Leg(NavProc::UnknownAirway(ident.to_string())),
                    IdentCandidate::Fix(AnyFix::Unknown(ident.to_string())),
                ])
                .collect())
        }
        LexerTokenValue::SpeedAndAltitude { .. } | LexerTokenValue::Vfr | LexerTokenValue::Ifr => {
            panic!("standalone amendment token reached candidate resolution: {ident}")
        }
    }
}

async fn find_route_candidates(
    navdata: &NavdataService,
    ident: &str,
) -> NavdataResult<Vec<NavProc>> {
    let (airway, sids, stars) = tokio::try_join!(
        navdata.find_airway(ident),
        navdata.find_sids(ident),
        navdata.find_stars(ident),
    )?;
    Ok((airway.into_iter().map(NavProc::Airway))
        .chain(sids.into_iter().map(NavProc::Sid))
        .chain(stars.into_iter().map(NavProc::Star))
        .collect())
}

async fn find_fix_candidates(navdata: &NavdataService, ident: &str) -> NavdataResult<Vec<AnyFix>> {
    let (airport, vhf, enroute_ndb, terminal_ndb, enroute_waypoint, terminal_waypoint) = tokio::try_join!(
        navdata.find_airport(ident),
        navdata.find_vhf(ident),
        navdata.find_enroute_ndb(ident),
        navdata.find_terminal_ndb(ident),
        navdata.find_enroute_waypoint(ident),
        navdata.find_terminal_waypoint(ident),
    )?;
    Ok((airport.into_iter().map(AnyFix::Airport))
        .chain(vhf.into_iter().map(AnyFix::Vhf))
        .chain(enroute_ndb.into_iter().map(AnyFix::Ndb))
        .chain(terminal_ndb.into_iter().map(AnyFix::Ndb))
        .chain(enroute_waypoint.into_iter().map(AnyFix::Waypoint))
        .chain(terminal_waypoint.into_iter().map(AnyFix::Waypoint))
        .collect())
}
