use super::{ValidatorError, checks::*, validate_flight};
use crate::modules::flight::flight_plan::matcher::SegmentMatchReport;
use crate::modules::flight::flight_plan::{
    PreferredRouteMatcher, RouteSegment, RouteSegmenter, Segment,
};
use crate::modules::flight::models::{Flight, ParsedRoute, ValidationScope, ValidatorResult};
use crate::modules::navdata::models::PreferredRoute;
use crate::modules::navdata::service::NavdataService;

/// Preserve every execution, including passing and suppressed checks. Segment
/// indices refer to the segmenter's output; leg indices refer to the full route.
pub async fn validate_route(
    flight: &Flight,
    route: &ParsedRoute,
    navdata: &NavdataService,
) -> Result<Vec<ValidatorResult>, ValidatorError> {
    let matcher = PreferredRouteMatcher::new(navdata);
    let segments = RouteSegmenter::new(navdata.china_boundaries()).segment(&route.legs);
    let mut reports = Vec::new();
    for segment in &segments {
        reports.push(matcher.inspect_segment(segment).await?);
    }
    Ok(validate_analyzed_route(
        flight, route, &segments, &reports, navdata,
    ))
}

pub(crate) fn validate_analyzed_route(
    flight: &Flight,
    route: &ParsedRoute,
    segments: &[Segment<'_>],
    reports: &[SegmentMatchReport<'_>],
    navdata: &NavdataService,
) -> Vec<ValidatorResult> {
    let context = ValidationContext {
        flight,
        route,
        preferred_route: reports
            .iter()
            .find_map(|report| report.selected().map(|candidate| &candidate.preferred)),
        preferred_routes: &[],
    };
    let mut results = validate_flight(&context, navdata);
    for (segment_index, (original, report)) in segments.iter().zip(reports).enumerate() {
        let selected = report.selected();
        results.extend(validate_segment(
            flight,
            original.route,
            segment_index,
            selected.map_or(original.route, |candidate| candidate.coverage),
            selected.map(|candidate| &candidate.preferred),
            &report.preferred_routes,
            navdata,
        ));
    }
    results
}

fn validate_segment(
    flight: &Flight,
    segment: RouteSegment<'_>,
    segment_index: usize,
    coverage: RouteSegment<'_>,
    preferred_route: Option<&PreferredRoute>,
    preferred_routes: &[PreferredRoute],
    navdata: &NavdataService,
) -> Vec<ValidatorResult> {
    let route = ParsedRoute {
        legs: segment.legs.to_vec(),
    };
    let preferred_routes = preferred_routes.iter().collect::<Vec<_>>();
    let context = ValidationContext {
        flight,
        route: &route,
        preferred_route,
        preferred_routes: &preferred_routes,
    };
    let scope = ValidationScope::Segment {
        segment_index,
        start_leg_index: segment.start_index,
        end_leg_index: segment.end_index(),
    };
    let mut results = vec![
        PreferredRouteValidator::execute(&context, navdata, scope.clone()),
        PreferredRouteCruiseAltitudeValidator::execute(&context, navdata, scope),
    ];
    for (offset, leg) in segment.legs.iter().enumerate() {
        let leg_index = segment.start_index + offset;
        let route = ParsedRoute {
            legs: vec![leg.clone()],
        };
        let context = ValidationContext {
            route: &route,
            // Matching a border alternative may cover only part of a segment.
            preferred_route: preferred_route
                .filter(|_| (coverage.start_index..coverage.end_index()).contains(&leg_index)),
            ..context
        };
        let scope = ValidationScope::Leg {
            segment_index,
            leg_index,
        };
        let mut leg_results = vec![
            RouteDirectLegValidator::execute(&context, navdata, scope.clone()),
            RouteUnknownLegOrFixValidator::execute(&context, navdata, scope.clone()),
            AirwayApprovalValidator::execute(&context, navdata, scope.clone()),
            AirwayDirectionValidator::execute(&context, navdata, scope),
        ];
        for result in &mut leg_results {
            for warning in &mut result.warnings {
                if let Some(index) = warning.field_index.as_mut() {
                    *index += leg_index;
                }
            }
        }
        results.extend(leg_results);
    }
    results
}
