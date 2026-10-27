use super::*;
use crate::modules::flight::flight_plan::RouteSegmenter;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

const FIXTURE: &str = r#"
CREATE TABLE tbl_pa_airports (airport_identifier TEXT, airport_ref_latitude REAL, airport_ref_longitude REAL);
INSERT INTO tbl_pa_airports VALUES ('ZSPD',31.143,121.805),('VHHH',22.309,113.915);
CREATE TABLE tbl_ea_enroute_waypoints (icao_code TEXT, waypoint_identifier TEXT, waypoint_latitude REAL, waypoint_longitude REAL);
INSERT INTO tbl_ea_enroute_waypoints VALUES ('ZS','PONAB',30.748,122.834),('ZS','BEGMO',28,121.833),('ZS','LELIM',22.939,117.312);
CREATE TABLE tbl_pc_terminal_waypoints (region_code TEXT, waypoint_identifier TEXT, waypoint_latitude REAL, waypoint_longitude REAL);
CREATE TABLE tbl_d_vhfnavaids (icao_code TEXT, airport_identifier TEXT, navaid_identifier TEXT, dme_ident TEXT, navaid_latitude REAL, navaid_longitude REAL, dme_latitude REAL, dme_longitude REAL);
CREATE TABLE tbl_db_enroute_ndbnavaids (icao_code TEXT, navaid_identifier TEXT, navaid_latitude REAL, navaid_longitude REAL);
CREATE TABLE tbl_pn_terminal_ndbnavaids (airport_identifier TEXT, navaid_identifier TEXT, navaid_latitude REAL, navaid_longitude REAL);
CREATE TABLE tbl_pd_sids (airport_identifier TEXT, procedure_identifier TEXT, route_type TEXT, transition_identifier TEXT, seqno INTEGER, path_termination TEXT, waypoint_identifier TEXT);
CREATE TABLE tbl_pe_stars (airport_identifier TEXT, procedure_identifier TEXT, route_type TEXT, transition_identifier TEXT, seqno INTEGER, path_termination TEXT, waypoint_identifier TEXT);
CREATE TABLE tbl_er_enroute_airways (area_code TEXT, direction_restriction TEXT, icao_code TEXT, route_identifier TEXT, seqno INTEGER, waypoint_description_code TEXT, waypoint_identifier TEXT, waypoint_latitude REAL, waypoint_longitude REAL, waypoint_ref_table TEXT);
INSERT INTO tbl_er_enroute_airways VALUES
('EEU','','ZS','W13',1,'','PONAB',30.748,122.834,'EA'),
('EEU','','ZS','W13',2,'','BEGMO',28,121.833,'EA'),
('EEU','','ZS','M503',1,'','BEGMO',28,121.833,'EA'),
('EEU','','ZS','M503',2,'','LELIM',22.939,117.312,'EA');
CREATE TABLE Routes (Dep TEXT, Arr TEXT, Waypoint1 TEXT, Waypoint2 TEXT, Name TEXT, EvenOdd TEXT, AltList TEXT, MinAlt TEXT, Route TEXT, Remarks TEXT);
INSERT INTO Routes VALUES
('ZSPD','VHHH','','','ZSPD-VHHH-01','SO','','','PONAB DCT BEGMO DCT LELIM',''),
('ZSPD','VHHH','','','ZSPD-VHHH-02','SO','','','PONAB W13 BEGMO M503 LELIM','AIP route'),
('ZSPD','','','LELIM','ZSPD-LELIM-01','SO','','','PONAB W13 BEGMO M503 LELIM',''),
('VHHH','ZSPD','','','VHHH-ZSPD-02','SE','','','LELIM M503 BEGMO W13 PONAB','');
"#;

