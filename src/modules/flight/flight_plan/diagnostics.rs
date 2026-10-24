use super::segmenter::enroute;
use super::validator::{checks::ValidationContext, validate_analyzed_route, validate_flight};
use super::{PreferredRouteMatcher, RouteSegmenter, parse_route_with_observer};
use crate::modules::flight::dto::*;
use crate::modules::flight::models::{Flight, ParsedRoute};
use crate::modules::navdata::models::AnyFix;
use crate::modules::navdata::service::NavdataService;

/// Parse the filed plan once, retaining stage snapshots and using the exact same
/// segment matches for both diagnostics and validator suppression.
pub(crate) async fn analyze_flight(flight: Flight, navdata: &NavdataService) -> FlightV2Dto {
    let route_text = format!(
        "{} {} {}",
        flight.departure, flight.raw_route, flight.arrival
    );
    let mut snapshots = RouteParseResults::default();
    let parsed = parse_route_with_observer(navdata, &route_text, |step| {
        snapshots.observe(step, navdata.china_boundaries());
    })
    .await;
    let mut response = FlightV2Dto {
        flight_plan: flight.clone().into(),
        route_parse: snapshots,
        segments: None,
        preferred_route_matching: None,
        validations: vec![],
        errors: vec![],
    };
    let route = match parsed {
        Ok(route) => route,
        Err(error) => {
            let stage = if response
                .route_parse
                .lex_grouper
                .as_ref()
                .is_none_or(|idents| {
                    idents.is_empty() || idents.iter().any(|ident| !ident.errors.is_empty())
                }) {
                "lex-grouper"
            } else if response.route_parse.resolver.is_none() {
                "resolver"
            } else if response
                .route_parse
                .constructor
                .as_ref()
                .is_none_or(Vec::is_empty)
            {
                "constructor"
            } else {
                "expander"
            };
            response.errors.push(FlightV2DiagnosticError {
                stage: stage.into(),
                message: error.to_string(),
            });
            response.validations =
                validate_without_matching(&flight, &ParsedRoute { legs: vec![] }, navdata);
            return response;
        }
    };
    let segments = RouteSegmenter::new(navdata.china_boundaries()).segment(&route.legs);
    response.segments = Some(
        segments
            .iter()
            .map(|segment| FlightV2Segment {
                metadata: segment.into(),
                legs: segment
                    .route
                    .legs
                    .iter()
                    .map(|leg| FlightV2Leg::new(leg, navdata.china_boundaries()))
                    .collect(),
            })
            .collect(),
    );
    let matcher = PreferredRouteMatcher::new(navdata);
    let mut reports = Vec::new();
    for segment in &segments {
        match matcher.inspect_segment(segment).await {
            Ok(report) => reports.push(report),
            Err(error) => {
                response.errors.push(FlightV2DiagnosticError {
                    stage: "preferred-route-matching".into(),
                    message: error.to_string(),
                });
                response.validations = validate_without_matching(&flight, &route, navdata);
                return response;
            }
        }
    }
    response.preferred_route_matching = Some(
        reports
            .iter()
            .enumerate()
            .map(|(segment_index, report)| SegmentRouteMatches {
                segment_index,
                selected_candidate_index: report.selected_candidate_index,
                candidates: report
                    .candidates
                    .iter()
                    .map(|candidate| {
                        let coverage = candidate.coverage;
                        let from = &coverage
                            .legs
                            .first()
                            .expect("nonempty matching variant")
                            .from;
                        let to = &coverage.legs.last().expect("nonempty matching variant").to;
                        let (enroute_from, _, enroute_to) =
                            enroute(coverage.legs).expect("nonempty matching variant");
                        PreferredRouteMatchCandidate {
                            route: (&candidate.preferred).into(),
                            matched: candidate.matched,
                            error: candidate.error.clone(),
                            start_leg_index: coverage.start_index,
                            end_leg_index: coverage.end_index(),
                            from: from.into(),
                            to: to.into(),
                            enroute_start_fix: enroute_from.into(),
                            enroute_end_fix: enroute_to.into(),
                            airport_pair: matches!(from, AnyFix::Airport(_))
                                && matches!(to, AnyFix::Airport(_)),
                        }
                    })
                    .collect(),
            })
            .collect(),
    );
    response.validations = validate_analyzed_route(&flight, &route, &segments, &reports, navdata);
    response
}

fn validate_without_matching(
    flight: &Flight,
    route: &ParsedRoute,
    navdata: &NavdataService,
) -> Vec<crate::modules::flight::models::ValidatorResult> {
    let mut results = validate_flight(
        &ValidationContext {
            flight,
            route,
            preferred_route: None,
            preferred_routes: &[],
        },
        navdata,
    );
    // Directional altitude validation depends on knowing whether a preferred route matched.
    if let Some(result) = results
        .iter_mut()
        .find(|result| result.validator_ident == "cruise-altitude-directional")
    {
        result.status = crate::modules::flight::models::ValidatorStatus::Unavailable;
        result.warnings.clear();
    }
    results
}
