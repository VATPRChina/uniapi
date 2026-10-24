use super::{ValidatorError, checks::*, validate_flight};
use crate::modules::flight::flight_plan::matcher::SegmentMatchReport;
use crate::modules::flight::flight_plan::{
    PreferredRouteMatcher, RouteSegment, RouteSegmenter, Segment, SegmentKind,
};
use crate::modules::flight::models::{
    Flight, ParsedRoute, ValidationScope, ValidatorResult, ValidatorStatus,
};
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
        let mut segment_results = validate_segment(
            flight,
            original.route,
            segment_index,
            selected.map_or(original.route, |candidate| candidate.coverage),
            selected.map(|candidate| &candidate.preferred),
            &report.preferred_routes,
            navdata,
        );
        // International portions retain scoped entries for display, but none of
        // their segment or leg checks should contribute validation warnings.
        if original.kind == SegmentKind::International {
            for result in &mut segment_results {
                result.status = ValidatorStatus::Suppressed;
                result.warnings.clear();
            }
        }
        results.extend(segment_results);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::flight::dto::TemporaryFlightQuery;
    use crate::modules::navdata::models::{
        AnyFix, DirectionRestriction, LegKind, ResolvedLeg, Waypoint, WaypointKind,
    };
    use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

    fn leg(region: &str, from: &str, to: &str) -> ResolvedLeg {
        let fix = |identifier: &str| {
            AnyFix::Waypoint(Waypoint {
                icao_code: region.try_into().unwrap(),
                identifier: identifier.try_into().unwrap(),
                latitude: 35.0.try_into().unwrap(),
                longitude: 140.0.try_into().unwrap(),
                kind: WaypointKind::Enroute,
            })
        };
        ResolvedLeg {
            from: fix(from),
            to: fix(to),
            identifier: Some("V1".into()),
            is_unknown: true,
            kind: LegKind::Airway,
            direction_restriction: DirectionRestriction::Backward,
        }
    }

    #[tokio::test]
    async fn international_segments_suppress_all_scoped_checks() {
        let directory = std::env::temp_dir().join(format!("validation-{}", ulid::Ulid::new()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("navdata.db");
        let db = SqlitePool::connect_with(
            SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(true),
        )
        .await
        .unwrap();
        db.close().await;
        let navdata = NavdataService::with_preferred_routes_path(path.to_str().unwrap(), &path)
            .await
            .unwrap();
        let flight = Flight::from(
            serde_json::from_value::<TemporaryFlightQuery>(serde_json::json!({
                "departure": "ZBAA", "arrival": "RJTT", "aircraft": "B738",
                "flight_rules": "I", "wake_category": "M"
            }))
            .unwrap(),
        );
        let route = ParsedRoute {
            legs: vec![
                leg("ZB", "A", "B"),
                leg("RJ", "C", "D"),
                leg("RJ", "D", "E"),
            ],
        };
        let segments = RouteSegmenter::new(navdata.china_boundaries()).segment(&route.legs);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].kind, SegmentKind::Mainland);
        assert_eq!(segments[1].kind, SegmentKind::International);
        let reports = segments
            .iter()
            .map(|_| SegmentMatchReport {
                candidates: vec![],
                selected_candidate_index: None,
                preferred_routes: vec![],
            })
            .collect::<Vec<_>>();
        let results = validate_analyzed_route(&flight, &route, &segments, &reports, &navdata);
        let flight_results = validate_flight(
            &ValidationContext {
                flight: &flight,
                route: &route,
                preferred_route: None,
                preferred_routes: &[],
            },
            &navdata,
        );
        assert_eq!(&results[..flight_results.len()], flight_results);
        let mut offset = flight_results.len();
        for (index, segment) in segments.iter().enumerate() {
            let mut expected = validate_segment(
                &flight,
                segment.route,
                index,
                segment.route,
                None,
                &[],
                &navdata,
            );
            // Ensure the fixture exercises warnings which must be removed.
            assert!(expected.iter().any(|result| {
                result.status == ValidatorStatus::Rejected && !result.warnings.is_empty()
            }));
            if segment.kind == SegmentKind::International {
                for result in &mut expected {
                    result.status = ValidatorStatus::Suppressed;
                    result.warnings.clear();
                }
            }
            assert_eq!(results[offset..offset + expected.len()], expected);
            offset += expected.len();
        }
        assert_eq!(results.len(), offset);

        // Also cover the public pipeline when the entire route is international.
        let international_route = ParsedRoute {
            legs: route.legs[1..].to_vec(),
        };
        let results = validate_route(&flight, &international_route, &navdata)
            .await
            .unwrap();
        let scoped = results
            .iter()
            .filter(|result| result.scope != ValidationScope::Flight)
            .collect::<Vec<_>>();
        assert_eq!(scoped.len(), 10); // Two segment checks and four per leg.
        assert!(scoped.iter().all(|result| {
            result.status == ValidatorStatus::Suppressed && result.warnings.is_empty()
        }));
        drop(navdata);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
