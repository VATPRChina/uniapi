use serde::Serialize;
use utoipa::ToSchema;

use super::{FlightDto, FlightRouteFix, FlightRouteLeg, FlightRouteSegment};
use crate::modules::flight::models::ValidatorResult;
use crate::modules::navdata::models::{
    ChinaMainlandBoundary, DirectionRestriction, LegKind, PreferredRoute, ResolvedLeg,
};

/// A single flight snapshot and its v2 analysis. No v1 parsing or validation is used.
#[derive(Serialize, ToSchema)]
pub struct FlightV2Dto {
    pub flight_plan: FlightDto,
    pub route_parse: RouteParseResults,
    /// None means parsing did not complete; an empty array means no segments.
    pub segments: Option<Vec<FlightV2Segment>>,
    /// None means matching could not complete. Indices refer to `segments`.
    pub preferred_route_matching: Option<Vec<SegmentRouteMatches>>,
    pub validations: Vec<ValidatorResult>,
    pub errors: Vec<FlightV2DiagnosticError>,
}

#[derive(Serialize, ToSchema)]
pub struct FlightV2DiagnosticError {
    pub stage: String,
    pub message: String,
}

/// A null stage has not completed. Completed stages are retained on later errors.
#[derive(Default, Serialize, ToSchema)]
pub struct RouteParseResults {
    pub lexer: Option<Vec<RouteLexerToken>>,
    pub lex_grouper: Option<Vec<RouteParsedIdent>>,
    pub resolver: Option<Vec<RouteResolvedIdent>>,
    pub solver: Option<Vec<RouteSolvedIdent>>,
    pub constructor: Option<Vec<RouteConstructedLeg>>,
    pub expander: Option<Vec<FlightV2Leg>>,
}

#[derive(Serialize, ToSchema)]
pub struct RouteLexerToken {
    pub text: String,
    pub kind: String,
    pub amendments: Vec<RouteAmendment>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub reference_identifier: Option<String>,
    pub heading: Option<u16>,
    pub distance_nm: Option<u16>,
}

#[derive(Serialize, ToSchema)]
pub struct RouteAmendment {
    pub kind: String,
    pub value: String,
}

#[derive(Serialize, ToSchema)]
pub struct RouteParsedIdent {
    pub identifier: String,
    pub amendments: Vec<RouteAmendment>,
    pub errors: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct RouteCandidate {
    /// airport, waypoint, ndb, vhf, coordinate, fix-reference, unknown-fix,
    /// airway, sid, star, or direct. Unknown procedures retain their kind.
    pub kind: String,
    pub identifier: String,
    pub is_unknown: bool,
    pub fix: Option<FlightRouteFix>,
}

#[derive(Serialize, ToSchema)]
pub struct RouteResolvedIdent {
    pub ident: RouteParsedIdent,
    pub candidates: Vec<RouteCandidate>,
}

#[derive(Serialize, ToSchema)]
pub struct RouteSolvedIdent {
    pub ident: RouteParsedIdent,
    pub candidates: Vec<RouteSolverState>,
}

#[derive(Serialize, ToSchema)]
pub struct RouteSolverState {
    pub candidate: RouteCandidate,
    /// Index in the previous identifier's surviving candidates; null at departure.
    pub predecessor_index: Option<usize>,
    pub distance_nm: Option<f64>,
    pub fallback_count: usize,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Serialize, ToSchema)]
pub struct RouteConstructedLeg {
    pub leg: FlightV2Leg,
    pub procedure: Option<RouteCandidate>,
}

#[derive(Serialize, ToSchema)]
pub struct FlightV2Leg {
    #[serde(flatten)]
    pub leg: FlightRouteLeg,
    pub kind: LegKind,
    pub direction_restriction: DirectionRestriction,
}

impl FlightV2Leg {
    pub fn new(leg: &ResolvedLeg, boundary: &ChinaMainlandBoundary) -> Self {
        Self {
            kind: leg.kind,
            direction_restriction: leg.direction_restriction.clone(),
            leg: FlightRouteLeg::from_leg(leg.clone(), boundary),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct FlightV2Segment {
    #[serde(flatten)]
    pub metadata: FlightRouteSegment,
    pub legs: Vec<FlightV2Leg>,
}

#[derive(Serialize, ToSchema)]
pub struct SegmentRouteMatches {
    pub segment_index: usize,
    /// First matching candidate in matching priority order, used by validation.
    pub selected_candidate_index: Option<usize>,
    pub candidates: Vec<PreferredRouteMatchCandidate>,
}

#[derive(Serialize, ToSchema)]
pub struct PreferredRouteMatchCandidate {
    pub route: PreferredRouteSnapshot,
    pub matched: bool,
    /// Invalid published routes are reported, without preventing other matches.
    pub error: Option<String>,
    pub start_leg_index: usize,
    pub end_leg_index: usize,
    pub from: FlightRouteFix,
    pub to: FlightRouteFix,
    pub enroute_start_fix: FlightRouteFix,
    pub enroute_end_fix: FlightRouteFix,
    pub airport_pair: bool,
}

#[derive(Serialize, ToSchema)]
pub struct PreferredRouteSnapshot {
    pub name: String,
    pub departure: String,
    pub arrival: String,
    pub raw_route: String,
    pub is_public: bool,
    pub cruising_level_restriction: String,
    pub allowed_altitudes: Vec<i32>,
    pub minimal_altitude: i32,
    pub remarks: String,
    pub valid_from: Option<chrono::DateTime<chrono::Utc>>,
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<&PreferredRoute> for PreferredRouteSnapshot {
    fn from(route: &PreferredRoute) -> Self {
        Self {
            name: route.name.clone(),
            departure: route.departure.clone(),
            arrival: route.arrival.clone(),
            raw_route: route.raw_route.clone(),
            is_public: route.is_public,
            cruising_level_restriction: serde_plain::to_string(&route.cruising_level_restriction)
                .expect("enum serializes"),
            allowed_altitudes: route.allowed_altitudes.clone(),
            minimal_altitude: route.minimal_altitude,
            remarks: route.remarks.clone(),
            valid_from: route.valid_from,
            valid_until: route.valid_until,
        }
    }
}
