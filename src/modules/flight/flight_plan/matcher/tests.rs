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
        let segments = RouteSegmenter::new(navdata.china_boundaries()).segment(&route.legs);
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
    let segments = RouteSegmenter::new(navdata.china_boundaries()).segment(&route.legs);
    let report = matcher.inspect_segment(&segments[0]).await.unwrap();
    assert!(!report.preferred_routes.is_empty());
    assert!(report.selected().is_none());
    drop(navdata);
    std::fs::remove_dir_all(directory).unwrap();
}
