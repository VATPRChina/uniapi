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
    assert_eq!(segments[0].route.end_index(), 3);
    assert_eq!(segments[1].kind, SegmentKind::Mainland);
    assert_eq!(segments[1].route.start_index, 3);
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
        fix(112.),
        fix(125.),
        fix(130.),
        fix(115.),
        fix(117.),
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
            (SegmentKind::International, 0, 4),
            (SegmentKind::Mainland, 4, 5),
            (SegmentKind::International, 5, 8),
            (SegmentKind::Mainland, 8, 9),
            (SegmentKind::International, 9, 11),
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
        Some(&fix(112.))
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
fn unresolved_exit_chain_starts_after_last_china_fix() {
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
    assert_eq!(segments[0].route.end_index(), 1);
    assert_eq!(segments[1].route.start_index, 1);
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

#[test]
fn route_dto_preserves_global_leg_indices_and_terminal_kinds() {
    use crate::modules::flight::dto::FlightRouteDto;
    let b = boundary();
    let mut legs = route(&[
        airport("ZSPD", 110.),
        fix(115.),
        fix(130.),
        airport("RJAA", 140.),
    ]);
    legs[0].kind = LegKind::Sid;
    legs[1].identifier = None;
    legs[1].kind = LegKind::Direct;
    legs[2].kind = LegKind::Star;
    let segments = RouteSegmenter::new(&b).segment(&legs);
    let dto = serde_json::to_value(FlightRouteDto::from_segments(&segments)).unwrap();
    assert_eq!(dto["segments"][0]["index"], 0);
    assert_eq!(dto["segments"][1]["index"], 1);
    assert_eq!(dto["segments"][0]["initial_fix"], "ZSPD");
    assert_eq!(dto["segments"][0]["last_fix"], "30.000000,115.000000");
    assert_eq!(
        dto["segments"][0]["enroute_initial_fix"],
        "30.000000,115.000000"
    );
    assert_eq!(dto["segments"][0]["legs"][0]["leg_kind"], "sid");
    assert_eq!(dto["segments"][1]["legs"][0]["leg_kind"], "enroute");
    assert!(dto["segments"][1]["legs"][0]["route_ident"].is_null());
    assert_eq!(dto["segments"][1]["legs"][1]["leg_kind"], "star");
    for segment in &segments {
        for (_, variant) in segment.endpoint_variants() {
            assert!(variant.end_index() <= segment.route.end_index());
        }
    }
    let indices: Vec<_> = dto["segments"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|segment| segment["legs"].as_array().unwrap().iter())
        .map(|leg| leg["index"].as_u64().unwrap())
        .collect();
    assert_eq!(indices, vec![0, 1, 2]);
}

#[test]
fn lone_china_departure_does_not_produce_an_empty_segment() {
    let b = boundary();
    let legs = route(&[fix(110.), fix(130.), fix(140.)]);
    let segments = RouteSegmenter::new(&b).segment(&legs);
    assert_partition(&legs, &segments);
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].kind, SegmentKind::International);
}

#[test]
fn unresolved_entry_chain_stays_international_until_first_china_fix() {
    let b = boundary();
    let legs = route(&[
        fix(130.),
        AnyFix::Unknown("U1".into()),
        AnyFix::Unknown("U2".into()),
        fix(110.),
        fix(115.),
    ]);
    let segments = RouteSegmenter::new(&b).segment(&legs);
    assert_partition(&legs, &segments);
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].kind, SegmentKind::International);
    assert_eq!(segments[0].route.end_index(), 3);
    let mainland = &segments[1];
    assert_eq!(mainland.kind, SegmentKind::Mainland);
    assert_eq!(mainland.route.start_index, 3);
    assert_eq!(mainland.route.legs[0].from, fix(110.));
    assert_eq!(
        mainland.get_cross_border_enter_start_fix(),
        Some(&fix(130.))
    );
    assert_eq!(mainland.get_cross_border_enter_end_fix(), Some(&fix(110.)));
    for (_, variant) in mainland.endpoint_variants() {
        assert_eq!(variant.start_index, mainland.route.start_index);
        assert!(variant.end_index() <= mainland.route.end_index());
    }
}

#[test]
fn isolated_china_fixes_do_not_split_international_legs() {
    let b = boundary();
    for fixes in [
        vec![fix(130.), fix(110.)],
        vec![fix(130.), fix(110.), fix(135.), fix(115.), fix(140.)],
    ] {
        let legs = route(&fixes);
        let segments = RouteSegmenter::new(&b).segment(&legs);
        assert_partition(&legs, &segments);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].kind, SegmentKind::International);
    }
}
