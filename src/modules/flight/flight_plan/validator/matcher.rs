use itertools::Itertools;

use crate::modules::flight::flight_plan::parse_route;
use crate::modules::flight::flight_plan::preferred_route_segment::PreferredRouteSegmentService;
use crate::modules::flight::flight_plan::validator::ValidatorError;
use crate::modules::navdata::models::{LegKind, PreferredRoute, ResolvedLeg};
use crate::modules::navdata::service::NavdataService;

pub(crate) async fn find_matching_route<'a>(
    navdata: &NavdataService,
    legs: &[ResolvedLeg],
    preferred_routes: &[&'a PreferredRoute],
) -> Result<Option<&'a PreferredRoute>, ValidatorError> {
    let Some(first) = preferred_routes.first() else {
        return Ok(None);
    };
    let selector = PreferredRouteSegmentService::new(navdata, &first.departure, &first.arrival)
        .await
        .map_err(ValidatorError::Navdata)?;
    for &preferred_route in preferred_routes
        .iter()
        .sorted_by_key(|route| if route.is_public { 0 } else { 1 })
    {
        tracing::info!(
            "checking preferred route {}: {}",
            preferred_route.name,
            preferred_route.raw_route
        );
        let parsed = parse_route(
            navdata,
            &format!(
                "{} {} {}",
                preferred_route.departure, preferred_route.raw_route, preferred_route.arrival
            ),
        )
        .await?;
        if route_matches_expected(legs, &parsed.legs, &selector) {
            return Ok(Some(preferred_route));
        }
    }
    Ok(None)
}

/// Compare the selected mainland enroute spans, aligning published procedures
/// with their entry/exit fixes in the preferred route.
fn route_matches_expected(
    actual: &[ResolvedLeg],
    expected: &[ResolvedLeg],
    selector: &PreferredRouteSegmentService,
) -> bool {
    let Ok(enroute) = selector.select(actual) else {
        // Malformed routes still need to reach the route/fix validators.
        return false;
    };
    let Ok(expected) = selector.select(expected) else {
        return false;
    };
    if enroute.is_empty() || enroute.len() != expected.len() {
        return false;
    }
    enroute
        .iter()
        .zip(&expected)
        .enumerate()
        .all(|(index, (segment, expected))| {
            enroute_matches_expected(
                segment,
                expected,
                index == 0 && actual.first().is_some_and(|leg| leg.kind == LegKind::Sid),
                index + 1 == enroute.len()
                    && actual.last().is_some_and(|leg| leg.kind == LegKind::Star),
            )
        })
}

fn enroute_matches_expected(
    enroute: &[ResolvedLeg],
    expected: &[ResolvedLeg],
    has_sid: bool,
    has_star: bool,
) -> bool {
    let (Some(first), Some(last)) = (enroute.first(), enroute.last()) else {
        return false;
    };
    // Expected routes generally omit procedures and include airport connections.
    // Only procedure ends may move the expected anchors; FIR selection has
    // already removed the foreign portion of both routes independently.
    let start = if has_sid {
        expected.iter().position(|leg| leg.from == first.from)
    } else {
        Some(0)
    };
    let end = if has_star {
        expected
            .iter()
            .rposition(|leg| leg.to == last.to)
            .map(|i| i + 1)
    } else {
        Some(expected.len())
    };
    let (Some(start), Some(end)) = (start, end) else {
        return false;
    };
    start < end
        && enroute.len() == end - start
        && enroute
            .iter()
            .zip(&expected[start..end])
            .all(|(actual, expected)| leg_matches(actual, expected))
}

fn leg_matches(actual: &ResolvedLeg, expected: &ResolvedLeg) -> bool {
    actual.identifier == expected.identifier
        && actual.from == expected.from
        && actual.to == expected.to
}

#[cfg(test)]
mod tests {
    use super::leg_matches;

    fn route_matches_expected(
        actual: &[ResolvedLeg],
        expected: &[ResolvedLeg],
        preferred: &PreferredRoute,
    ) -> bool {
        let selector = crate::modules::flight::flight_plan::preferred_route_segment::tests::fixture(
            &preferred.departure,
            &preferred.arrival,
        );
        super::route_matches_expected(actual, expected, &selector)
    }
    use crate::modules::navdata::models::{
        AnyFix, DirectionRestriction, GeoPoint, LegKind, LevelRestrictionType, PreferredRoute,
        ResolvedLeg, Waypoint, WaypointKind,
    };

    fn preferred(departure: &str, arrival: &str) -> PreferredRoute {
        PreferredRoute {
            name: "test".into(),
            departure: departure.into(),
            arrival: arrival.into(),
            raw_route: String::new(),
            cruising_level_restriction: LevelRestrictionType::Standard,
            allowed_altitudes: vec![],
            minimal_altitude: 0,
            remarks: String::new(),
            valid_from: None,
            valid_until: None,
            is_public: true,
        }
    }

    fn leg(from: (f64, f64), to: (f64, f64), kind: LegKind) -> ResolvedLeg {
        ResolvedLeg {
            from: GeoPoint::new(from.0, from.1).unwrap().into(),
            to: GeoPoint::new(to.0, to.1).unwrap().into(),
            identifier: Some("A1".into()),
            is_unknown: false,
            kind,
            direction_restriction: DirectionRestriction::None,
        }
    }

