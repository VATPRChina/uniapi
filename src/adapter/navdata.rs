use arrayvec::ArrayString;
use itertools::Itertools;
use ordered_float::NotNan;
use sqlx::{SqlitePool, prelude::FromRow};

use crate::{
    flight_plan::PreferredRoute,
    model::navdata::{
        Airport, AnyFix, DirectionRestriction, Ndb, NdbKind, ResolvedLeg, Vhf, Waypoint,
        WaypointKind,
    },
};

pub type NavdataResult<T> = Result<T, anyhow::Error>;

#[derive(Clone)]
pub struct NavdataAdapter {
    pub db: SqlitePool,
}

// TODO: do not use unwrap
impl NavdataAdapter {
    pub async fn new(
        remote_data_url: impl AsRef<str>,
        local_data_path: impl AsRef<str>,
        download_file: bool,
    ) -> Self {
        if download_file {
            let response = reqwest::get(remote_data_url.as_ref()).await.unwrap();
            let bytes = response.bytes().await.unwrap();
            std::fs::write(local_data_path.as_ref(), bytes).unwrap();
        }

        let local_data_path = std::fs::canonicalize(local_data_path.as_ref())
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let db = SqlitePool::connect(&format!("sqlite:{local_data_path}"))
            .await
            .unwrap();
        Self { db }
    }

    pub async fn find_airport(&self, ident: &str) -> NavdataResult<Option<Airport>> {
        let result: Option<AirportRecord> = sqlx::query_as(
            r#"
            SELECT airport_identifier, airport_ref_latitude, airport_ref_longitude
            FROM tbl_pa_airports
            WHERE airport_identifier = $1;
            "#,
        )
        .bind(ident)
        .fetch_optional(&self.db)
        .await?;
        let airport = result.map(|record| Airport {
            identifier: ArrayString::from(&record.airport_identifier).unwrap(),
            latitude: record.airport_ref_latitude,
            longitude: record.airport_ref_longitude,
        });

        Ok(airport)
    }

    pub async fn find_nearest_fix(
        &self,
        latitude: f64,
        longitude: f64,
        ident: &str,
    ) -> NavdataResult<Option<AnyFix>> {
        let vhf = self.find_nearest_vhf(latitude, longitude, ident).await?;
        if let Some(vhf) = vhf {
            return Ok(Some(AnyFix::Vhf(vhf)));
        }

        let ndb = self
            .find_nearest_enroute_ndb(latitude, longitude, ident)
            .await?;
        if let Some(ndb) = ndb {
            return Ok(Some(AnyFix::Ndb(ndb)));
        }

        let waypoint = self
            .find_nearest_enroute_waypoint(latitude, longitude, ident)
            .await?;
        if let Some(waypoint) = waypoint {
            return Ok(Some(AnyFix::Waypoint(waypoint)));
        }

        Ok(None)
    }

    pub async fn find_nearest_vhf(
        &self,
        latitude: f64,
        longitude: f64,
        ident: &str,
    ) -> NavdataResult<Option<Vhf>> {
        let result: Vec<VhfRecord> = sqlx::query_as(
            r#"
            SELECT airport_identifier,
                dme_ident,
                dme_latitude,
                dme_longitude,
                icao_code,
                navaid_identifier,
                navaid_latitude,
                navaid_longitude
            FROM tbl_d_vhfnavaids
            WHERE navaid_identifier = $1
                OR dme_ident = $1;
            "#,
        )
        .bind(ident)
        .fetch_all(&self.db)
        .await?;
        let vhf = result
            .into_iter()
            .filter(|vhf| vhf.airport_identifier.is_none())
            .map(|record| Vhf {
                icao_code: ArrayString::from(&record.icao_code).unwrap(),
                identifier: ArrayString::from(&record.navaid_identifier).unwrap(),
                latitude: record.navaid_latitude.or(record.dme_latitude).unwrap(),
                longitude: record.navaid_longitude.or(record.dme_longitude).unwrap(),
            })
            .min_by_key(|vhf| {
                NotNan::new(geo_distance_ordering(
                    latitude,
                    longitude,
                    vhf.latitude,
                    vhf.longitude,
                ))
                .unwrap()
            });

        Ok(vhf)
    }

