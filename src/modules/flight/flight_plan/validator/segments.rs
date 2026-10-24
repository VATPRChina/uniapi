use super::{ValidatorError, checks::*, validate_all};
use crate::modules::flight::flight_plan::{PreferredRouteMatcher, RouteSegment, RouteSegmenter};
use crate::modules::flight::models::{Flight, ParsedRoute, ValidatorResult, ValidatorStatus};
use crate::modules::navdata::models::PreferredRoute;
use crate::modules::navdata::service::NavdataService;

/// Match each mainland portion independently, preserving the full-flight checks
/// and original leg indices. A preferred route suppresses checks only on the
/// legs it covers; foreign and unmatched portions are still checked normally.
pub async fn validate_route(
    flight: &Flight,
    route: &ParsedRoute,
    navdata: &NavdataService,
) -> Result<Vec<ValidatorResult>, ValidatorError> {
    let matcher = PreferredRouteMatcher::new(navdata);
    let segments = RouteSegmenter::new(navdata.china_boundaries()).segment(&route.legs);
    let matches = matcher.find_matching_route(segments.clone()).await?;
    let context = ValidationContext {
        flight,
        route,
        preferred_route: matches.iter().find_map(|(_, preferred)| preferred.as_ref()),
        preferred_routes: &[],
    };
    let mut results = validate_all(&context, navdata)?;
    let mut combined = Vec::new();
    let mut cursor = 0;
    for (original, (segment, preferred)) in segments.iter().zip(&matches) {
        if cursor < segment.start_index {
            merge_part(
                &mut combined,
                validate_part(
                    flight,
                    RouteSegment {
                        legs: &route.legs[cursor..segment.start_index],
                        start_index: cursor,
                    },
                    None,
                    &[],
                    navdata,
                ),
            );
        }
        let candidates = matcher.preferred_routes(original).await?;
        merge_part(
            &mut combined,
            validate_part(flight, *segment, preferred.as_ref(), &candidates, navdata),
        );
        cursor = segment.end_index();
    }
    if cursor < route.legs.len() {
        merge_part(
            &mut combined,
            validate_part(
                flight,
                RouteSegment {
                    legs: &route.legs[cursor..],
                    start_index: cursor,
                },
                None,
                &[],
                navdata,
            ),
        );
    }
    for result in combined {
        if let Some(original) = results
            .iter_mut()
            .find(|original| original.validator_ident == result.validator_ident)
        {
            *original = result;
        }
    }
    Ok(results)
}

fn validate_part(
    flight: &Flight,
    segment: RouteSegment<'_>,
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
    let mut results = vec![
        PreferredRouteValidator::validate(&context, navdata),
        PreferredRouteCruiseAltitudeValidator::validate(&context, navdata),
        RouteDirectLegValidator::validate(&context, navdata),
        RouteUnknownLegOrFixValidator::validate(&context, navdata),
        AirwayApprovalValidator::validate(&context, navdata),
        AirwayDirectionValidator::validate(&context, navdata),
    ];
    for result in &mut results {
        for warning in &mut result.warnings {
            if let Some(index) = warning.field_index.as_mut() {
                *index += segment.start_index;
            }
        }
    }
    results
}

fn merge_part(combined: &mut Vec<ValidatorResult>, results: Vec<ValidatorResult>) {
    for result in results {
        if let Some(current) = combined
            .iter_mut()
            .find(|current| current.validator_ident == result.validator_ident)
        {
            if status_priority(&result.status) > status_priority(&current.status) {
                current.status = result.status;
            }
            for warning in result.warnings {
                if !current.warnings.contains(&warning) {
                    current.warnings.push(warning);
                }
            }
        } else {
            combined.push(result);
        }
    }
}

fn status_priority(status: &ValidatorStatus) -> u8 {
    match status {
        ValidatorStatus::Suppressed => 0,
        ValidatorStatus::Pass => 1,
        ValidatorStatus::Unavailable => 2,
        ValidatorStatus::Rejected => 3,
    }
}
