use super::{ValidatorError, checks::*, validate_flight_checks};
use crate::modules::flight::flight_plan::matcher::SegmentMatchReport;
use crate::modules::flight::flight_plan::{
    PreferredRouteMatcher, RouteSegment, RouteSegmenter, Segment, SegmentKind,
};
use crate::modules::flight::models::{Flight, ParsedRoute, ValidationField, ValidatorExecution};
use crate::modules::navdata::models::PreferredRoute;
use crate::modules::navdata::service::NavdataService;

/// Preserve every execution, including passing and suppressed checks. Hong Kong
/// segments run preferred-route checks; international segments skip scoped checks.
/// Segment indices refer to the segmenter's output; leg indices to the full route.
pub async fn validate_route(
    flight: &Flight,
    route: &ParsedRoute,
    navdata: &NavdataService,
) -> Result<Vec<ValidatorExecution>, ValidatorError> {
    let matcher = PreferredRouteMatcher::new(navdata);
    let segments = RouteSegmenter::new(navdata.china_boundaries())
        .with_hong_kong_boundary(navdata.hong_kong_boundary())
        .segment(&route.legs);
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
) -> Vec<ValidatorExecution> {
    let context = ValidationContext {
        flight,
        route,
        preferred_route: segments
            .iter()
            .zip(reports)
            // Hong Kong matches must not suppress the mainland directional check.
            .filter(|(segment, _)| segment.kind == SegmentKind::Mainland)
            .find_map(|(_, report)| report.selected().map(|candidate| &candidate.preferred)),
        preferred_routes: &[],
    };
    let hong_kong_only = segments
        .iter()
        .any(|segment| segment.kind == SegmentKind::HongKong)
        && !segments
            .iter()
            .any(|segment| segment.kind == SegmentKind::Mainland);
    let mut results = validate_flight_checks(&context, navdata, !hong_kong_only);
    for (original, report) in segments.iter().zip(reports) {
        if original.kind == SegmentKind::International {
            continue;
        }
        let selected = report.selected();
        results.extend(validate_segment(
            original.kind,
            flight,
            original.route,
            selected.map_or(original.route, |candidate| candidate.coverage),
            selected.map(|candidate| &candidate.preferred),
            &report.preferred_routes,
            navdata,
        ));
    }
    results
}

fn validate_segment(
    kind: SegmentKind,
    flight: &Flight,
    segment: RouteSegment<'_>,
    coverage: RouteSegment<'_>,
    preferred_route: Option<&PreferredRoute>,
    preferred_routes: &[PreferredRoute],
    navdata: &NavdataService,
) -> Vec<ValidatorExecution> {
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
    let scope = ValidationField::MultipleLegs {
        indices: segment.start_index..segment.end_index(),
    };
    let mut results = vec![PreferredRouteValidator::execute(
        &context,
        navdata,
        scope.clone(),
    )];
    if kind == SegmentKind::HongKong {
        return results;
    }
    results.push(PreferredRouteCruiseAltitudeValidator::execute(
        &context, navdata, scope,
    ));
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
        let scope = ValidationField::Leg { index: leg_index };
        let leg_results = vec![
            RouteDirectLegValidator::execute(&context, navdata, scope.clone()),
            RouteUnknownLegOrFixValidator::execute(&context, navdata, scope.clone()),
            AirwayApprovalValidator::execute(&context, navdata, scope.clone()),
            AirwayDirectionValidator::execute(&context, navdata, scope),
        ];
        results.extend(leg_results);
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::flight::dto::TemporaryFlightQuery;
    use crate::modules::flight::flight_plan::validator::validate_flight;
    use crate::modules::flight::models::ValidatorStatus;
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
    async fn international_segments_skip_all_scoped_checks() {
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
                leg("ZB", "F", "G"),
            ],
        };
        let mut segments = RouteSegmenter::new(navdata.china_boundaries())
            .with_hong_kong_boundary(navdata.hong_kong_boundary())
            .segment(&route.legs);
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].kind, SegmentKind::Mainland);
        assert_eq!(segments[1].kind, SegmentKind::International);
        assert_eq!(segments[2].kind, SegmentKind::Mainland);
        let reports = segments
            .iter()
            .map(|_| SegmentMatchReport {
                candidates: vec![],
                selected_candidate_index: None,
                preferred_routes: vec![],
            })
            .collect::<Vec<_>>();
        let results = validate_analyzed_route(&flight, &route, &segments, &reports, &navdata);
        segments[1].kind = SegmentKind::HongKong;
        let hong_kong_results =
            validate_analyzed_route(&flight, &route, &segments, &reports, &navdata);
        let hk_scope = ValidationField::MultipleLegs { indices: 1..3 };
        let hk_checks: Vec<_> = hong_kong_results
            .iter()
            .filter(|check| check.field == hk_scope)
            .collect();
        assert_eq!(hk_checks.len(), 1);
        assert!(
            hk_checks
                .iter()
                .all(|check| check.status == ValidatorStatus::Suppressed)
        );
        assert_eq!(hong_kong_results.len(), results.len() + 1);
        segments[1].kind = SegmentKind::International;
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
        for segment in &segments {
            if segment.kind == SegmentKind::International {
                continue;
            }
            let expected = validate_segment(
                segment.kind,
                &flight,
                segment.route,
                segment.route,
                None,
                &[],
                &navdata,
            );
            // Mainland warnings and original segment/leg indices remain intact.
            assert!(expected.iter().any(|result| {
                result.status == ValidatorStatus::Rejected && !result.problems.is_empty()
            }));
            assert_eq!(results[offset..offset + expected.len()], expected);
            offset += expected.len();
        }
        assert_eq!(results.len(), offset);

        // Also cover the public pipeline when the entire route is international.
        let international_route = ParsedRoute {
            legs: route.legs[1..3].to_vec(),
        };
        let results = validate_route(&flight, &international_route, &navdata)
            .await
            .unwrap();
        let expected = validate_flight(
            &ValidationContext {
                flight: &flight,
                route: &international_route,
                preferred_route: None,
                preferred_routes: &[],
            },
            &navdata,
        );
        assert_eq!(results, expected);
        drop(navdata);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