    pub async fn find_nearest_enroute_ndb(
        &self,
        latitude: f64,
        longitude: f64,
        ident: &str,
    ) -> NavdataResult<Option<Ndb>> {
        let result: Vec<EnrouteNdbRecord> = sqlx::query_as(
            r#"
            SELECT
                icao_code,
                navaid_identifier,
                navaid_latitude,
                navaid_longitude
            FROM tbl_db_enroute_ndbnavaids
            WHERE navaid_identifier = $1;
            "#,
        )
        .bind(ident)
        .fetch_all(&self.db)
        .await?;

        let ndb = result
            .into_iter()
            .map(|record| Ndb {
                icao_code: ArrayString::from(&record.icao_code).unwrap(),
                identifier: ArrayString::from(&record.navaid_identifier).unwrap(),
                latitude: record.navaid_latitude,
                longitude: record.navaid_longitude,
                kind: NdbKind::Enroute,
            })
            .min_by_key(|ndb| {
                NotNan::new(geo_distance_ordering(
                    latitude,
                    longitude,
                    ndb.latitude,
                    ndb.longitude,
                ))
                .unwrap()
            });

        Ok(ndb)
    }

    pub async fn find_nearest_enroute_waypoint(
        &self,
        latitude: f64,
        longitude: f64,
        ident: &str,
    ) -> NavdataResult<Option<Waypoint>> {
        let result: Vec<EnrouteWaypointRecord> = sqlx::query_as(
            r#"
            SELECT
                icao_code,
                waypoint_identifier,
                waypoint_latitude,
                waypoint_longitude
            FROM tbl_ea_enroute_waypoints
            WHERE waypoint_identifier = $1;
            "#,
        )
        .bind(ident)
        .fetch_all(&self.db)
        .await?;

        let waypoint = result
            .into_iter()
            .map(|record| Waypoint {
                icao_code: ArrayString::from(&record.icao_code).unwrap(),
                identifier: ArrayString::from(&record.waypoint_identifier).unwrap(),
                latitude: record.waypoint_latitude,
                longitude: record.waypoint_longitude,
                kind: WaypointKind::Enroute,
            })
            .min_by_key(|waypoint| {
                NotNan::new(geo_distance_ordering(
                    latitude,
                    longitude,
                    waypoint.latitude,
                    waypoint.longitude,
                ))
                .unwrap()
            });

        Ok(waypoint)
    }

    pub async fn exists_sid(&self, airport_ident: &str, ident: &str) -> NavdataResult<bool> {
        let result: u32 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM tbl_pd_sids
            WHERE airport_identifier = $1
                AND procedure_identifier = $2;
            "#,
        )
        .bind(airport_ident)
        .bind(ident)
        .fetch_one(&self.db)
        .await?;

        Ok(result > 0)
    }

    pub async fn exists_star(&self, airport_ident: &str, ident: &str) -> NavdataResult<bool> {
        let result: u32 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM tbl_pe_stars
            WHERE airport_identifier = $1
                AND procedure_identifier = $2;
            "#,
        )
        .bind(airport_ident)
        .bind(ident)
        .fetch_one(&self.db)
        .await?;

        Ok(result > 0)
    }

    pub async fn exists_airway_with_fix(
        &self,
        airway_ident: &str,
        fix_ident: &str,
    ) -> NavdataResult<bool> {
        let result: u32 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM tbl_er_enroute_airways
            WHERE route_identifier = $1
                AND waypoint_identifier = $2;
            "#,
        )
        .bind(airway_ident)
        .bind(fix_ident)
        .fetch_one(&self.db)
        .await?;

        Ok(result > 0)
    }

    pub async fn list_airway_legs_between(
        &self,
        airway_ident: &str,
        from_ident: &str,
        to_ident: &str,
    ) -> NavdataResult<Vec<ResolvedLeg>> {
        let result: Vec<EnrouteAirwayRecord> = sqlx::query_as(
            r#"
            WITH boundary AS (
                SELECT seqno
                FROM tbl_er_enroute_airways
                WHERE route_identifier = $1
                    AND (waypoint_identifier = $2 OR waypoint_identifier = $3)
            )
            SELECT
                area_code,
                direction_restriction,
                icao_code,
                route_identifier,
                seqno,
                waypoint_description_code,
                waypoint_identifier,
                waypoint_latitude,
                waypoint_longitude,
                waypoint_ref_table
            FROM tbl_er_enroute_airways
            WHERE route_identifier = $1
                AND seqno >= (SELECT min(seqno) FROM boundary) 
                AND seqno <= (SELECT max(seqno) FROM boundary)
            ORDER BY area_code, seqno;
            "#,
        )
        .bind(airway_ident)
        .bind(from_ident)
        .bind(to_ident)
        .fetch_all(&self.db)
        .await?;

        // TODO: validate result size

        let legs = result
            .iter()
            .tuple_windows()
            .flat_map(|(prev, record)| record.to_leg(prev))
            .collect();
        Ok(legs)
    }

    pub async fn list_preferred_routes(&self) -> NavdataResult<Vec<PreferredRoute>> {
        // TODO: list preferred routes
        Ok(vec![])
    }
}

