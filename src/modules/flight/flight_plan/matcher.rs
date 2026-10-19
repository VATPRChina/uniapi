use super::segmenter::enroute;
use super::{ParseRouteError, RouteSegment, Segment, parse_route};
use crate::modules::flight::flight_plan::validator::ValidatorError;
use crate::modules::navdata::models::{AnyFix, PreferredRoute, ResolvedLeg};
use crate::modules::navdata::service::NavdataService;

pub struct PreferredRouteMatcher<'s> {
    navdata: &'s NavdataService,
}

impl<'s> PreferredRouteMatcher<'s> {
    pub fn new(navdata: &'s NavdataService) -> Self {
        Self { navdata }
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
            if self.matches(variant, preferred).await? {
                return Ok(Some((variant, preferred.clone())));
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
    ) -> Result<Vec<&'s PreferredRoute>, ValidatorError> {
        let mut routes: Vec<&'s PreferredRoute> = Vec::new();
        for (_, preferred) in self.candidates(*segment).await? {
            if !routes.iter().any(|route| std::ptr::eq(*route, preferred)) {
                routes.push(preferred);
            }
        }
        Ok(routes)
    }

    async fn candidates<'l>(
        &self,
        segment: Segment<'l>,
    ) -> Result<Vec<(RouteSegment<'l>, &'s PreferredRoute)>, ValidatorError> {
        let mut candidates = Vec::new();
        for (airport_pair, variant) in segment.endpoint_variants() {
            let Some(from) = variant.legs.first().and_then(|leg| leg.from.identifier()) else {
                continue;
            };
            let Some(to) = variant.legs.last().and_then(|leg| leg.to.identifier()) else {
                continue;
            };
            let navdata: &'s NavdataService = self.navdata;
            candidates.extend(
                navdata
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
        // CSV routes omit departure/arrival airports. Add only those anchors so
        // the parser can also represent designated routes containing one fix.
        let mut route = preferred.raw_route.trim().to_owned();
        if route.is_empty() {
            return Ok(false);
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
                return Ok(false);
            }
            Err(error) => return Err(error.into()),
        };
        Ok(routes_match(segment.legs, &expected.legs))
    }
}

fn routes_match(actual: &[ResolvedLeg], expected: &[ResolvedLeg]) -> bool {
    let (Some((from, actual, to)), Some((expected_from, expected, expected_to))) =
        (enroute(actual), enroute(expected))
    else {
        return false;
    };
    !from.is_unknown()
        && !to.is_unknown()
        && from == expected_from
        && to == expected_to
        && actual.len() == expected.len()
        && actual.iter().zip(expected).all(|(actual, expected)| {
            !actual.is_unknown
                && !expected.is_unknown
                && !actual.from.is_unknown()
                && !actual.to.is_unknown()
                && actual.kind == expected.kind
                && actual.identifier == expected.identifier
                && actual.from == expected.from
                && actual.to == expected.to
        })
}