#[tokio::test]
async fn airport_pair_routes_match_only_the_mainland_coverage_in_both_directions() {
    let directory = std::env::temp_dir().join(format!("preferred-route-{}", ulid::Ulid::new()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("navdata.db");
    let db = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true),
    )
    .await
    .unwrap();
    sqlx::raw_sql(FIXTURE).execute(&db).await.unwrap();
    db.close().await;
    let navdata = NavdataService::with_preferred_routes_path(path.to_str().unwrap(), &path)
        .await
        .unwrap();
    let matcher = PreferredRouteMatcher::new(&navdata);
    for (raw_route, expected_name, expected_range) in [
        ("ZSPD PONAB W13 BEGMO M503 LELIM VHHH", "ZSPD-VHHH-02", 0..3),
        ("VHHH LELIM M503 BEGMO W13 PONAB ZSPD", "VHHH-ZSPD-02", 1..4),
    ] {
        let route = parse_route(&navdata, raw_route).await.unwrap();
        let segments = RouteSegmenter::new(navdata.china_boundaries())
            .with_hong_kong_boundary(navdata.hong_kong_boundary())
            .segment(&route.legs);
        assert_eq!(segments.len(), 2);
        let mainland = segments
            .iter()
            .find(|s| s.kind == SegmentKind::Mainland)
            .unwrap();
        let report = matcher.inspect_segment(mainland).await.unwrap();
        let selected = report
            .selected()
            .expect("airport-pair preferred route should match the China segment");
        assert_eq!(selected.preferred.name, expected_name);
        assert_eq!(selected.coverage.start_index, expected_range.start);
        assert_eq!(selected.coverage.end_index(), expected_range.end);
        assert!(
            report
                .preferred_routes
                .iter()
                .any(|preferred| preferred.name == expected_name)
        );
        // The airport pair takes priority over a public border-keyed route.
        assert_eq!(
            matcher.match_segment(mainland).await.unwrap().unwrap().name,
            expected_name
        );
        let international = segments
            .iter()
            .find(|s| s.kind == SegmentKind::International)
            .unwrap();
        assert!(
            matcher
                .match_segment(international)
                .await
                .unwrap()
                .is_none()
        );
    }
    // Matching endpoints alone does not authorize a different enroute path.
    let route = parse_route(&navdata, "ZSPD PONAB DCT LELIM VHHH")
        .await
        .unwrap();
    let segments = RouteSegmenter::new(navdata.china_boundaries())
        .with_hong_kong_boundary(navdata.hong_kong_boundary())
        .segment(&route.legs);
    let report = matcher.inspect_segment(&segments[0]).await.unwrap();
    assert!(!report.preferred_routes.is_empty());
    assert!(report.selected().is_none());
    drop(navdata);
    std::fs::remove_dir_all(directory).unwrap();
}

const HONG_KONG_FIXTURE: &str = r#"
CREATE TABLE tbl_uf_fir_uir (area_code TEXT, fir_uir_identifier TEXT, fir_uir_indicator TEXT, seqno INTEGER, boundary_via TEXT, fir_uir_latitude REAL, fir_uir_longitude REAL);
INSERT INTO tbl_uf_fir_uir VALUES
('PAC','VHHK','F',10,'H ',21,113),
('PAC','VHHK','F',20,'H ',21,118),
('PAC','VHHK','F',30,'H ',24,118),
('PAC','VHHK','F',40,'HE',24,113);
INSERT INTO tbl_ea_enroute_waypoints VALUES ('VH','LEKEN',22.1,114.4),('VH','ABBEY',22.5,116);
INSERT INTO tbl_er_enroute_airways VALUES
('PAC','','VH','V611',1,'','LEKEN',22.1,114.4,'EA'),
('PAC','','ZS','V611',2,'','LELIM',22.939,117.312,'EA'),
('PAC','','ZS','V591',1,'','LELIM',22.939,117.312,'EA'),
('PAC','','VH','V591',2,'','ABBEY',22.5,116,'EA');
INSERT INTO Routes VALUES
('VHHH','','','LELIM','HK-DEPARTURE','FE','F320','30000','LEKEN V611 LELIM',''),
('VHHH','','','LELIM','HK-DEPARTURE-02','FE','F320','30000','LEKEN DCT ABBEY DCT LELIM',''),
('','VHHH','LELIM','','HK-ARRIVAL','FE','F320','30000','LELIM V591 ABBEY',''),
('','','LEKEN','ABBEY','HK-TRANSIT','FE','F320','30000','LEKEN V611 LELIM V591 ABBEY','');
"#;