#[derive(Debug, Clone, FromRow)]
struct AirportRecord {
    airport_identifier: String,
    airport_ref_latitude: f64,
    airport_ref_longitude: f64,
}

#[derive(Debug, Clone, FromRow)]
struct VhfRecord {
    airport_identifier: Option<String>,
    #[allow(unused)]
    dme_ident: Option<String>,
    dme_latitude: Option<f64>,
    dme_longitude: Option<f64>,
    icao_code: String,
    navaid_identifier: String,
    navaid_latitude: Option<f64>,
    navaid_longitude: Option<f64>,
}

#[derive(Debug, Clone, FromRow)]
struct EnrouteNdbRecord {
    icao_code: String,
    navaid_identifier: String,
    navaid_latitude: f64,
    navaid_longitude: f64,
}

#[derive(Debug, Clone, FromRow)]
struct EnrouteWaypointRecord {
    icao_code: String,
    waypoint_identifier: String,
    waypoint_latitude: f64,
    waypoint_longitude: f64,
}

#[derive(Debug, Clone, FromRow)]
struct EnrouteAirwayRecord {
    #[allow(unused)]
    area_code: String,
    direction_restriction: String,
    icao_code: String,
    route_identifier: String,
    #[allow(unused)]
    seqno: u32,
    waypoint_description_code: String,
    waypoint_identifier: String,
    waypoint_latitude: f64,
    waypoint_longitude: f64,
    waypoint_ref_table: String,
}

impl EnrouteAirwayRecord {
    fn to_leg(&self, prev: &Self) -> Option<ResolvedLeg> {
        if prev.waypoint_description_code.chars().nth(1) == Some('E') {
            return None;
        }

        let leg = ResolvedLeg {
            identifier: Some(self.route_identifier.clone()),
            from: match prev.waypoint_ref_table.as_str() {
                "EA" => AnyFix::Waypoint(Waypoint {
                    icao_code: ArrayString::from(&prev.icao_code).unwrap(),
                    identifier: ArrayString::from(&prev.waypoint_identifier).unwrap(),
                    latitude: prev.waypoint_latitude,
                    longitude: prev.waypoint_longitude,
                    kind: WaypointKind::Enroute,
                }),
                "DB" => AnyFix::Ndb(Ndb {
                    icao_code: ArrayString::from(&prev.icao_code).unwrap(),
                    identifier: ArrayString::from(&prev.waypoint_identifier).unwrap(),
                    latitude: prev.waypoint_latitude,
                    longitude: prev.waypoint_longitude,
                    kind: NdbKind::Enroute,
                }),
                "D " => AnyFix::Vhf(Vhf {
                    icao_code: ArrayString::from(&prev.icao_code).unwrap(),
                    identifier: ArrayString::from(&prev.waypoint_identifier).unwrap(),
                    latitude: prev.waypoint_latitude,
                    longitude: prev.waypoint_longitude,
                }),
                _ => unimplemented!(
                    "Unknown waypoint reference table '{}'",
                    prev.waypoint_ref_table
                ),
            },
            to: self.to_fix().unwrap(),
            direction_restriction: match self.direction_restriction.as_str() {
                "F" => DirectionRestriction::Forward,
                "B" => DirectionRestriction::Backward,
                _ => DirectionRestriction::None,
            },
        };
        Some(leg)
    }

    fn to_fix(&self) -> NavdataResult<AnyFix> {
        let fix = match self.waypoint_ref_table.as_str() {
            "EA" => AnyFix::Waypoint(Waypoint {
                icao_code: ArrayString::from(&self.icao_code).unwrap(),
                identifier: ArrayString::from(&self.waypoint_identifier).unwrap(),
                latitude: self.waypoint_latitude,
                longitude: self.waypoint_longitude,
                kind: WaypointKind::Enroute,
            }),
            "DB" => AnyFix::Ndb(Ndb {
                icao_code: ArrayString::from(&self.icao_code).unwrap(),
                identifier: ArrayString::from(&self.waypoint_identifier).unwrap(),
                latitude: self.waypoint_latitude,
                longitude: self.waypoint_longitude,
                kind: NdbKind::Enroute,
            }),
            "D " => AnyFix::Vhf(Vhf {
                icao_code: ArrayString::from(&self.icao_code).unwrap(),
                identifier: ArrayString::from(&self.waypoint_identifier).unwrap(),
                latitude: self.waypoint_latitude,
                longitude: self.waypoint_longitude,
            }),
            _ => unimplemented!(
                "Unknown waypoint reference table '{}'",
                self.waypoint_ref_table
            ),
        };
        Ok(fix)
    }
}

