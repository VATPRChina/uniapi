use crate::modules::navdata::models::{AnyFix, ChinaMainlandBoundary, LegKind, ResolvedLeg};

/// A connected borrowed portion of the original expanded route.
#[derive(Debug, Clone, Copy)]
pub struct RouteSegment<'l> {
    pub legs: &'l [ResolvedLeg],
    pub start_index: usize,
}

impl RouteSegment<'_> {
    pub fn end_index(&self) -> usize {
        self.start_index + self.legs.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SegmentKind {
    Mainland,
    International,
}

/// A mainland visit from its first to last China fix, or an international portion.
/// Fix offsets refer to vertices in `route`: vertex n is the end of leg n - 1.
#[derive(Debug, Clone, Copy)]
pub struct Segment<'l> {
    pub kind: SegmentKind,
    pub route: RouteSegment<'l>,
    enter: Option<(&'l AnyFix, &'l AnyFix)>,
    exit: Option<(&'l AnyFix, &'l AnyFix)>,
}

impl<'l> Segment<'l> {
    fn fix(&self, index: usize) -> &'l AnyFix {
        if index == 0 {
            &self.route.legs[0].from
        } else {
            &self.route.legs[index - 1].to
        }
    }

    pub fn get_route_start_airport(&self) -> Option<&'l AnyFix> {
        let fix = self.fix(0);
        matches!(fix, AnyFix::Airport(_)).then_some(fix)
    }

    pub fn get_route_end_airport(&self) -> Option<&'l AnyFix> {
        let fix = self.fix(self.route.legs.len());
        matches!(fix, AnyFix::Airport(_)).then_some(fix)
    }

    pub fn get_enroute_start_fix(&self) -> &'l AnyFix {
        enroute(self.route.legs).unwrap().0
    }
    pub fn get_enroute_end_fix(&self) -> &'l AnyFix {
        enroute(self.route.legs).unwrap().2
    }
    pub fn get_cross_border_enter_start_fix(&self) -> Option<&'l AnyFix> {
        self.enter.map(|(start, _)| start)
    }
    pub fn get_cross_border_enter_end_fix(&self) -> Option<&'l AnyFix> {
        self.enter.map(|(_, end)| end)
    }
    pub fn get_cross_border_exit_start_fix(&self) -> Option<&'l AnyFix> {
        self.exit.map(|(start, _)| start)
    }
    pub fn get_cross_border_exit_end_fix(&self) -> Option<&'l AnyFix> {
        self.exit.map(|(_, end)| end)
    }

    pub(crate) fn endpoint_variants(&self) -> Vec<(bool, RouteSegment<'l>)> {
        let len = self.route.legs.len();
        let airport_pair =
            self.get_route_start_airport().is_some() && self.get_route_end_airport().is_some();
        let mut variants = Vec::new();
        if airport_pair {
            variants.push((true, self.route));
        }
        let mut starts = Vec::new();
        let mut ends = Vec::new();
        if self.get_route_start_airport().is_some() {
            starts.push(0);
        }
        if self.enter.is_some() {
            starts.push(0);
        }
        if starts.is_empty() {
            starts.push(0);
        }
        if self.get_route_end_airport().is_some() {
            ends.push(len);
        }
        if self.exit.is_some() {
            ends.push(len);
        }
        if ends.is_empty() {
            ends.push(len);
        }
        starts.sort_unstable();
        starts.dedup();
        ends.sort_unstable();
        ends.dedup();
        for start in starts {
            for &end in &ends {
                if start < end && !(airport_pair && start == 0 && end == len) {
                    variants.push((
                        false,
                        RouteSegment {
                            legs: &self.route.legs[start..end],
                            start_index: self.route.start_index + start,
                        },
                    ));
                }
            }
        }
        variants
    }
}

pub struct RouteSegmenter<'b> {
    boundary: &'b ChinaMainlandBoundary,
}

impl<'b> RouteSegmenter<'b> {
    pub fn new(boundary: &'b ChinaMainlandBoundary) -> Self {
        Self { boundary }
    }

