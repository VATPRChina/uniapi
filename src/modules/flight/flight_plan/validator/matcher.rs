use itertools::Itertools;
use tracing::info;

use crate::modules::flight::flight_plan::parse_route;
use crate::modules::flight::flight_plan::validator::ValidatorError;
use crate::modules::navdata::models::{AnyFix, Fix, LegKind, PreferredRoute, ResolvedLeg};
use crate::modules::navdata::service::NavdataService;

async fn find_matching_route<'a>(
    navdata: &NavdataService,
    legs: &[ResolvedLeg],
    preferred_routes: &[&'a PreferredRoute],
) -> Result<Option<&'a PreferredRoute>, ValidatorError> {
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
        if route_matches_expected(legs, &parsed.legs, preferred_route) {
            return Ok(Some(preferred_route));
        }
    }
    Ok(None)
}

/// Match the actual enroute span between its leading SID and trailing STAR.
/// Procedure boundaries must occur in expected; absent procedures anchor that
/// end of the comparison to the corresponding route edge.
fn route_matches_expected(
    actual: &[ResolvedLeg],
    expected: &[ResolvedLeg],
    preferred_route: &PreferredRoute,
) -> bool {
    if actual.is_empty() || expected.is_empty() {
        return false;
    }

    // incomplete expected route for international routes
    let (expected, actual) = if (!preferred_route.arrival.starts_with('Z')
        || preferred_route.arrival.starts_with("ZM")
        || preferred_route.arrival.starts_with("ZK"))
        && let Some(expected_final_fix) = expected.last().and_then(|last| last.from.identifier())
    {
        let actual_pos = actual
            .iter()
            .take_while(|leg| leg.from.identifier() != Some(expected_final_fix))
            .count();
        info!(
            "incomplete route end at {}, truncate actual to [0..{}/{}]",
            expected_final_fix,
            actual_pos,
            actual.len()
        );
        (&expected[0..(expected.len() - 1)], &actual[0..actual_pos])
    } else {
        (expected, actual)
    };

    let enroute_start = actual
        .iter()
        .take_while(|leg| leg.kind == LegKind::Sid)
        .count();
    let enroute_end = actual.len()
        - actual
            .iter()
            .rev()
            .take_while(|leg| leg.kind == LegKind::Star)
            .count();
    let enroute = &actual[enroute_start..enroute_end];
    let sid_exit = enroute_start.checked_sub(1).map(|index| &actual[index].to);
    let star_enter = actual.get(enroute_end).map(|leg| &leg.from);
    info!(
        "actual enroute={:?}[{}:{}], sid_exit={:?}, star_enter={:?}",
        enroute, enroute_start, enroute_end, sid_exit, star_enter
    );

    let expected_enroute_start = sid_exit
        .map(|sid_exit| {
            expected
                .iter()
                .take_while(|leg| !fix_matches(&leg.from, sid_exit))
                .count()
        })
        .unwrap_or_default();
    let expected_enroute_end = expected.len()
        - star_enter
            .map(|star_enter| {
                expected
                    .iter()
                    .rev()
                    .take_while(|leg| !fix_matches(&leg.to, star_enter))
                    .count()
            })
            .unwrap_or_default();
    if expected_enroute_end <= expected_enroute_start {
        info!(
            "unable to find start and end on preferred route: {}-{}",
            expected_enroute_start, expected_enroute_end
        );
        return false;
    }
    let expected_enroute = &expected[expected_enroute_start..expected_enroute_end];
    info!(
        "expected enroute={:?}[{}:{}]",
        expected_enroute, expected_enroute_start, expected_enroute_end
    );

    if enroute.len() != expected_enroute.len() {
        return false;
    }

    enroute
        .iter()
        .zip(expected_enroute.iter())
        .all(|(expected, actual)| leg_matches(actual, expected))
}

fn leg_matches(actual: &ResolvedLeg, expected: &ResolvedLeg) -> bool {
    actual.identifier == expected.identifier
        && fix_matches(&actual.from, &expected.from)
        && fix_matches(&actual.to, &expected.to)
}

fn fix_matches(actual: &AnyFix, expected: &AnyFix) -> bool {
    match (actual.identifier(), expected.identifier()) {
        (Some(actual), Some(expected)) => actual.eq_ignore_ascii_case(expected),
        (None, None) => {
            approx::relative_eq!(actual.latitude(), expected.latitude(), max_relative = 1e-6)
                && approx::relative_eq!(
                    actual.longitude(),
                    expected.longitude(),
                    max_relative = 1e-6
                )
        }
        _ => false,
    }
}
