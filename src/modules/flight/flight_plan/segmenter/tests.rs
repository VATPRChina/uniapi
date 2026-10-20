use super::*;
use crate::modules::navdata::models::{Airport, DirectionRestriction, GeoPoint};

fn boundary() -> ChinaMainlandBoundary {
    ChinaMainlandBoundary::from_rings(vec![vec![
        (100., 20.),
        (120., 20.),
        (120., 40.),
        (100., 40.),
    ]])
    .unwrap()
}
fn fix(lon: f64) -> AnyFix {
    GeoPoint::new(30., lon).unwrap().into()
}
fn airport(ident: &str, lon: f64) -> AnyFix {
    AnyFix::Airport(Airport {
        identifier: ident.try_into().unwrap(),
        latitude: 30.0.try_into().unwrap(),
        longitude: lon.try_into().unwrap(),
    })
}
fn route(fixes: &[AnyFix]) -> Vec<ResolvedLeg> {
    fixes
        .windows(2)
        .map(|pair| ResolvedLeg {
            from: pair[0].clone(),
            to: pair[1].clone(),
            identifier: Some("A1".into()),
            kind: LegKind::Airway,
            is_unknown: pair.iter().any(AnyFix::is_unknown),
            direction_restriction: DirectionRestriction::None,
        })
        .collect()
}
fn assert_partition(legs: &[ResolvedLeg], segments: &[Segment<'_>]) {
    let mut cursor = 0;
    for segment in segments {
        assert_eq!(segment.route.start_index, cursor);
        assert!(!segment.route.legs.is_empty());
        assert!(std::ptr::eq(
            segment.route.legs.as_ptr(),
            legs[cursor..].as_ptr()
        ));
        cursor = segment.route.end_index();
    }
    assert_eq!(cursor, legs.len());
}

#[test]
fn international_departure_is_not_attached_to_mainland_visit() {
    let b = boundary();
    let legs = route(&[
        airport("EDDK", 7.),
        fix(90.),
        fix(95.),
        fix(105.),
        airport("ZSPD", 115.),
    ]);
    let segments = RouteSegmenter::new(&b).segment(&legs);
    assert_partition(&legs, &segments);
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].kind, SegmentKind::International);
    assert_eq!(segments[0].route.end_index(), 2);
    assert_eq!(segments[1].kind, SegmentKind::Mainland);
    assert_eq!(segments[1].route.start_index, 2);
    assert!(segments[1].get_route_start_airport().is_none());
    assert_eq!(
        segments[1].get_cross_border_enter_start_fix(),
        Some(&fix(95.))
    );
    assert_eq!(
        segments[1].get_cross_border_enter_end_fix(),
        Some(&fix(105.))
    );
    assert_eq!(
        segments[1].get_route_end_airport().unwrap().identifier(),
        Some("ZSPD")
    );
}

#[test]
fn visits_crossings_unknown_chains_and_foreign_gaps_partition_once() {
    let b = boundary();
    let legs = route(&[
        airport("RJAA", 140.),
        fix(130.),
        AnyFix::Unknown("U1".into()),
        AnyFix::Unknown("U2".into()),
        fix(110.),
        fix(125.),
        fix(130.),
        fix(115.),
        fix(135.),
        airport("RKAA", 140.),
    ]);
    let segments = RouteSegmenter::new(&b).segment(&legs);
    assert_partition(&legs, &segments);
    assert_eq!(
        segments
            .iter()
            .map(|s| (s.kind, s.route.start_index, s.route.end_index()))
            .collect::<Vec<_>>(),
        vec![
            (SegmentKind::International, 0, 1),
            (SegmentKind::Mainland, 1, 5),
            (SegmentKind::International, 5, 6),
            (SegmentKind::Mainland, 6, 8),
            (SegmentKind::International, 8, 9),
        ]
    );
    assert_eq!(
        segments[1].get_cross_border_enter_start_fix(),
        Some(&fix(130.))
    );
    assert_eq!(
        segments[1].get_cross_border_enter_end_fix(),
        Some(&fix(110.))
    );
    assert_eq!(
        segments[1].get_cross_border_exit_start_fix(),
        Some(&fix(110.))
    );
    assert_eq!(
        segments[1].get_cross_border_exit_end_fix(),
        Some(&fix(125.))
    );
}

#[test]
fn single_region_empty_unresolved_and_disconnected_routes() {
    let b = boundary();
    let segmenter = RouteSegmenter::new(&b);
    assert!(segmenter.segment(&[]).is_empty());
    for (fixes, kind) in [
        (vec![fix(105.), fix(110.), fix(115.)], SegmentKind::Mainland),
        (
            vec![fix(80.), AnyFix::Unknown("U".into()), fix(90.)],
            SegmentKind::International,
        ),
        (
            vec![AnyFix::Unknown("U1".into()), AnyFix::Unknown("U2".into())],
            SegmentKind::International,
        ),
        (
            vec![
                AnyFix::Unknown("U1".into()),
                fix(105.),
                AnyFix::Unknown("U2".into()),
            ],
            SegmentKind::Mainland,
        ),
    ] {
        let legs = route(&fixes);
        let segments = segmenter.segment(&legs);
        assert_partition(&legs, &segments);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].kind, kind);
    }
    let mut disconnected = route(&[fix(105.), fix(110.)]);
    disconnected.extend(route(&[fix(115.), fix(116.)]));
    let segments = segmenter.segment(&disconnected);
    assert_partition(&disconnected, &segments);
    assert_eq!(segments.len(), 2);
}

#[test]
fn unresolved_exit_chain_stays_with_mainland_and_foreign_arrival_is_separate() {
    let b = boundary();
    let legs = route(&[
        airport("ZSPD", 110.),
        fix(115.),
        AnyFix::Unknown("U1".into()),
        AnyFix::Unknown("U2".into()),
        fix(130.),
        airport("RJAA", 140.),
    ]);
    let segments = RouteSegmenter::new(&b).segment(&legs);
    assert_partition(&legs, &segments);
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].kind, SegmentKind::Mainland);
    assert_eq!(segments[0].route.end_index(), 4);
    assert_eq!(
        segments[0].get_route_start_airport().unwrap().identifier(),
        Some("ZSPD")
    );
    assert!(segments[0].get_route_end_airport().is_none());
    assert_eq!(
        segments[0].get_cross_border_exit_start_fix(),
        Some(&fix(115.))
    );
    assert_eq!(
        segments[0].get_cross_border_exit_end_fix(),
        Some(&fix(130.))
    );
    assert_eq!(segments[1].kind, SegmentKind::International);
    assert_eq!(
        segments[1].get_route_end_airport().unwrap().identifier(),
        Some("RJAA")
    );
}