fn geo_distance_ordering(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let dlat = lat2 - lat1;

    let mut dlon = lon2 - lon1;
    dlon = (dlon + 180.0).rem_euclid(360.0) - 180.0;

    let mean_lat = ((lat1 + lat2) / 2.0).to_radians();
    let x = dlon * mean_lat.cos();
    let y = dlat;

    x * x + y * y
}

#[cfg(test)]
mod test {
    use super::*;

    const DFD_V2_SAMPLE_DATA_URL: &str = "https://developers.navigraph.com/downloads/navigation-data/navigraph-dfd-sample-sqlite-dfdv2-2401.zip";
    const LOCAL_DATA_PATH: &str = "test_data/ng_jeppesen_fwdfd_2401.s3db";

    async fn get_navdata_adapter() -> NavdataAdapter {
        NavdataAdapter::new(
            DFD_V2_SAMPLE_DATA_URL.to_string(),
            LOCAL_DATA_PATH.to_string(),
            false,
        )
        .await
    }

    #[tokio::test]
    async fn test_navdata_adapter() {
        let adapter = get_navdata_adapter().await;
        assert!(adapter.db.acquire().await.is_ok());
    }

    #[tokio::test]
    async fn test_find_airport_present() {
        let adapter = get_navdata_adapter().await;
        let airport = adapter.find_airport("MBAC").await.unwrap();
        assert!(airport.is_some());
        let airport = airport.unwrap();
        assert_eq!(&airport.identifier, "MBAC");
        approx::assert_relative_eq!(airport.latitude, 21.3006333333333, max_relative = 1e-6);
        approx::assert_relative_eq!(airport.longitude, -71.64115, max_relative = 1e-6);
    }

    #[tokio::test]
    async fn test_find_airport_absent() {
        let adapter = get_navdata_adapter().await;
        let airport = adapter.find_airport("ZSPD").await.unwrap();
        assert!(airport.is_none());
    }

    #[tokio::test]
    async fn test_find_nearest_vhf_present() {
        let adapter = get_navdata_adapter().await;
        let vhf = adapter
            .find_nearest_vhf(15.55, -61.29, "DOM")
            .await
            .unwrap();
        assert!(vhf.is_some());
        let vhf = vhf.unwrap();
        assert_eq!(vhf.identifier.as_str(), "DOM");
        assert_eq!(vhf.icao_code.as_str(), "TD");
        approx::assert_relative_eq!(vhf.latitude, 15.5505555555556, max_relative = 1e-6);
        approx::assert_relative_eq!(vhf.longitude, -61.2955555555556, max_relative = 1e-6);
    }

    #[tokio::test]
    async fn test_find_nearest_vhf_absent() {
        let adapter = get_navdata_adapter().await;
        let vhf = adapter.find_nearest_vhf(0.0, 0.0, "INVL").await.unwrap();
        assert!(vhf.is_none());
    }

    #[tokio::test]
    async fn test_find_nearest_enroute_ndb_present() {
        let adapter = get_navdata_adapter().await;
        let ndb = adapter
            .find_nearest_enroute_ndb(15.55, -61.29, "DOM")
            .await
            .unwrap();
        assert!(ndb.is_some());
        let ndb = ndb.unwrap();
        assert_eq!(ndb.identifier.as_str(), "DOM");
        assert_eq!(ndb.icao_code.as_str(), "TD");
        assert_eq!(ndb.kind, NdbKind::Enroute);
        approx::assert_relative_eq!(ndb.latitude, 15.5509333333333, max_relative = 1e-6);
        approx::assert_relative_eq!(ndb.longitude, -61.295625, max_relative = 1e-6);
    }

    #[tokio::test]
    async fn test_find_nearest_enroute_ndb_absent() {
        let adapter = get_navdata_adapter().await;
        let ndb = adapter
            .find_nearest_enroute_ndb(0.0, 0.0, "INVL")
            .await
            .unwrap();
        assert!(ndb.is_none());
    }

