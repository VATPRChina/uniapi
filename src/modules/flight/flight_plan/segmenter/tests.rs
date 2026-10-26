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

#[test]
fn hong_kong_boundaries_split_open_legs_in_both_directions() {
    let mainland = ChinaMainlandBoundary::default();
    let hk = FirBoundary::from_rings(vec![vec![
        (100., 20.),
        (110., 20.),
        (110., 40.),
        (100., 40.),
    ]])
    .unwrap();
    for fixes in [
        vec![fix(95.), fix(100.), fix(105.), fix(110.), fix(115.)],
        vec![fix(115.), fix(110.), fix(105.), fix(100.), fix(95.)],
    ] {
        let legs = route(&fixes);
        let segments = RouteSegmenter::new(&mainland)
            .with_hong_kong_boundary(&hk)
            .segment(&legs);
        assert_partition(&legs, &segments);
        assert_eq!(
            segments
                .iter()
                .map(|s| (s.kind, s.route.start_index, s.route.end_index()))
                .collect::<Vec<_>>(),
            vec![
                (SegmentKind::International, 0, 1),
                (SegmentKind::HongKong, 1, 3),
                (SegmentKind::International, 3, 4)
            ]
        );
        let dto = serde_json::to_value(crate::modules::flight::dto::FlightRouteDto::from_segments(
            &segments,
        ))
        .unwrap();
        assert_eq!(dto["segments"][1]["kind"], "hong_kong");
    }
}

#[test]
fn hong_kong_reentry_unknown_chains_and_disconnections_preserve_partition() {
    let mainland = ChinaMainlandBoundary::default();
    let hk = FirBoundary::from_rings(vec![vec![
        (100., 20.),
        (110., 20.),
        (110., 40.),
        (100., 40.),
    ]])
    .unwrap();
    let mut legs = route(&[
        fix(105.),
        AnyFix::Unknown("GAP".into()),
        fix(110.),
        fix(115.),
        fix(110.),
        fix(105.),
    ]);
    legs.extend(route(&[fix(106.), fix(100.), fix(95.)]));
    let segments = RouteSegmenter::new(&mainland)
        .with_hong_kong_boundary(&hk)
        .segment(&legs);
    assert_partition(&legs, &segments);
    assert_eq!(
        segments
            .iter()
            .map(|s| (s.kind, s.route.start_index, s.route.end_index()))
            .collect::<Vec<_>>(),
        vec![
            (SegmentKind::HongKong, 0, 2),
            (SegmentKind::International, 2, 4),
            (SegmentKind::HongKong, 4, 5),
            (SegmentKind::HongKong, 5, 6),
            (SegmentKind::International, 6, 7)
        ]
    );
}

#[tokio::test]
#[ignore = "requires VATPRC_NAVDATA_PATH pointing to the current full nav database"]
async fn aip_hong_kong_routes_cut_at_published_exits() {
    use crate::modules::flight::flight_plan::parse_route;
    use crate::modules::navdata::service::NavdataService;
    let path = std::env::var("VATPRC_NAVDATA_PATH").expect("set VATPRC_NAVDATA_PATH");
    let navdata = NavdataService::with_preferred_routes_path(&path, "data/routes.db")
        .await
        .unwrap();
    // ENR 1.10 section 6 is an assertion oracle only. No named exit list is
    // used by the implementation; all classifications come from navdata UF.
    for (raw, exit) in [
        ("VHHH BEKOL A461 IDUMA", "BEKOL"),
        ("VHHH LEKEN V601 DOTMI A470 BEBEM", "DOTMI"),
        ("VHHH LEKEN V611 LELIM M503 LAPUG", "LELIM"),
        ("VHHH DALOL V621 ELATO A1 ANPOG", "ELATO"),
        ("VHHH DALOL V631 ENVAR M750 DADON", "ENVAR"),
        ("VHHH DALOL V642 NOMAN M501 MIKIN", "NOMAN"),
        ("VHHH DALOL V642 SKATE DCT KAPLI G86 RECON", "KAPLI"),
        ("VHHH DALOL V652 SABNO A583 DALRA", "SABNO"),
        ("VHHH PECAN V10 SIKOU R339 GIVIV", "SIKOU"),
        ("VHHH PECAN V11 IDOSI A1 IKELA A1 LENKO", "IKELA"),
        ("VHHH PECAN V11 IDOSI P901 IKELA A1 LENKO", "IKELA"),
        ("VHHH PECAN V12 EPDOS L642 EPKAL", "EPKAL"),
        ("VHHH PECAN V12 EPDOS L642 EPKAL L642 EGEMU", "EPKAL"),
    ] {
        let parsed = parse_route(&navdata, raw).await.unwrap();
        let segments = RouteSegmenter::new(navdata.china_boundaries())
            .with_hong_kong_boundary(navdata.hong_kong_boundary())
            .segment(&parsed.legs);
        assert_partition(&parsed.legs, &segments);
        let hk = segments
            .iter()
            .find(|s| s.kind == SegmentKind::HongKong)
            .expect(raw);
        assert_eq!(
            hk.route.legs.last().unwrap().to.identifier(),
            Some(exit),
            "{raw}: {segments:?}"
        );
    }
}

