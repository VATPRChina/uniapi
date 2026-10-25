use std::collections::HashSet;

use super::segmenter::enroute;
use super::{ParseRouteError, RouteSegment, Segment, SegmentKind, parse_route};
use crate::modules::flight::flight_plan::validator::ValidatorError;
use crate::modules::navdata::models::{AnyFix, PreferredRoute, ResolvedLeg};
use crate::modules::navdata::service::NavdataService;

pub struct PreferredRouteMatcher<'s> {
    navdata: &'s NavdataService,
}

/// Candidates are in the same priority order used to select a preferred route.
pub(crate) struct SegmentMatchReport<'l> {
    pub candidates: Vec<RouteMatchAttempt<'l>>,
    pub selected_candidate_index: Option<usize>,
    pub preferred_routes: Vec<PreferredRoute>,
}

pub(crate) struct RouteMatchAttempt<'l> {
    pub coverage: RouteSegment<'l>,
    pub preferred: PreferredRoute,
    pub matched: bool,
    #[allow(unused)]
    pub error: Option<String>,
}

impl<'l> SegmentMatchReport<'l> {
    pub fn selected(&self) -> Option<&RouteMatchAttempt<'l>> {
        self.selected_candidate_index
            .map(|index| &self.candidates[index])
    }
}

impl<'s> PreferredRouteMatcher<'s> {
    pub fn new(navdata: &'s NavdataService) -> Self {
        Self { navdata }
    }

    pub(crate) async fn inspect_segment<'l>(
        &self,
        segment: &Segment<'l>,
    ) -> Result<SegmentMatchReport<'l>, ValidatorError> {
        let preferred_routes = self.preferred_routes(segment).await?;
        let mut candidates = Vec::new();
        for (coverage, preferred) in self.candidates(*segment).await? {
            let (matched, error) = self.match_outcome(coverage, &preferred).await?;
            candidates.push(RouteMatchAttempt {
                coverage,
                preferred,
                matched,
                error,
            });
        }
        let selected_candidate_index = candidates.iter().position(|candidate| candidate.matched);
        Ok(SegmentMatchReport {
            candidates,
            selected_candidate_index,
            preferred_routes,
        })
    }

    /// Match one visit, trying airport pairs before all border alternatives.
    pub async fn match_segment(
        &self,
        segment: &Segment<'_>,
    ) -> Result<Option<PreferredRoute>, ValidatorError> {
        Ok(self
            .match_with_coverage(*segment)
            .await?
            .map(|(_, route)| route))
    }

    async fn match_with_coverage<'l>(
        &self,
        segment: Segment<'l>,
    ) -> Result<Option<(RouteSegment<'l>, PreferredRoute)>, ValidatorError> {
        for (variant, preferred) in self.candidates(segment).await? {
            if self.matches(variant, &preferred).await? {
                return Ok(Some((variant, preferred)));
            }
        }
        Ok(None)
    }

    pub async fn find_matching_route<'l>(
        &self,
        segments: impl IntoIterator<Item = Segment<'l>>,
    ) -> Result<Vec<(RouteSegment<'l>, Option<PreferredRoute>)>, ValidatorError> {
        let mut matches = Vec::new();
        for segment in segments {
            matches.push(match self.match_with_coverage(segment).await? {
                Some((coverage, route)) => (coverage, Some(route)),
                None => (segment.route, None),
            });
        }
        Ok(matches)
    }

    pub async fn preferred_routes(
        &self,
        segment: &Segment<'_>,
    ) -> Result<Vec<PreferredRoute>, ValidatorError> {
        if segment.kind != SegmentKind::Mainland {
            return Ok(Vec::new());
        }
        let mut seen = HashSet::new();
        let mut routes = Vec::new();
        for (airport_pair, variant) in segment.endpoint_variants() {
            let Some((from, to)) = endpoints(variant) else {
                continue;
            };
            // Query each pair once, retaining separate database rows with identical contents.
            if seen.insert((from.to_ascii_uppercase(), to.to_ascii_uppercase())) {
                routes.extend(
                    self.navdata
                        .list_preferred_routes(from, to)
                        .await
                        .map_err(ValidatorError::Navdata)?
                        .into_iter()
                        .map(|route| (airport_pair, route)),
                );
            }
        }
        routes.sort_by_key(|(airport_pair, route)| (!airport_pair, !route.is_public));
        Ok(routes.into_iter().map(|(_, route)| route).collect())
    }

    async fn candidates<'l>(
        &self,
        segment: Segment<'l>,
    ) -> Result<Vec<(RouteSegment<'l>, PreferredRoute)>, ValidatorError> {
        if segment.kind != SegmentKind::Mainland {
            return Ok(Vec::new());
        }
        let mut candidates = Vec::new();
        for (airport_pair, variant) in segment.endpoint_variants() {
            let Some((from, to)) = endpoints(variant) else {
                continue;
            };
            candidates.extend(
                self.navdata
                    .list_preferred_routes(from, to)
                    .await
                    .map_err(ValidatorError::Navdata)?
                    .into_iter()
                    .map(|route| (airport_pair, variant, route)),
            );
        }
        candidates.sort_by_key(|(airport_pair, _, route)| (!airport_pair, !route.is_public));
        Ok(candidates
            .into_iter()
            .map(|(_, variant, route)| (variant, route))
            .collect())
    }

    async fn matches(
        &self,
        segment: RouteSegment<'_>,
        preferred: &PreferredRoute,
    ) -> Result<bool, ValidatorError> {
        Ok(self.match_outcome(segment, preferred).await?.0)
    }

    async fn match_outcome(
        &self,
        segment: RouteSegment<'_>,
        preferred: &PreferredRoute,
    ) -> Result<(bool, Option<String>), ValidatorError> {
        // Preferred routes omit departure/arrival airports. Add only those anchors so
        // the parser can also represent designated routes containing one fix.
        let mut route = preferred.raw_route.trim().to_owned();
        if route.is_empty() {
            return Ok((false, Some("Preferred route is empty".into())));
        }
        if let Some(AnyFix::Airport(airport)) = segment.legs.first().map(|leg| &leg.from)
            && !route
                .split_whitespace()
                .next()
                .is_some_and(|ident| ident.eq_ignore_ascii_case(&airport.identifier))
        {
            route = format!("{} {route}", airport.identifier);
        }
        if let Some(AnyFix::Airport(airport)) = segment.legs.last().map(|leg| &leg.to)
            && !route
                .split_whitespace()
                .last()
                .is_some_and(|ident| ident.eq_ignore_ascii_case(&airport.identifier))
        {
            route = format!("{route} {}", airport.identifier);
        }
        let expected = match parse_route(self.navdata, &route).await {
            Ok(route) => route,
            Err(ParseRouteError::InvalidRoute(error)) => {
                tracing::warn!(route = preferred.name, %error, "ignoring invalid preferred route");
                return Ok((false, Some(error)));
            }
            Err(error) => return Err(error.into()),
        };
        Ok((routes_match(segment.legs, &expected.legs), None))
    }
}

