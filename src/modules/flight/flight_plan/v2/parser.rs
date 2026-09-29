//! Flight route parser
//!
//! ```text
//! route = dep seg* arr
//! dep = IDENTIFIER SPEED_AND_ALTITUDE?
//! arr = IDENTIFIER
//! seg = (IDENTIFIER | IDENTIFIER_REFERENCE | GEO) (VFR | IFR)? | DIRECT
//! ```

use crate::modules::flight::flight_plan::v2::resolver::ResolvedToken;
use crate::modules::navdata::models::{
    AnyFix, DirectionRestriction, Fix, FixReference, GeoPoint, ResolvedLeg,
};
use crate::modules::navdata::service::ResolvedIdent;

use super::lexer::LexerTokenValue;

pub struct RouteParser<'r> {
    tokens: Vec<ResolvedToken<'r>>,
}

enum ParserState {
    Fix(AnyFix),
    Leg(AnyFix, ParsedRoute),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedRoute {
    Airway(ArrayString<7>),
    Sid(ArrayString<4>, ArrayString<8>),
    Star(ArrayString<4>, ArrayString<8>),
    Direct,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedLeg {
    pub leg: ResolvedLeg,
    pub route: ParsedRoute,
}

pub type RouteParserResult<T> = Result<T, RouteParserError>;

#[derive(Debug, thiserror::Error)]
pub enum RouteParserError {
    #[error("First token in route is not a fix.")]
    FirstTokenIsNotFix,
}

impl<'r> RouteParser<'r> {
    pub fn new(tokens: Vec<ResolvedToken<'r>>) -> Self {
        Self { tokens }
    }

    pub fn parse(self) -> RouteParserResult<Vec<ParsedLeg>> {
        let mut tokens = self.tokens.into_iter();
        let Some(first) = tokens.next() else {
            return Ok(Vec::new());
        };
        let mut state = ParserState::Fix(initial_fix(first)?);
        let mut result = Vec::new();
        for token in tokens {
            let (next_state, leg) = state.advance(token);
            state = next_state;
            result.extend(leg);
        }
        Ok(result)
    }
}

// FIXME: add a resolution decider to pick resolved identifier
impl ParserState {
    fn advance(self, token: ResolvedToken<'_>) -> (Self, Option<ParsedLeg>) {
        match &token.token.value {
            LexerTokenValue::SpeedAndAltitude { .. } => (self, None),
            LexerTokenValue::Direct => {
                let (fix, leg) = self.into_leg();
                (Self::Leg(fix, leg), None)
            }
            LexerTokenValue::Identifier => self.advance_identifier(token),
            LexerTokenValue::Geo { lat, lon } => {
                let (fix, _) = self.into_leg();
                finish_leg(fix, GeoPoint::new(*lat, *lon).into(), ParsedRoute::Direct)
            }
            LexerTokenValue::IdentifierReference { .. } => self.finish_at_token(token),
        }
    }

    fn advance_identifier(self, token: ResolvedToken<'_>) -> (Self, Option<ParsedLeg>) {
        if let Self::Fix(ref fix) = self
            && let Some(leg) = select_leg(&token.resolved_identifiers, fix)
        {
            let (fix, _) = self.into_leg();
            return (Self::Leg(fix, leg), None);
        }
        self.finish_at_token(token)
    }

    fn finish_at_token(self, token: ResolvedToken<'_>) -> (Self, Option<ParsedLeg>) {
        let (fix, leg) = self.into_leg();
        let identifier = token.token.str;
        let to = token_fix(token, &fix).unwrap_or_else(|| AnyFix::Unknown(identifier.to_owned()));
        finish_leg(fix, to, leg)
    }

    /// A fix without a pending leg connects directly to the next point.
    fn into_leg(self) -> (AnyFix, ParsedRoute) {
        match self {
            Self::Fix(fix) => (fix, ParsedRoute::Direct),
            Self::Leg(fix, leg) => (fix, leg),
        }
    }
}

impl ParsedRoute {
    fn from_candidate(candidate: &ResolvedIdent, fix: &AnyFix) -> Option<Self> {
        match candidate {
            ResolvedIdent::Airway(airway) => airway
                .contains_fix(fix)
                .then_some(Self::Airway(airway.identifier)),
            ResolvedIdent::Sid(airport, ident) => Some(Self::Sid(*airport, *ident)),
            ResolvedIdent::Star(airport, ident) => Some(Self::Star(*airport, *ident)),
            ResolvedIdent::Fix(_) => None,
        }
    }

    fn priority(&self) -> u8 {
        match self {
            Self::Airway(_) => 0,
            Self::Sid(_, _) => 1,
            Self::Star(_, _) => 2,
            Self::Direct => 3,
        }
    }
}

fn select_leg(candidates: &[ResolvedIdent], fix: &AnyFix) -> Option<ParsedRoute> {
    candidates
        .iter()
        .filter_map(|candidate| ParsedRoute::from_candidate(candidate, fix))
        .min_by_key(ParsedRoute::priority)
}

fn initial_fix(token: ResolvedToken<'_>) -> RouteParserResult<AnyFix> {
    let origin = GeoPoint::new(0., 0.).into();
    token_fix(token, &origin).ok_or(RouteParserError::FirstTokenIsNotFix)
}

fn token_fix(token: ResolvedToken<'_>, current: &AnyFix) -> Option<AnyFix> {
    match token.token.value {
        LexerTokenValue::Geo { lat, lon } => Some(GeoPoint::new(lat, lon).into()),
        LexerTokenValue::IdentifierReference {
            heading, distance, ..
        } => nearest_fix(token.resolved_identifiers, current)
            .map(|base| reference_fix(base, heading, distance)),
        _ => nearest_fix(token.resolved_identifiers, current),
    }
}

fn nearest_fix(candidates: Vec<ResolvedIdent>, current: &AnyFix) -> Option<AnyFix> {
    candidates
        .into_iter()
        .filter_map(ResolvedIdent::into_fix)
        .min_by_key(|fix| {
            geo_distance_ordering(
                current.latitude(),
                current.longitude(),
                fix.latitude(),
                fix.longitude(),
            )
        })
}

fn reference_fix(base: AnyFix, heading: u16, distance: u16) -> AnyFix {
    match base {
        AnyFix::Airport(fix) => FixReference::new(fix, heading, distance).into(),
        AnyFix::Vhf(fix) => FixReference::new(fix, heading, distance).into(),
        AnyFix::Ndb(fix) => FixReference::new(fix, heading, distance).into(),
        AnyFix::Waypoint(fix) => FixReference::new(fix, heading, distance).into(),
        _ => unreachable!("navdata reference candidates are named fixes"),
    }
}

fn finish_leg(from: AnyFix, to: AnyFix, leg: ParsedRoute) -> (ParserState, Option<ParsedLeg>) {
    let identifier = match &leg {
        ParsedRoute::Airway(ident) => Some(ident.to_string()),
        ParsedRoute::Sid(_, ident) | ParsedRoute::Star(_, ident) => Some(ident.to_string()),
        ParsedRoute::Direct => None,
    };
    (
        ParserState::Fix(to.clone()),
        Some(ParsedLeg {
            route: leg,
            leg: ResolvedLeg {
                from,
                to,
                identifier,
                direction_restriction: DirectionRestriction::None,
            },
        }),
    )
}

fn geo_distance_ordering(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> OrderedFloat<f64> {
    let dlat = lat2 - lat1;

    let mut dlon = lon2 - lon1;
    dlon = (dlon + 180.0).rem_euclid(360.0) - 180.0;

    let mean_lat = ((lat1 + lat2) / 2.0).to_radians();
    let x = dlon * mean_lat.cos();
    let y = dlat;

    OrderedFloat(x * x + y * y)
}

#[cfg(test)]
mod tests {
    use super::super::{lexer::Lexer, resolver::Resolver};
    use super::*;
    use crate::modules::navdata::service::NavdataService;

    async fn navdata() -> NavdataService {
        NavdataService::with_preferred_routes_path(
            "data/ng_jeppesen_fwdfd_2401.s3db?mode=ro",
            "assets/test/routes.csv",
        )
        .await
        .unwrap()
    }

    async fn parse(navdata: &NavdataService, route: &str) -> Vec<ResolvedLeg> {
        let tokens = Resolver::new(navdata, Lexer::new(route).parse_all())
            .resolve_tokens()
            .await
            .unwrap();
        RouteParser::new(tokens)
            .parse()
            .unwrap()
            .into_iter()
            .map(|parsed| parsed.leg)
            .collect::<Vec<_>>()
    }

    #[tokio::test]
    async fn parses_preloaded_airways_after_database_is_closed() {
        let navdata = navdata().await;
        let tokens = Resolver::new(&navdata, Lexer::new("ASIVO L453 ASIVO").parse_all())
            .resolve_tokens()
            .await
            .unwrap();
        navdata.db.close().await;
        drop(navdata);
        let legs = RouteParser::new(tokens)
            .parse()
            .unwrap()
            .into_iter()
            .map(|parsed| parsed.leg)
            .collect::<Vec<_>>();
        assert_eq!(legs.len(), 1);
        assert_eq!(legs[0].identifier.as_deref(), Some("L453"));
    }

    #[tokio::test]
    async fn selects_priority_and_requires_airway_connection() {
        let navdata = navdata().await;
        for (start, airway, sid, star, expected) in [
            ("ASIVO", true, true, true, Some("L453")),
            ("MBAC", true, true, true, Some("GTK2A")),
            ("MBAC", false, true, true, Some("GTK2A")),
            ("MBAC", false, false, true, Some("ANTE2D")),
            ("MBAC", true, false, false, None),
        ] {
            let route = format!("{start} ASIVO ASIVO");
            let mut tokens = Resolver::new(&navdata, Lexer::new(&route).parse_all())
                .resolve_tokens()
                .await
                .unwrap();
            // Assemble competing candidates from existing navdata, least preferred first.
            if star {
                tokens[1].resolved_identifiers.extend(
                    navdata
                        .find_airways("ANTE2D")
                        .await
                        .unwrap()
                        .into_iter()
                        .filter(|item| matches!(item, ResolvedIdent::Star(_, _))),
                );
            }
            if sid {
                tokens[1]
                    .resolved_identifiers
                    .extend(navdata.find_airways("GTK2A").await.unwrap());
            }
            if airway {
                tokens[1]
                    .resolved_identifiers
                    .extend(navdata.find_airways("L453").await.unwrap());
            }
            let legs = RouteParser::new(tokens)
                .parse()
                .unwrap()
                .into_iter()
                .map(|parsed| parsed.leg)
                .collect::<Vec<_>>();
            assert_eq!(legs[0].identifier.as_deref(), expected, "start: {start}");
            assert_eq!(legs[0].to.identifier(), Some("ASIVO"));
        }
        let asivo = navdata.find_fixes("ASIVO").await.unwrap().remove(0);
        let airways = navdata.find_airways("L453").await.unwrap();
        let ResolvedIdent::Airway(airway) = &airways[0] else {
            panic!("expected airway");
        };
        assert!(airway.contains_fix(&asivo));
        let airport = navdata.find_fixes("MBAC").await.unwrap().remove(0);
        assert!(!airway.contains_fix(&airport));
    }

    #[tokio::test]
    async fn nearest_fix_uses_current_position_in_both_states() {
        let navdata = navdata().await;
        let legs = parse(&navdata, "18N066W VP001 20N156W DCT DCT VP001").await;
        assert_eq!(legs.len(), 3);
        assert_eq!(legs[0].to.icao_code(), Some("TJ"));
        assert_eq!(legs[2].to.icao_code(), Some("MU"));
        assert!(legs.iter().all(|leg| leg.identifier.is_none()));
        let legs = parse(&navdata, "ASIVO L453 DCT ASIVO").await;
        assert_eq!(legs.len(), 1);
        assert_eq!(legs[0].identifier.as_deref(), Some("L453"));
        let legs = parse(&navdata, "ASIVO L453 18N066W VP001").await;
        assert_eq!(legs.len(), 2);
        assert!(legs.iter().all(|leg| leg.identifier.is_none()));
        assert_eq!(legs[1].to.icao_code(), Some("TJ"));
    }

    #[tokio::test]
    async fn reference_tokens_choose_nearest_base_and_keep_leg() {
        let navdata = navdata().await;
        let base = navdata
            .find_fixes("VP001")
            .await
            .unwrap()
            .into_iter()
            .find(|fix| fix.icao_code() == Some("TJ"))
            .unwrap();
        let expected = reference_fix(base, 90, 40);
        for route in ["18N066W VP001090040", "18N066W DCT VP001090040"] {
            let legs = parse(&navdata, route).await;
            assert_eq!(legs.len(), 1);
            assert_eq!(legs[0].to, expected);
            assert!(legs[0].to.latitude().is_finite());
            assert!(legs[0].to.longitude() > -66.2452083333333);
        }
        let legs = parse(&navdata, "ASIVO L453 ASIVO090000 18N066W").await;
        assert_eq!(legs[0].identifier.as_deref(), Some("L453"));
        assert!(matches!(legs[0].to, AnyFix::FixReference(_)));
        assert!(legs[1].identifier.is_none());
    }

    #[tokio::test]
    async fn handles_empty_and_unresolved_routes() {
        let navdata = navdata().await;
        assert!(parse(&navdata, "").await.is_empty());
        let tokens = Resolver::new(&navdata, Lexer::new("UNKNOWN").parse_all())
            .resolve_tokens()
            .await
            .unwrap();
        assert!(matches!(
            RouteParser::new(tokens).parse(),
            Err(RouteParserError::FirstTokenIsNotFix)
        ));

        for (route, unknown, identifier) in [
            ("MBAC UNKNOWN MBAC", "UNKNOWN", None),
            ("MBAC DCT UNKNOWN MBAC", "UNKNOWN", None),
            ("MBAC UNKNOWN090040 MBAC", "UNKNOWN090040", None),
            ("MBAC DCT UNKNOWN090040 MBAC", "UNKNOWN090040", None),
            ("ASIVO L453 UNKNOWN MBAC", "UNKNOWN", Some("L453")),
        ] {
            let legs = parse(&navdata, route).await;
            assert_eq!(legs.len(), 2);
            assert_eq!(legs[0].to, AnyFix::Unknown(unknown.to_owned()));
            assert_eq!(legs[0].identifier.as_deref(), identifier);
            assert_eq!(legs[1].from, legs[0].to);
            assert_eq!(legs[1].to.identifier(), Some("MBAC"));
            assert!(legs[1].identifier.is_none());
        }
    }
}