#[tokio::test]
#[ignore = "requires VATPRC_NAVDATA_PATH pointing to the current full nav database"]
async fn aip_hong_kong_arrivals_and_transits_cut_at_published_entries() {
    use crate::modules::flight::flight_plan::parse_route;
    use crate::modules::navdata::service::NavdataService;
    let path = std::env::var("VATPRC_NAVDATA_PATH").expect("set VATPRC_NAVDATA_PATH");
    let navdata = NavdataService::with_preferred_routes_path(&path, "data/routes.db")
        .await
        .unwrap();
    for (raw, entry, exit) in [
        ("BEBEM A470 DOTMI V512 ABBEY VHHH", "DOTMI", "VHHH"),
        ("LAPUG M503 LELIM V591 ABBEY VHHH", "LELIM", "VHHH"),
        ("ANPOG A1 ELATO V522 ABBEY VHHH", "ELATO", "VHHH"),
        ("MIKIN M501 NOMAN V533 BETTY VHHH", "NOMAN", "VHHH"),
        ("DALRA A583 SABNO V542 BETTY VHHH", "SABNO", "VHHH"),
        (
            "ATBUD M772 ASOBA M772 DULOP Q1 CARSO V551 BETTY VHHH",
            "ASOBA",
            "VHHH",
        ),
        ("LENKO A1 IKELA A1 IDOSI V561 CANTO VHHH", "IKELA", "VHHH"),
        ("GIVIV R339 SIKOU V571 CANTO VHHH", "SIKOU", "VHHH"),
        ("ANPOG A1 ELATO A1 IKELA A1 LENKO", "ELATO", "IKELA"),
        ("ATBUD M772 ASOBA M772 DULOP M771 DOSUT", "ASOBA", "DOSUT"),
    ] {
        let parsed = parse_route(&navdata, raw).await.unwrap();
        let segments = RouteSegmenter::new(navdata.china_boundaries())
            .with_hong_kong_boundary(navdata.hong_kong_boundary())
            .segment(&parsed.legs);
        assert_partition(&parsed.legs, &segments);
        let hk = segments
            .iter()
            .find(|s| s.kind == SegmentKind::HongKong)
            .expect(raw);
        assert_eq!(
            hk.route.legs.first().unwrap().from.identifier(),
            Some(entry),
            "{raw}"
        );
        assert_eq!(
            hk.route.legs.last().unwrap().to.identifier(),
            Some(exit),
            "{raw}"
        );
    }
}

#[test]
fn hong_kong_geometry_preserves_operational_mainland_coverage() {
    let mainland = boundary();
    let hk = FirBoundary::from_rings(vec![vec![
        (100., 20.),
        (110., 20.),
        (110., 40.),
        (100., 40.),
    ]])
    .unwrap();
    let legs = route(&[
        airport("ZAAA", 100.),
        fix(105.),
        airport("ZBBB", 110.),
        fix(115.),
    ]);
    let segments = RouteSegmenter::new(&mainland)
        .with_hong_kong_boundary(&hk)
        .segment(&legs);
    assert_partition(&legs, &segments);
    assert_eq!(
        segments
            .iter()
            .map(|s| (s.kind, s.route.start_index, s.route.end_index()))
            .collect::<Vec<_>>(),
        vec![(SegmentKind::Mainland, 0, 3)]
    );
}