    #[tokio::test]
    async fn test_find_nearest_enroute_waypoint_present() {
        let adapter = get_navdata_adapter().await;
        let waypoint = adapter
            .find_nearest_enroute_waypoint(18.3, -66.2, "VP001")
            .await
            .unwrap();
        assert!(waypoint.is_some());
        let waypoint = waypoint.unwrap();
        assert_eq!(waypoint.identifier.as_str(), "VP001");
        assert_eq!(waypoint.icao_code.as_str(), "MD");
        assert_eq!(waypoint.kind, WaypointKind::Enroute);
        approx::assert_relative_eq!(waypoint.latitude, 18.3041527777778, max_relative = 1e-6);
        approx::assert_relative_eq!(waypoint.longitude, -66.2452083333333, max_relative = 1e-6);
    }

    #[tokio::test]
    async fn test_find_nearest_enroute_waypoint_absent() {
        let adapter = get_navdata_adapter().await;
        let waypoint = adapter
            .find_nearest_enroute_waypoint(0.0, 0.0, "INVL")
            .await
            .unwrap();
        assert!(waypoint.is_none());
    }

    #[tokio::test]
    async fn test_find_nearest_fix_prefers_vhf() {
        let adapter = get_navdata_adapter().await;
        let fix = adapter
            .find_nearest_fix(15.55, -61.29, "DOM")
            .await
            .unwrap();
        assert!(matches!(fix, Some(AnyFix::Vhf(_))));
    }

    #[tokio::test]
    async fn test_find_nearest_fix_falls_back_to_waypoint() {
        let adapter = get_navdata_adapter().await;
        let fix = adapter
            .find_nearest_fix(18.3, -66.2, "VP001")
            .await
            .unwrap();
        let Some(AnyFix::Waypoint(waypoint)) = fix else {
            panic!("expected waypoint fix");
        };
        assert_eq!(waypoint.identifier.as_str(), "VP001");
        assert_eq!(waypoint.icao_code.as_str(), "MD");
    }

    #[tokio::test]
    async fn test_find_nearest_fix_absent() {
        let adapter = get_navdata_adapter().await;
        let fix = adapter.find_nearest_fix(0.0, 0.0, "INVL").await.unwrap();
        assert!(fix.is_none());
    }

    #[tokio::test]
    async fn test_exists_sid_present() {
        let adapter = get_navdata_adapter().await;
        let exists = adapter.exists_sid("MBAC", "GTK2A").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_exists_sid_absent() {
        let adapter = get_navdata_adapter().await;
        let exists = adapter.exists_sid("MBAC", "INVL2D").await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_exists_star_present() {
        let adapter = get_navdata_adapter().await;
        let exists = adapter.exists_star("MDLR", "ANTE2D").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_exists_star_absent() {
        let adapter = get_navdata_adapter().await;
        let exists = adapter.exists_star("MDLR", "INVL2D").await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_exists_airway_with_fix_present() {
        let adapter = get_navdata_adapter().await;
        let exists = adapter.exists_airway_with_fix("A312", "DOM").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_exists_airway_with_fix_absent() {
        let adapter = get_navdata_adapter().await;
        let exists = adapter
            .exists_airway_with_fix("A312", "INVL")
            .await
            .unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_list_airway_legs_between_present() {
        let adapter = get_navdata_adapter().await;
        let legs = adapter
            .list_airway_legs_between("L453", "MACKI", "ASIVO")
            .await
            .unwrap();
        assert_eq!(legs.len(), 1);

        let leg = &legs[0];
        assert_eq!(leg.identifier.as_deref(), Some("L453"));
        assert_eq!(leg.direction_restriction, DirectionRestriction::None);
        assert_eq!(leg.from.identifier(), Some("MACKI"));
        assert_eq!(leg.to.identifier(), Some("ASIVO"));
        assert!(matches!(leg.from, AnyFix::Waypoint(_)));
        assert!(matches!(leg.to, AnyFix::Waypoint(_)));
    }

    #[tokio::test]
    async fn test_list_airway_legs_between_absent_airway() {
        let adapter = get_navdata_adapter().await;
        let legs = adapter
            .list_airway_legs_between("INVL", "MACKI", "ASIVO")
            .await
            .unwrap();
        assert!(legs.is_empty());
    }

    #[tokio::test]
    async fn test_list_airway_legs_between_absent_fix() {
        let adapter = get_navdata_adapter().await;
        let legs = adapter
            .list_airway_legs_between("L453", "INVL", "ASIVO")
            .await
            .unwrap();
        assert!(legs.is_empty());
    }
}