    #[test]
    fn domestic_procedure_boundaries_must_match_expected_fixes() {
        let expected = vec![
            leg((30., 110.), (31., 111.), LegKind::Direct),
            leg((31., 111.), (32., 112.), LegKind::Airway),
            leg((32., 112.), (33., 113.), LegKind::Direct),
        ];
        let mut actual = expected.clone();
        actual[0].kind = LegKind::Sid;
        actual[2].kind = LegKind::Star;
        let preferred = preferred("ZBAA", "ZSPD");
        assert!(route_matches_expected(&actual, &expected, &preferred));
        actual[1].identifier = Some("A2".into());
        assert!(!route_matches_expected(&actual, &expected, &preferred));
        assert!(!route_matches_expected(&actual[..1], &expected, &preferred));
    }

    #[test]
    fn international_matching_ignores_foreign_route_in_both_directions() {
        let expected = vec![
            leg((30., 110.), (31., 111.), LegKind::Airway),
            leg((31., 111.), (32., 112.), LegKind::Airway),
            leg((32., 112.), (35., 140.), LegKind::Direct),
        ];
        let mut actual = expected.clone();
        actual[2].to = GeoPoint::new(37., 127.).unwrap().into();
        actual.push(leg((37., 127.), (35., 140.), LegKind::Airway));
        assert!(route_matches_expected(
            &actual,
            &expected,
            &preferred("ZBAA", "RJTT")
        ));
        let reversed = |legs: Vec<ResolvedLeg>| {
            legs.into_iter()
                .rev()
                .map(ResolvedLeg::into_reversed)
                .collect::<Vec<_>>()
        };
        assert!(route_matches_expected(
            &reversed(actual.clone()),
            &reversed(expected.clone()),
            &preferred("RJTT", "ZBAA")
        ));
        actual[1].identifier = Some("A2".into());
        assert!(!route_matches_expected(
            &actual,
            &expected,
            &preferred("ZBAA", "RJTT")
        ));
    }

    #[test]
    fn international_matching_compares_every_child_segment_in_order() {
        let expected = vec![
            leg((30., 110.), (31., 111.), LegKind::Sid),
            leg((31., 111.), (32., 112.), LegKind::Airway),
            leg((32., 112.), (35., 140.), LegKind::Direct),
            leg((35., 140.), (33., 113.), LegKind::Direct),
            leg((33., 113.), (34., 114.), LegKind::Airway),
            leg((34., 114.), (35., 115.), LegKind::Airway),
            leg((35., 115.), (36., 140.), LegKind::Direct),
        ];
        let preferred = preferred("ZBAA", "RJTT");
        let mut actual = expected.clone();
        // The preferred route has no published SID.
        let mut published = expected.clone();
        published[0].kind = LegKind::Direct;
        // Changes wholly outside the selected portions do not affect matching.
        actual[2].to = GeoPoint::new(37., 127.).unwrap().into();
        actual[3].from = actual[2].to.clone();
        assert!(route_matches_expected(&actual, &published, &preferred));
        // A mismatch in the second child must still reject the route.
        actual[5].identifier = Some("A2".into());
        assert!(!route_matches_expected(&actual, &published, &preferred));
        actual[5] = expected[5].clone();
        // Missing and reordered children cannot match.
        assert!(!route_matches_expected(
            &actual[..3],
            &published,
            &preferred
        ));
        let reordered: Vec<_> = actual[3..].iter().chain(&actual[..3]).cloned().collect();
        assert!(!route_matches_expected(&reordered, &published, &preferred));
        // Arrivals align the STAR only on the last child.
        let reverse = |legs: Vec<ResolvedLeg>| {
            legs.into_iter()
                .rev()
                .map(|leg| {
                    let mut leg = leg.into_reversed();
                    if leg.kind == LegKind::Sid {
                        leg.kind = LegKind::Star;
                    }
                    leg
                })
                .collect::<Vec<_>>()
        };
        assert!(route_matches_expected(
            &reverse(actual),
            &reverse(published),
            &self::preferred("RJTT", "ZBAA")
        ));
    }

    #[test]
    fn routes_without_a_comparable_segment_do_not_match() {
        let route = vec![leg((35., 140.), (36., 140.), LegKind::Airway)];
        let preferred = preferred("ZBAA", "RJTT");
        assert!(!route_matches_expected(&route, &route, &preferred));
        assert!(!route_matches_expected(&[], &route, &preferred));
        let mut unknown = route.clone();
        unknown[0].to = AnyFix::Unknown("UNKNOWN".into());
        assert!(!route_matches_expected(&unknown, &route, &preferred));
    }

    #[test]
    fn preferred_route_endpoints_require_fix_equality() {
        let waypoint = Waypoint {
            icao_code: "ZG".try_into().unwrap(),
            identifier: "START".try_into().unwrap(),
            latitude: 30.0.try_into().unwrap(),
            longitude: 110.0.try_into().unwrap(),
            kind: WaypointKind::Enroute,
        };
        let expected = ResolvedLeg {
            from: AnyFix::Waypoint(waypoint.clone()),
            to: AnyFix::GeoPoint(GeoPoint::new(31., 111.).unwrap()),
            identifier: Some("A1".to_owned()),
            is_unknown: false,
            kind: LegKind::Airway,
            direction_restriction: DirectionRestriction::Forward,
        };
        let mut actual = expected.clone();
        actual.from = AnyFix::Waypoint(Waypoint {
            icao_code: "RJ".try_into().unwrap(),
            ..waypoint.clone()
        });
        assert!(leg_matches(&actual, &expected));
        actual.from = AnyFix::Waypoint(Waypoint {
            latitude: 31.0.try_into().unwrap(),
            ..waypoint
        });
        assert!(!leg_matches(&actual, &expected));
        actual.from = AnyFix::GeoPoint(GeoPoint::new(30., 110.).unwrap());
        assert!(!leg_matches(&actual, &expected));
    }
}