fn endpoints(variant: RouteSegment<'_>) -> Option<(&str, &str)> {
    Some((
        variant.legs.first()?.from.identifier()?,
        variant.legs.last()?.to.identifier()?,
    ))
}

/// Airport endpoints allow the preferred path to extend into the filed SID or
/// STAR. Compare only the window between the filed enroute endpoints; border
/// endpoints must still coincide with the preferred path's endpoints.
fn routes_match(actual: &[ResolvedLeg], expected: &[ResolvedLeg]) -> bool {
    let trim_start = actual
        .first()
        .is_some_and(|leg| matches!(leg.from, AnyFix::Airport(_)));
    let trim_end = actual
        .last()
        .is_some_and(|leg| matches!(leg.to, AnyFix::Airport(_)));
    let (Some((from, actual, to)), Some((expected_from, expected, expected_to))) =
        (enroute(actual), enroute(expected))
    else {
        return false;
    };
    if from.is_unknown() || to.is_unknown() || actual.len() > expected.len() {
        return false;
    }
    (0..=expected.len() - actual.len()).any(|start| {
        let end = start + actual.len();
        if (!trim_start && start != 0) || (!trim_end && end != expected.len()) {
            return false;
        }
        let window_from = if start == 0 {
            expected_from
        } else {
            &expected[start - 1].to
        };
        let window_to = if end == expected.len() {
            expected_to
        } else {
            &expected[end].from
        };
        from == window_from
            && to == window_to
            && actual
                .iter()
                .zip(&expected[start..end])
                .all(|(actual, expected)| {
                    !actual.is_unknown
                        && !expected.is_unknown
                        && !actual.from.is_unknown()
                        && !actual.to.is_unknown()
                        && actual.kind == expected.kind
                        && actual.identifier == expected.identifier
                        && actual.from == expected.from
                        && actual.to == expected.to
                })
    })
}
