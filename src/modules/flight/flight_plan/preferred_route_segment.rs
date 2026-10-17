//! Select the original route legs subject to preferred-route matching.

use crate::modules::navdata::models::{Fix, LegKind, MainlandFirBoundaries, ResolvedLeg};
use crate::modules::navdata::service::{NavdataResult, NavdataService};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SegmentError {
    #[error("route leg {index} has an endpoint without coordinates")]
    MissingCoordinates { index: usize },
}

/// Flight-specific selector using FIR boundaries from the active ARINC navdata.
pub struct PreferredRouteSegmentService {
    boundaries: Option<MainlandFirBoundaries>,
}

impl PreferredRouteSegmentService {
    /// Query the nine mainland FIRs for international flights. Domestic flights
    /// need no geographic lookup and work even with a regional navdata database.
    pub async fn new(
        navdata: &NavdataService,
        departure: &str,
        arrival: &str,
    ) -> NavdataResult<Self> {
        let boundaries = if is_mainland_airport(departure) && is_mainland_airport(arrival) {
            None
        } else {
            Some(navdata.mainland_fir_boundaries().await?)
        };
        Ok(Self { boundaries })
    }

    /// Return each contiguous matching portion as a borrowed subslice of `route`.
    /// Segments remain in route order; the route legs themselves are not copied.
    ///
    /// Leading SID and trailing STAR legs are always removed. For a flight with
    /// two mainland ICAO airports, the entire remaining route is returned.
    /// Otherwise, only legs with both endpoints inside/on the mainland FIR union
    /// are selected. A crossing leg with an outside endpoint is omitted: no new
    /// intersection fixes are synthesized. This also supports international
    /// arrivals and overflights, independently of navdata ICAO region codes.
    ///
    /// No eligible legs produces an empty vector. Leaving and re-entering the
    /// mainland FIR union produces separate child segments, excluding the foreign
    /// legs between them. Missing coordinates are explicit errors; error indices
    /// refer to the original `route`.
    pub fn select<'a>(
        &self,
        route: &'a [ResolvedLeg],
    ) -> Result<Vec<&'a [ResolvedLeg]>, SegmentError> {
        let start = route
            .iter()
            .take_while(|leg| leg.kind == LegKind::Sid)
            .count();
        let end = route.len()
            - route[start..]
                .iter()
                .rev()
                .take_while(|leg| leg.kind == LegKind::Star)
                .count();
        let enroute = &route[start..end];
        let Some(boundaries) = &self.boundaries else {
            return Ok(if enroute.is_empty() {
                vec![]
            } else {
                vec![enroute]
            });
        };

        let mut segments = Vec::new();
        let mut segment_start = None;
        for (offset, leg) in enroute.iter().enumerate() {
            let from = leg
                .from
                .position()
                .ok_or(SegmentError::MissingCoordinates {
                    index: start + offset,
                })?;
            let to = leg.to.position().ok_or(SegmentError::MissingCoordinates {
                index: start + offset,
            })?;
            if boundaries.contains(from) && boundaries.contains(to) {
                segment_start.get_or_insert(offset);
            } else if let Some(first) = segment_start.take() {
                segments.push(&enroute[first..offset]);
            }
        }
        if let Some(first) = segment_start {
            segments.push(&enroute[first..]);
        }
        Ok(segments)
    }
}

