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
        ("ZSPD PONAB DCT BEGMO DCT LELIM VHHH", "ZSPD-VHHH-01", 0..3),
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
        // Both airport-pair rows are eligible regardless of AIP remarks;
        // border definitions are excluded from matching and suggestions.
        assert!(
            report
                .preferred_routes
                .iter()
                .all(|route| route.departure == "ZSPD" && route.arrival == "VHHH"
                    || route.departure == "VHHH" && route.arrival == "ZSPD")
        );
        assert_eq!(
            report.preferred_routes.len(),
            if expected_name.starts_with("ZSPD") {
                2
            } else {
                1
            }
        );
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

#[tokio::test]
async fn border_definitions_are_used_when_the_airport_pair_has_no_definitions() {
    let directory = std::env::temp_dir().join(format!("border-fallback-{}", ulid::Ulid::new()));
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
    sqlx::query("DELETE FROM Routes WHERE Dep = 'ZSPD' AND Arr = 'VHHH'")
        .execute(&db)
        .await
        .unwrap();
    db.close().await;
    let navdata = NavdataService::with_preferred_routes_path(path.to_str().unwrap(), &path)
        .await
        .unwrap();
    let matcher = PreferredRouteMatcher::new(&navdata);
    for (raw, matches) in [
        ("ZSPD PONAB W13 BEGMO M503 LELIM VHHH", true),
        ("ZSPD PONAB DCT LELIM VHHH", false),
    ] {
        let route = parse_route(&navdata, raw).await.unwrap();
        let segments = RouteSegmenter::new(navdata.china_boundaries()).segment(&route.legs);
        let mainland = segments
            .iter()
            .find(|segment| segment.kind == SegmentKind::Mainland)
            .unwrap();
        let report = matcher.inspect_segment(mainland).await.unwrap();
        assert_eq!(report.preferred_routes.len(), 1);
        assert_eq!(report.preferred_routes[0].name, "ZSPD-LELIM-01");
        assert_eq!(report.selected().is_some(), matches);
        assert_eq!(
            matcher.match_segment(mainland).await.unwrap().is_some(),
            matches
        );
    }
    drop(navdata);
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn airport_pair_definitions_reject_a_matching_border_route_via_abbey() {
    use crate::modules::flight::dto::TemporaryFlightQuery;
    use crate::modules::flight::flight_plan::validator::validate_route;
    use crate::modules::flight::models::{
        Flight, ValidationProblem, ValidatorIdentifier, ValidatorStatus,
    };

    let directory = std::env::temp_dir().join(format!("preferred-priority-{}", ulid::Ulid::new()));
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
    sqlx::raw_sql(
        "DELETE FROM Routes; INSERT INTO tbl_pa_airports VALUES ('ZBAA',40.073,116.598);",
    )
    .execute(&db)
    .await
    .unwrap();
    // Keep the mainland route outside the synthetic Hong Kong FIR until DOTMI.
    let fixes = [
        "ELKUR", "PANKI", "AR", "ATVIM", "HFE", "XLN", "DOTMI", "SIERA",
    ];
    for (index, ident) in fixes.iter().enumerate() {
        sqlx::query("INSERT INTO tbl_ea_enroute_waypoints VALUES ('ZS',?,?,116)")
            .bind(ident)
            .bind(39.0 - index as f64 * 2.0)
            .execute(&db)
            .await
            .unwrap();
    }
    for (index, airway) in ["W40", "W158", "H104", "W127", "H2", "A470", "V512"]
        .iter()
        .enumerate()
    {
        for (seqno, ident, lat) in [
            (1, fixes[index], 39.0 - index as f64 * 2.0),
            (
                2,
                if index == 6 {
                    "ABBEY"
                } else {
                    fixes[index + 1]
                },
                if index == 6 {
                    22.5
                } else {
                    37.0 - index as f64 * 2.0
                },
            ),
        ] {
            sqlx::query(
                "INSERT INTO tbl_er_enroute_airways VALUES ('EEU','',?,?,?,'',?,?,116,'EA')",
            )
            .bind(if ident == "ABBEY" { "VH" } else { "ZS" })
            .bind(airway)
            .bind(seqno)
            .bind(ident)
            .bind(lat)
            .execute(&db)
            .await
            .unwrap();
        }
    }
    sqlx::raw_sql(
        r#"INSERT INTO Routes VALUES
        ('ZBAA','VHHH','','','ZBAA-VHHH-STANDARD','SO','','','SIERA',''),
        ('ZBAA','','','DOTMI','ZBAA-DOTMI-FALLBACK','SO','','',
            'ELKUR W40 PANKI W158 AR H104 ATVIM W127 HFE H2 XLN A470 DOTMI',''),
        ('','VHHH','DOTMI','','DOTMI-VHHH','FE','','','DOTMI V512 ABBEY','');"#,
    )
    .execute(&db)
    .await
    .unwrap();
    db.close().await;
    let navdata = NavdataService::with_preferred_routes_path(path.to_str().unwrap(), &path)
        .await
        .unwrap();
    let matcher = PreferredRouteMatcher::new(&navdata);
    let flight = Flight::from(
        serde_json::from_value::<TemporaryFlightQuery>(serde_json::json!({
            "departure": "ZBAA", "arrival": "VHHH", "aircraft": "B738", "cruising_level": 33100
        }))
        .unwrap(),
    );
    for (raw, expected_status) in [
        (
            "ZBAA ELKUR W40 PANKI W158 AR H104 ATVIM W127 HFE H2 XLN A470 DOTMI V512 ABBEY VHHH",
            ValidatorStatus::Rejected,
        ),
        ("ZBAA SIERA VHHH", ValidatorStatus::Pass),
    ] {
        let route = parse_route(&navdata, raw).await.unwrap();
        assert!(route.legs.iter().all(|leg| !leg.is_unknown));
        let segments = RouteSegmenter::new(navdata.china_boundaries())
            .with_hong_kong_boundary(navdata.hong_kong_boundary())
            .segment(&route.legs);
        let mainland = segments
            .iter()
            .find(|segment| segment.kind == SegmentKind::Mainland)
            .unwrap();
        let report = matcher.inspect_segment(mainland).await.unwrap();
        assert_eq!(report.candidates.len(), 1);
        assert_eq!(report.preferred_routes.len(), 1);
        assert_eq!(report.preferred_routes[0].name, "ZBAA-VHHH-STANDARD");
        assert_eq!(
            matcher.preferred_routes(mainland).await.unwrap()[0].name,
            "ZBAA-VHHH-STANDARD"
        );
        assert_eq!(
            matcher.match_segment(mainland).await.unwrap().is_some(),
            expected_status == ValidatorStatus::Pass
        );
        let checks = validate_route(&flight, &route, &navdata).await.unwrap();
        let check = checks
            .iter()
            .find(|check| check.id == ValidatorIdentifier::PreferredRoute)
            .unwrap();
        assert_eq!(check.status, expected_status);
        if expected_status == ValidatorStatus::Rejected {
            let mut border_only = *mainland;
            border_only.airport_pair = None;
            assert_eq!(
                matcher
                    .match_segment(&border_only)
                    .await
                    .unwrap()
                    .unwrap()
                    .name,
                "ZBAA-DOTMI-FALLBACK"
            );
            let [ValidationProblem::NotPreferredRoute { routes }] = check.problems.as_slice()
            else {
                panic!("expected designated-route rejection");
            };
            assert_eq!(routes.len(), 1);
            assert_eq!(routes[0].to, "VHHH");
            assert_eq!(routes[0].raw_route, "SIERA");
        }
    }
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
                        cruising_level_restriction: matched.cruising_level_restriction,
                        allowed_altitudes: matched.allowed_altitudes.clone(),
                        minimal_altitude: matched.minimal_altitude,
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

#[tokio::test]
async fn hong_kong_retrieves_departure_definition_and_rejects_a_direct_shortcut() {
    use crate::modules::flight::dto::TemporaryFlightQuery;
    use crate::modules::flight::flight_plan::validator::validate_route;
    use crate::modules::flight::models::{
        Flight, ValidationField, ValidationProblem, ValidatorIdentifier, ValidatorStatus,
    };

    let directory = std::env::temp_dir().join(format!("hk-zggg-definition-{}", ulid::Ulid::new()));
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
    sqlx::raw_sql(
        r#"
        DELETE FROM Routes;
        UPDATE tbl_uf_fir_uir SET fir_uir_latitude = 18 WHERE fir_uir_latitude = 21;
        INSERT INTO tbl_pa_airports VALUES ('ZGGG',23.392,113.310),('WSSS',1.35,103.99);
        INSERT INTO tbl_ea_enroute_waypoints VALUES
            ('ZG','VIBOS',23,113.4),('ZG','SIERA',21.985,113.553),
            ('VH','MULET',21.5,114),('VH','SKATE',20.5,115),('VH','SABNO',18.985,115.845),
            ('VH','DALRA',17,116);
        INSERT INTO tbl_er_enroute_airways VALUES
            ('EEU','','ZG','R473',1,'','VIBOS',23,113.4,'EA'),
            ('EEU','','ZG','R473',2,'','SIERA',21.985,113.553,'EA'),
            ('PAC','','VH','V652',1,'','SKATE',20.5,115,'EA'),
            ('PAC','','VH','V652',2,'','SABNO',18.985,115.845,'EA');
        INSERT INTO Routes VALUES
            ('ZGGG','','','SABNO','ZGGG-Dep-4','','','','SIERA DCT MULET DCT SKATE V652 SABNO',
             'A583; ENR 1.10 8.2(4)');
    "#,
    )
    .execute(&db)
    .await
    .unwrap();
    db.close().await;
    let navdata = NavdataService::with_preferred_routes_path(path.to_str().unwrap(), &path)
        .await
        .unwrap();
    let matcher = PreferredRouteMatcher::new(&navdata);
    let flight = Flight::from(
        serde_json::from_value::<TemporaryFlightQuery>(serde_json::json!({
            "departure": "ZGGG", "arrival": "WSSS", "aircraft": "B738", "cruising_level": 32000
        }))
        .unwrap(),
    );
    for (raw, expected_status) in [
        ("ZGGG VIBOS R473 SIERA DCT SABNO", ValidatorStatus::Rejected),
        (
            "ZGGG VIBOS R473 SIERA DCT SABNO DALRA WSSS",
            ValidatorStatus::Rejected,
        ),
        (
            "ZGGG VIBOS R473 SIERA DCT MULET DCT SKATE V652 SABNO",
            ValidatorStatus::Pass,
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
        let definitions = matcher.preferred_routes(hk).await.unwrap();
        assert_eq!(definitions.len(), 1, "{raw}");
        assert_eq!(definitions[0].name, "ZGGG-Dep-4");
        let report = matcher.inspect_segment(hk).await.unwrap();
        assert_eq!(
            report.selected().is_some(),
            expected_status == ValidatorStatus::Pass,
            "{raw}"
        );
        let checks = validate_route(&flight, &route, &navdata).await.unwrap();
        let check = checks
            .iter()
            .find(|check| {
                check.id == ValidatorIdentifier::PreferredRoute
                    && check.field
                        == ValidationField::MultipleLegs {
                            indices: hk.route.start_index..hk.route.end_index(),
                        }
            })
            .unwrap();
        assert_eq!(check.status, expected_status, "{raw}");
        if expected_status == ValidatorStatus::Rejected {
            let [ValidationProblem::NotPreferredRoute { routes }] = check.problems.as_slice()
            else {
                panic!(
                    "expected the Hong Kong shortcut to be checked against its departure definition"
                );
            };
            assert_eq!(routes.len(), 1);
            assert_eq!(routes[0].from, "ZGGG");
            assert_eq!(routes[0].to, "SABNO");
            assert_eq!(routes[0].raw_route, "SIERA DCT MULET DCT SKATE V652 SABNO");
        }
    }
    // Airport context belongs to one connected route, not to later disconnected legs.
    let route = parse_route(&navdata, "ZGGG VIBOS R473 SIERA DCT SABNO")
        .await
        .unwrap();
    let mut disconnected = route.legs.clone();
    disconnected.remove(1);
    let segments = RouteSegmenter::new(navdata.china_boundaries())
        .with_hong_kong_boundary(navdata.hong_kong_boundary())
        .segment(&disconnected);
    let hk = segments
        .iter()
        .find(|segment| segment.kind == SegmentKind::HongKong)
        .unwrap();
    assert!(matcher.preferred_routes(hk).await.unwrap().is_empty());
    drop(navdata);
    std::fs::remove_dir_all(directory).unwrap();
}

fn matching_fixture() -> Vec<ResolvedLeg> {
    use crate::modules::navdata::models::{Airport, DirectionRestriction, Waypoint, WaypointKind};
    let waypoint = |identifier: &str, longitude: f64| {
        AnyFix::Waypoint(Waypoint {
            icao_code: "ZB".try_into().unwrap(),
            identifier: identifier.try_into().unwrap(),
            latitude: 38.0.try_into().unwrap(),
            longitude: longitude.try_into().unwrap(),
            kind: WaypointKind::Enroute,
        })
    };
    let airport = |identifier: &str, longitude: f64| {
        AnyFix::Airport(Airport {
            identifier: identifier.try_into().unwrap(),
            latitude: 38.0.try_into().unwrap(),
            longitude: longitude.try_into().unwrap(),
        })
    };
    let fixes = [
        airport("ZBYN", 112.),
        waypoint("ISLEP", 113.),
        waypoint("TONOV", 114.),
        waypoint("AMVOG", 115.),
        waypoint("NUDKU", 116.),
        waypoint("DUGEB", 117.),
        airport("ZBAA", 118.),
    ];
    fixes
        .windows(2)
        .zip([
            None,
            Some("G212"),
            Some("G212"),
            Some("W61"),
            Some("W56"),
            None,
        ])
        .map(|(pair, airway)| ResolvedLeg {
            from: pair[0].clone(),
            to: pair[1].clone(),
            identifier: airway.map(str::to_owned),
            kind: if airway.is_some() {
                LegKind::Airway
            } else {
                LegKind::Direct
            },
            is_unknown: false,
            direction_restriction: DirectionRestriction::Forward,
        })
        .collect()
}

#[test]
fn airport_connections_cannot_skip_preferred_route_endpoints() {
    let expected = matching_fixture();
    assert!(routes_match(&expected, &expected));
    // ZBYN DCT TONOV omits ISLEP G212 TONOV: an airport anchor alone
    // must not authorize matching the preferred route's remaining suffix.
    let mut departure = expected.clone();
    departure[0].to = departure[1].to.clone();
    departure.remove(1);
    assert!(!routes_match(&departure, &expected));
    // Apply the same rule to an arrival connecting early to the airport.
    let mut arrival = expected.clone();
    arrival[5].from = arrival[4].from.clone();
    arrival.remove(4);
    assert!(!routes_match(&arrival, &expected));
}

#[test]
fn only_resolved_filed_procedures_allow_preferred_route_overlap() {
    let expected = matching_fixture();
    for (kind, index) in [(LegKind::Sid, 0), (LegKind::Star, 4)] {
        let mut actual = expected.clone();
        if kind == LegKind::Sid {
            actual[0].to = actual[1].to.clone();
            actual.remove(1);
        } else {
            actual[5].from = actual[4].from.clone();
            actual.remove(4);
        }
        actual[index].kind = kind;
        actual[index].identifier = Some("PROC1".into());
        assert!(
            routes_match(&actual, &expected),
            "resolved {kind:?} should permit overlap"
        );
        actual[index].is_unknown = true;
        assert!(
            !routes_match(&actual, &expected),
            "unresolved {kind:?} must not permit overlap"
        );
    }
}

#[test]
fn procedure_overlap_supports_airport_connections_but_not_border_anchors() {
    let expected = matching_fixture();
    let mut actual = expected.clone();
    // An airport connection followed by a resolved SID is still a filed
    // procedure. Here ISLEP is the SID's initial fix and TONOV its exit.
    actual[1].kind = LegKind::Sid;
    actual[1].identifier = Some("PROC1".into());
    assert!(routes_match(&actual, &expected));
    actual[0].is_unknown = true;
    assert!(!routes_match(&actual, &expected));
    actual[0].is_unknown = false;
    // Removing the airport makes this a border slice: its start cannot move
    // from ISLEP to TONOV even though the first leg is a resolved SID.
    assert!(!routes_match(&actual[1..], &expected[1..]));
}