    pub fn segment<'l>(&self, legs: &'l [ResolvedLeg]) -> Vec<Segment<'l>> {
        let mut segments = Vec::new();
        let mut start = 0;
        for end in 1..=legs.len() {
            if end == legs.len() || legs[end - 1].to != legs[end].from {
                self.connected(&legs[start..end], start, &mut segments);
                start = end;
            }
        }
        segments
    }

    fn connected<'l>(
        &self,
        legs: &'l [ResolvedLeg],
        offset: usize,
        segments: &mut Vec<Segment<'l>>,
    ) {
        // Unknown fixes are skipped only for geographical classification. They
        // stay in the borrowed path and still participate in exact matching.
        let fixes = std::iter::once(&legs[0].from).chain(legs.iter().map(|leg| &leg.to));
        let known: Vec<_> = fixes
            .enumerate()
            .filter(|(_, fix)| !fix.is_unknown())
            .map(|(index, fix)| (index, fix.is_china_or_unknown(self.boundary)))
            .collect();
        let mut cursor = 0;
        let mut covered = 0;
        while cursor < known.len() {
            if !known[cursor].1 {
                cursor += 1;
                continue;
            }
            let first = cursor;
            while cursor + 1 < known.len() && known[cursor + 1].1 {
                cursor += 1;
            }
            let last = cursor;
            let enter = (first > 0).then(|| (known[first - 1].0, known[first].0));
            let exit = (last + 1 < known.len()).then(|| (known[last].0, known[last + 1].0));
            // Crossings stay outside the mainland slice. Keep their endpoints
            // for diagnostics even though they may lie outside the slice.
            let start = enter.map_or(0, |(_, end)| end);
            let end = exit.map_or(legs.len(), |(start, _)| start);
            // A lone China vertex has no mainland leg to validate. Leave it
            // in the surrounding international portion without splitting it.
            if start >= end {
                cursor += 1;
                continue;
            }
            let fix = |index: usize| {
                if index == 0 {
                    &legs[0].from
                } else {
                    &legs[index - 1].to
                }
            };
            if covered < start {
                segments.push(Segment {
                    kind: SegmentKind::International,
                    route: RouteSegment {
                        legs: &legs[covered..start],
                        start_index: offset + covered,
                    },
                    enter: None,
                    exit: None,
                });
            }
            if start < end {
                segments.push(Segment {
                    kind: SegmentKind::Mainland,
                    route: RouteSegment {
                        legs: &legs[start..end],
                        start_index: offset + start,
                    },
                    enter: enter.map(|(a, b)| (fix(a), fix(b))),
                    exit: exit.map(|(a, b)| (fix(a), fix(b))),
                });
            }
            covered = end;
            cursor += 1;
        }
        if covered < legs.len() {
            segments.push(Segment {
                kind: SegmentKind::International,
                route: RouteSegment {
                    legs: &legs[covered..],
                    start_index: offset + covered,
                },
                enter: None,
                exit: None,
            });
        }
    }
}

/// Exclude terminal procedures and airport connections, while retaining the
/// resulting fix when the entire enroute path consists of a single waypoint.
pub(super) fn enroute(legs: &[ResolvedLeg]) -> Option<(&AnyFix, &[ResolvedLeg], &AnyFix)> {
    let mut from = &legs.first()?.from;
    let mut to = &legs.last()?.to;
    let (mut start, mut end) = (0, legs.len());
    while start < end
        && (legs[start].kind == LegKind::Sid || matches!(legs[start].from, AnyFix::Airport(_)))
    {
        from = &legs[start].to;
        start += 1;
    }
    while start < end
        && (legs[end - 1].kind == LegKind::Star || matches!(legs[end - 1].to, AnyFix::Airport(_)))
    {
        to = &legs[end - 1].from;
        end -= 1;
    }
    Some((from, &legs[start..end], to))
}

#[cfg(test)]
mod tests;