#[tokio::test]
async fn hong_kong_preferred_routes_validate_only_their_segment_coverage() {
    use crate::modules::flight::dto::TemporaryFlightQuery;
    use crate::modules::flight::flight_plan::validator::validate_route;
    use crate::modules::flight::models::{
        Flight, ValidationField, ValidatorIdentifier, ValidatorStatus,
    };

    let directory = std::env::temp_dir().join(format!("hk-preferred-{}", ulid::Ulid::new()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("navdata.db");
    let db = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true),
    )
    .await
    .unwrap();
    sqlx::raw_sql(FIXTURE).execute(&db).await.unwrap();
    sqlx::raw_sql(HONG_KONG_FIXTURE).execute(&db).await.unwrap();
    db.close().await;
    let navdata = NavdataService::with_preferred_routes_path(path.to_str().unwrap(), &path)
        .await
        .unwrap();
    let matcher = PreferredRouteMatcher::new(&navdata);
    for (raw, expected_name, departure, arrival, expected_range) in [
        (
            "VHHH LEKEN V611 LELIM M503 BEGMO W13 PONAB ZSPD",
            "HK-DEPARTURE",
            "VHHH",
            "ZSPD",
            0..2,
        ),
        (
            "ZSPD PONAB W13 BEGMO M503 LELIM V591 ABBEY VHHH",
            "HK-ARRIVAL",
            "ZSPD",
            "VHHH",
            3..5,
        ),
        (
            "LEKEN V611 LELIM V591 ABBEY",
            "HK-TRANSIT",
            "VHHH",
            "VHHH",
            0..2,
        ),
    ] {
        let route = parse_route(&navdata, raw).await.unwrap();
        let segments = RouteSegmenter::new(navdata.china_boundaries())
            .with_hong_kong_boundary(navdata.hong_kong_boundary())
            .segment(&route.legs);
        let hk = segments
            .iter()
            .find(|segment| segment.kind == SegmentKind::HongKong)
            .unwrap();
        let mut international = *hk;
        international.kind = SegmentKind::International;
        let skipped = matcher.inspect_segment(&international).await.unwrap();
        assert!(skipped.preferred_routes.is_empty());
        assert!(skipped.candidates.is_empty());
        let report = matcher.inspect_segment(hk).await.unwrap();
        assert_eq!(report.selected().unwrap().preferred.name, expected_name);
        assert_eq!(
            report.selected().unwrap().coverage.start_index,
            expected_range.start
        );
        assert_eq!(
            report.selected().unwrap().coverage.end_index(),
            expected_range.end
        );
        assert_eq!(
            matcher.match_segment(hk).await.unwrap().unwrap().name,
            expected_name
        );
        assert!(
            report
                .preferred_routes
                .iter()
                .all(|route| route.name.starts_with("HK-"))
        );
        let scope = ValidationField::MultipleLegs {
            indices: expected_range.clone(),
        };
        for level in [32000, 31000] {
            let flight = Flight::from(serde_json::from_value::<TemporaryFlightQuery>(serde_json::json!({
                "departure": departure, "arrival": arrival, "aircraft": "B738", "cruising_level": level
            })).unwrap());
            let checks = validate_route(&flight, &route, &navdata).await.unwrap();
            let preferred = checks
                .iter()
                .find(|check| {
                    check.id == ValidatorIdentifier::PreferredRoute && check.field == scope
                })
                .unwrap();
            assert_eq!(preferred.status, ValidatorStatus::Pass);
            let matched = &report.selected().unwrap().preferred;
            assert_eq!(
                preferred.problems,
                vec![
                    crate::modules::flight::models::ValidationProblem::RouteMatchPreferred {
                        from: matched.departure.clone(),
                        to: matched.arrival.clone(),
                        raw_route: matched.raw_route.clone(),
                    }
                ]
            );
            assert!(!checks.iter().any(|check| check.id
                == ValidatorIdentifier::PreferredRouteCruiseAltitude
                && check.field == scope));
            if segments
                .iter()
                .all(|segment| segment.kind != SegmentKind::Mainland)
            {
                assert!(
                    !checks
                        .iter()
                        .any(|check| check.id == ValidatorIdentifier::DirectionalCruiseAltitude)
                );
            } else {
                assert!(
                    checks
                        .iter()
                        .any(|check| check.id == ValidatorIdentifier::PreferredRouteCruiseAltitude)
                );
            }
            assert!(!checks.iter().any(|check| matches!(&check.field, ValidationField::Leg { index } if expected_range.contains(index))));
        }
    }

    // Same endpoints with a different path must be rejected, while no definition
    // for a Hong Kong segment suppresses the check instead of inventing a rule.
    for (raw, expected_status) in [
        ("VHHH LEKEN DCT LELIM", ValidatorStatus::Rejected),
        ("VHHH ABBEY", ValidatorStatus::Suppressed),
    ] {
        let route = parse_route(&navdata, raw).await.unwrap();
        let flight = Flight::from(
            serde_json::from_value::<TemporaryFlightQuery>(serde_json::json!({
                "departure": "VHHH", "arrival": "ZSPD", "aircraft": "B738", "cruising_level": 32000
            }))
            .unwrap(),
        );
        let checks = validate_route(&flight, &route, &navdata).await.unwrap();
        let preferred = checks
            .iter()
            .find(|check| check.id == ValidatorIdentifier::PreferredRoute)
            .unwrap();
        assert_eq!(preferred.status, expected_status);
        assert!(!checks.iter().any(|check| matches!(
            check.id,
            ValidatorIdentifier::PreferredRouteCruiseAltitude
                | ValidatorIdentifier::DirectionalCruiseAltitude
        )));
        if expected_status == ValidatorStatus::Rejected {
            let [crate::modules::flight::models::ValidationProblem::NotPreferredRoute { routes }] =
                preferred.problems.as_slice()
            else {
                panic!("expected one grouped preferred-route problem")
            };
            assert_eq!(routes.len(), 2);
            assert_eq!(routes[0].from, "VHHH");
            assert_eq!(routes[0].to, "LELIM");
            assert_eq!(routes[0].raw_route, "LEKEN V611 LELIM");
            assert_eq!(routes[1].raw_route, "LEKEN DCT ABBEY DCT LELIM");
        }
    }
    // A Hong Kong match must not suppress the directional altitude fallback
    // when the mainland part of a mainland departure remains unmatched.
    let route = parse_route(&navdata, "ZSPD PONAB DCT LELIM V591 ABBEY VHHH")
        .await
        .unwrap();
    let flight = Flight::from(
        serde_json::from_value::<TemporaryFlightQuery>(serde_json::json!({
            "departure": "ZSPD", "arrival": "VHHH", "aircraft": "B738", "cruising_level": 32000
        }))
        .unwrap(),
    );
    let checks = validate_route(&flight, &route, &navdata).await.unwrap();
    assert_eq!(
        checks
            .iter()
            .find(|check| check.id == ValidatorIdentifier::DirectionalCruiseAltitude)
            .unwrap()
            .status,
        ValidatorStatus::Rejected
    );

    drop(navdata);
    std::fs::remove_dir_all(directory).unwrap();
}