fn is_mainland_airport(identifier: &str) -> bool {
    let code = identifier.to_ascii_uppercase();
    code.len() == 4
        && code.bytes().all(|byte| byte.is_ascii_alphabetic())
        && code.starts_with('Z')
        && !code.starts_with("ZK")
        && !code.starts_with("ZM")
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn select<'a>(
        route: &'a [ResolvedLeg],
        departure: &str,
        arrival: &str,
    ) -> Result<Vec<&'a [ResolvedLeg]>, SegmentError> {
        fixture(departure, arrival).select(route)
    }

    pub(crate) fn fixture(departure: &str, arrival: &str) -> PreferredRouteSegmentService {
        PreferredRouteSegmentService {
            boundaries: if is_mainland_airport(departure) && is_mainland_airport(arrival) {
                None
            } else {
                Some(MainlandFirBoundaries::fixture())
            },
        }
    }
    use crate::modules::navdata::models::{AnyFix, DirectionRestriction, GeoPoint};

    pub(super) fn leg(from: (f64, f64), to: (f64, f64), kind: LegKind) -> ResolvedLeg {
        ResolvedLeg {
            from: GeoPoint::new(from.0, from.1).unwrap().into(),
            to: GeoPoint::new(to.0, to.1).unwrap().into(),
            identifier: Some("A1".to_owned()),
            is_unknown: false,
            kind,
            direction_restriction: DirectionRestriction::None,
        }
    }

    #[test]
    fn domestic_strips_only_terminal_procedures_and_borrows_original() {
        // Domestic selection is independent of coordinates, even outside China.
        let route = vec![
            leg((35., 140.), (36., 140.), LegKind::Sid),
            leg((36., 140.), (37., 140.), LegKind::Sid),
            leg((37., 140.), (38., 140.), LegKind::Airway),
            leg((38., 140.), (39., 140.), LegKind::Direct),
            leg((39., 140.), (40., 140.), LegKind::Star),
        ];
        let selected = select(&route, "zbaa", "zspd").unwrap();
        assert!(std::ptr::eq(selected[0], &route[2..4]));
        let selected = select(&route[2..4], "ZBAA", "ZSPD").unwrap();
        assert!(std::ptr::eq(selected[0], &route[2..4]));
    }

    #[test]
    fn empty_and_procedure_only_routes_are_empty() {
        for route in [
            vec![],
            vec![leg((30., 110.), (31., 111.), LegKind::Sid)],
            vec![leg((30., 110.), (31., 111.), LegKind::Star)],
            vec![
                leg((30., 110.), (31., 111.), LegKind::Sid),
                leg((31., 111.), (32., 112.), LegKind::Star),
            ],
        ] {
            for arrival in ["ZSPD", "RJTT"] {
                assert!(select(&route, "ZBAA", arrival).unwrap().is_empty());
            }
        }
    }

    #[test]
    fn international_departure_arrival_and_overflight_use_coordinates() {
        let route = vec![
            leg((35., 140.), (31., 111.), LegKind::Direct),
            leg((31., 111.), (32., 112.), LegKind::Airway),
            leg((32., 112.), (33., 113.), LegKind::Airway),
            leg((33., 113.), (37., 127.), LegKind::Direct),
            leg((37., 127.), (35., 140.), LegKind::Airway),
        ];
        for (departure, arrival) in [("RJTT", "RKSI"), ("ZBAA", "RJTT"), ("RJTT", "ZBAA")] {
            let selected = select(&route, departure, arrival).unwrap();
            assert!(std::ptr::eq(selected[0], &route[1..3]));
        }
        let reversed: Vec<_> = route
            .iter()
            .cloned()
            .rev()
            .map(ResolvedLeg::into_reversed)
            .collect();
        assert!(std::ptr::eq(
            select(&reversed, "RKSI", "RJTT").unwrap()[0],
            &reversed[2..4],
        ));
    }

    #[test]
    fn international_procedures_are_removed_before_geography() {
        let route = vec![
            leg((30., 110.), (31., 111.), LegKind::Sid),
            leg((31., 111.), (32., 112.), LegKind::Airway),
            leg((32., 112.), (35., 140.), LegKind::Direct),
            leg((35., 140.), (36., 140.), LegKind::Star),
        ];
        assert!(std::ptr::eq(
            select(&route, "ZBAA", "RJTT").unwrap()[0],
            &route[1..2],
        ));
    }

    #[test]
    fn foreign_routes_are_empty_including_mongolia_and_north_korea() {
        let route = vec![leg((35., 140.), (36., 140.), LegKind::Airway)];
        for (departure, arrival) in [
            ("RJTT", "RKSI"),
            ("ZBAA", "ZMCK"),
            ("ZKPY", "ZBAA"),
            ("VHHH", "ZBAA"),
            ("RCTP", "ZBAA"),
        ] {
            assert!(select(&route, departure, arrival).unwrap().is_empty());
        }
    }

    #[test]
    fn unknown_coordinates_preserve_original_index_in_error() {
        let mut unknown = leg((31., 111.), (32., 112.), LegKind::Airway);
        unknown.to = AnyFix::Unknown("UNKNOWN".to_owned());
        let route = vec![leg((30., 110.), (31., 111.), LegKind::Sid), unknown];
        assert_eq!(
            select(&route, "ZBAA", "RJTT"),
            Err(SegmentError::MissingCoordinates { index: 1 })
        );
        assert_eq!(select(&route, "ZBAA", "ZSPD").unwrap().len(), 1);
    }

    #[test]
    fn reentry_returns_separate_borrowed_subslices_in_route_order() {
        let route = vec![
            leg((30., 110.), (31., 111.), LegKind::Airway),
            leg((31., 111.), (35., 140.), LegKind::Direct),
            leg((35., 140.), (32., 112.), LegKind::Direct),
            leg((32., 112.), (33., 113.), LegKind::Airway),
            leg((33., 113.), (34., 114.), LegKind::Airway),
            leg((34., 114.), (35., 140.), LegKind::Direct),
        ];
        let selected = select(&route, "ZBAA", "RJTT").unwrap();
        assert_eq!(selected.len(), 2);
        assert!(std::ptr::eq(selected[0], &route[0..1]));
        assert!(std::ptr::eq(selected[1], &route[3..5]));
        let reversed: Vec<_> = route
            .iter()
            .cloned()
            .rev()
            .map(ResolvedLeg::into_reversed)
            .collect();
        let selected = select(&reversed, "RJTT", "ZBAA").unwrap();
        assert_eq!(selected.len(), 2);
        assert!(std::ptr::eq(selected[0], &reversed[1..3]));
        assert!(std::ptr::eq(selected[1], &reversed[5..6]));
        // Domestic flights retain the whole enroute span, including excursions.
        let selected = select(&route, "ZBAA", "ZSPD").unwrap();
        assert_eq!(selected.len(), 1);
        assert!(std::ptr::eq(selected[0], route.as_slice()));
    }
}
