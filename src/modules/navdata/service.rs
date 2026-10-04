mod expansion;

use arrayvec::{ArrayString, CapacityError};
use itertools::Itertools;
use ordered_float::NotNan;
use sqlx::{SqlitePool, prelude::FromRow};

use tracing::instrument;

use crate::modules::navdata::models::{
    Airport, Airway, AnyFix, DirectionRestriction, Fix, Ndb, NdbKind, ResolvedLeg, Vhf, Waypoint,
    WaypointKind,
};
use crate::modules::navdata::repository::{
    PreferredRouteRepository, PreferredRouteRepositoryError,
};

pub type NavdataResult<T> = Result<T, InvalidNavdataError>;

#[derive(Debug, thiserror::Error)]
pub enum InvalidNavdataError {
    #[error("string in navdata is too long")]
    StringTooLong,
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("invalid navaid: latitude/longitude is null")]
    InvalidNavaidNullLatLong,
    #[error("failed to compute distance ordering: {0}")]
    GeoDistanceOrderingError(#[from] ordered_float::FloatIsNan),
    #[error("preferred route data error: {0}")]
    PreferredRoute(#[from] PreferredRouteRepositoryError),
    #[error("internal error: {0}")]
    InternalError(&'static str),
}

impl From<arrayvec::CapacityError<&str>> for InvalidNavdataError {
    fn from(_: CapacityError<&str>) -> Self {
        InvalidNavdataError::StringTooLong
    }
}

#[derive(Clone)]
pub struct NavdataService {
    pub db: SqlitePool,
    preferred_routes: PreferredRouteRepository,
}

impl NavdataService {
    pub async fn with_preferred_routes_path(
        local_data_path: impl AsRef<str>,
        preferred_routes_path: impl AsRef<std::path::Path>,
    ) -> NavdataResult<Self> {
        let local_data_path = local_data_path.as_ref();
        let db = SqlitePool::connect(&format!("sqlite:{local_data_path}")).await?;
        let preferred_routes = PreferredRouteRepository::from_csv_path(preferred_routes_path)?;
        Ok(Self {
            db,
            preferred_routes,
        })
    }

    pub async fn resolve_identifier(&self, ident: &str) -> NavdataResult<Vec<ResolvedIdent>> {
        let airways = self.find_airways(ident).await?;
        let fixes = self.find_fixes(ident).await?;
        Ok(airways
            .into_iter()
            .chain(fixes.into_iter().map(ResolvedIdent::Fix))
            .collect())
    }

    pub async fn find_fixes(&self, ident: &str) -> NavdataResult<Vec<AnyFix>> {
        let fixes: Vec<FindFixRecord> = sqlx::query_as(
            r#"
            SELECT
                'tbl_pa_airports' AS kind,
                icao_code AS icao_code,
                airport_identifier AS identifier,
                airport_ref_latitude AS latitude,
                airport_ref_longitude AS longitude
            FROM tbl_pa_airports
            WHERE airport_identifier = $1
            UNION SELECT
                'tbl_d_vhfnavaids' AS kind,
                icao_code AS icao_code,
                coalesce(navaid_identifier, dme_ident) AS identifier,
                coalesce(navaid_latitude, dme_latitude) AS latitude,
                coalesce(navaid_longitude, dme_longitude) AS longitude
            FROM tbl_d_vhfnavaids
            WHERE navaid_identifier = $1 OR dme_ident = $1
            UNION SELECT
                'tbl_db_enroute_ndbnavaids' AS kind,
                icao_code AS icao_code,
                navaid_identifier AS identifier,
                navaid_latitude AS latitude,
                navaid_longitude AS longitude
            FROM tbl_db_enroute_ndbnavaids
            WHERE navaid_identifier = $1
            UNION SELECT
                'tbl_pn_terminal_ndbnavaids' AS kind,
                airport_identifier AS icao_code,
                navaid_identifier AS identifier,
                navaid_latitude AS latitude,
                navaid_longitude AS longitude
            FROM tbl_pn_terminal_ndbnavaids
            WHERE navaid_identifier = $1
            UNION SELECT
                'tbl_ea_enroute_waypoints' AS kind,
                icao_code AS icao_code,
                waypoint_identifier AS identifier,
                waypoint_latitude AS latitude,
                waypoint_longitude AS longitude
            FROM tbl_ea_enroute_waypoints
            WHERE waypoint_identifier = $1
            UNION SELECT
                'tbl_pc_terminal_waypoints' AS kind,
                region_code AS icao_code,
                waypoint_identifier AS identifier,
                waypoint_latitude AS latitude,
                waypoint_longitude AS longitude
            FROM tbl_pc_terminal_waypoints
            WHERE waypoint_identifier = $1;
                    "#,
        )
        .bind(ident)
        .fetch_all(&self.db)
        .await?;
        Ok(fixes.into_iter().map(Into::into).collect())
    }

    pub async fn find_airways(&self, ident: &str) -> NavdataResult<Vec<ResolvedIdent>> {
        let airways: Vec<FindAirwayRecord> = sqlx::query_as(
            r#"
            SELECT
                'tbl_er_enroute_airways' AS kind,
                NULL AS airport_identifier,
                route_identifier AS identifier,
                json_group_array(DISTINCT waypoint_identifier) AS fix_identifiers
            FROM
                tbl_er_enroute_airways
            WHERE
                route_identifier = $1
            GROUP BY route_identifier
            UNION
            SELECT DISTINCT
                'tbl_pd_sids' AS kind,
                airport_identifier AS airport_identifier,
                procedure_identifier AS identifier,
                '[]' AS fix_identifiers
            FROM
                tbl_pd_sids
            WHERE
                procedure_identifier = $1
            UNION
            SELECT DISTINCT
                'tbl_pe_stars' AS kind,
                airport_identifier AS airport_identifier,
                procedure_identifier AS identifier,
                '[]' AS fix_identifiers
            FROM
                tbl_pe_stars
            WHERE
                procedure_identifier = $1;
            "#,
        )
        .bind(ident)
        .fetch_all(&self.db)
        .await?;
        Ok(airways.into_iter().map(Into::into).collect())
    }

    pub async fn exists_airway(&self, ident: &str) -> NavdataResult<bool> {
        Ok(sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM tbl_er_enroute_airways WHERE route_identifier = $1)",
        )
        .bind(ident)
        .fetch_one(&self.db)
        .await?)
    }

    /// List every airport with a SID of this name, once per airport.
    pub async fn list_sid_airports(&self, ident: &str) -> NavdataResult<Vec<String>> {
        Ok(sqlx::query_scalar("SELECT DISTINCT airport_identifier FROM tbl_pd_sids WHERE procedure_identifier = $1 ORDER BY airport_identifier")
            .bind(ident).fetch_all(&self.db).await?)
    }

    /// List every airport with a STAR of this name, once per airport.
    pub async fn list_star_airports(&self, ident: &str) -> NavdataResult<Vec<String>> {
        Ok(sqlx::query_scalar("SELECT DISTINCT airport_identifier FROM tbl_pe_stars WHERE procedure_identifier = $1 ORDER BY airport_identifier")
            .bind(ident).fetch_all(&self.db).await?)
    }

    #[instrument(skip(self), fields(ident = %ident))]
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
        let airport = result
            .map(|record| -> NavdataResult<Airport> {
                Ok(Airport {
                    identifier: ArrayString::from(&record.airport_identifier)?,
                    latitude: record.airport_ref_latitude,
                    longitude: record.airport_ref_longitude,
                })
            })
            .transpose()?;

        Ok(airport)
    }

    #[instrument(skip(self))]
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

        Ok(first_by_geodistance(
            result
                .into_iter()
                .filter(|vhf| vhf.airport_identifier.is_none()),
            |record| {
                Ok(Vhf {
                    icao_code: ArrayString::from(&record.icao_code)?,
                    identifier: ArrayString::from(&record.navaid_identifier)?,
                    latitude: record
                        .navaid_latitude
                        .or(record.dme_latitude)
                        .ok_or(InvalidNavdataError::InvalidNavaidNullLatLong)?,
                    longitude: record
                        .navaid_longitude
                        .or(record.dme_longitude)
                        .ok_or(InvalidNavdataError::InvalidNavaidNullLatLong)?,
                })
            },
            latitude,
            longitude,
        ))
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

        Ok(first_by_geodistance(
            result.into_iter(),
            |record| {
                Ok(Ndb {
                    icao_code: ArrayString::from(&record.icao_code)?,
                    identifier: ArrayString::from(&record.navaid_identifier)?,
                    latitude: record.navaid_latitude,
                    longitude: record.navaid_longitude,
                    kind: NdbKind::Enroute,
                })
            },
            latitude,
            longitude,
        ))
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

        Ok(first_by_geodistance(
            result.into_iter(),
            |record| {
                Ok(Waypoint {
                    icao_code: ArrayString::from(&record.icao_code)?,
                    identifier: ArrayString::from(&record.waypoint_identifier)?,
                    latitude: record.waypoint_latitude,
                    longitude: record.waypoint_longitude,
                    kind: WaypointKind::Enroute,
                })
            },
            latitude,
            longitude,
        ))
    }

    #[instrument(skip(self), fields(airport_ident = %airport_ident, ident = %ident))]
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

    #[instrument(skip(self), fields(airway_ident = %airway_ident, fix_ident = %fix_ident))]
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

    #[instrument(skip(self), fields(airway_ident = %airway_ident, from_ident = %from_ident, to_ident = %to_ident))]
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
            ORDER BY MIN(seqno) OVER (PARTITION BY area_code),
                MAX(seqno) OVER (PARTITION BY area_code),
                seqno;
            "#,
        )
        .bind(airway_ident)
        .bind(from_ident)
        .bind(to_ident)
        .fetch_all(&self.db)
        .await?;

        if result.len() < 2 {
            return Ok(vec![]);
        }

        let legs = result
            .iter()
            .tuple_windows()
            .flat_map(|(prev, record)| {
                record
                    .to_leg(prev)
                    .inspect_err(|e| tracing::warn!("skipping invalid airway leg record: {e}"))
                    .ok()
                    .flatten()
            })
            .collect();
        Ok(legs)
    }

    pub async fn list_preferred_routes(
        &self,
        departure: &str,
        arrival: &str,
    ) -> NavdataResult<Vec<&crate::modules::navdata::models::PreferredRoute>> {
        Ok(self
            .preferred_routes
            .list_preferred_routes(departure, arrival))
    }
}

fn first_by_geodistance<T, F>(
    items: impl Iterator<Item = T>,
    map: impl Fn(T) -> Result<F, InvalidNavdataError>,
    latitude: f64,
    longitude: f64,
) -> Option<F>
where
    F: Fix,
{
    items
        .map(|item| {
            map(item).and_then(|fix| {
                Ok((
                    NotNan::new(geo_distance_ordering(
                        latitude,
                        longitude,
                        fix.latitude(),
                        fix.longitude(),
                    ))?,
                    fix,
                ))
            })
        })
        .flat_map(|fix| {
            if let Err(e) = &fix {
                tracing::warn!("skipping invalid fix record: {e}");
            }
            fix
        })
        .min_by_key(|(ord, _)| *ord)
        .map(|(_, fix)| fix)
}

#[derive(Debug, PartialEq)]
pub enum ResolvedIdent {
    Fix(AnyFix),
    Airway(Airway),
    Sid(ArrayString<4>, ArrayString<8>),
    Star(ArrayString<4>, ArrayString<8>),
}

impl ResolvedIdent {
    pub fn as_fix(&self) -> Option<&AnyFix> {
        match self {
            ResolvedIdent::Fix(f) => Some(f),
            _ => None,
        }
    }

    pub fn into_fix(self) -> Option<AnyFix> {
        match self {
            ResolvedIdent::Fix(f) => Some(f),
            _ => None,
        }
    }
}

#[derive(FromRow)]
struct FindFixRecord {
    kind: String,
    icao_code: String,
    identifier: String,
    latitude: f64,
    longitude: f64,
}

impl From<FindFixRecord> for AnyFix {
    fn from(val: FindFixRecord) -> Self {
        match val.kind.as_str() {
            "tbl_pa_airports" => AnyFix::Airport(Airport {
                identifier: val.identifier.as_str().try_into().unwrap(),
                latitude: val.latitude,
                longitude: val.longitude,
            }),
            "tbl_d_vhfnavaids" => AnyFix::Vhf(Vhf {
                icao_code: val.icao_code.as_str().try_into().unwrap(),
                identifier: val.identifier.as_str().try_into().unwrap(),
                latitude: val.latitude,
                longitude: val.longitude,
            }),
            "tbl_db_enroute_ndbnavaids" => AnyFix::Ndb(Ndb {
                icao_code: val.icao_code.as_str().try_into().unwrap(),
                identifier: val.identifier.as_str().try_into().unwrap(),
                latitude: val.latitude,
                longitude: val.longitude,
                kind: NdbKind::Enroute,
            }),
            "tbl_pn_terminal_ndbnavaids" => AnyFix::Ndb(Ndb {
                icao_code: val.icao_code.as_str().try_into().unwrap(),
                identifier: val.identifier.as_str().try_into().unwrap(),
                latitude: val.latitude,
                longitude: val.longitude,
                kind: NdbKind::Terminal,
            }),
            "tbl_ea_enroute_waypoints" => AnyFix::Waypoint(Waypoint {
                icao_code: val.icao_code.as_str().try_into().unwrap(),
                identifier: val.identifier.as_str().try_into().unwrap(),
                latitude: val.latitude,
                longitude: val.longitude,
                kind: WaypointKind::Enroute,
            }),
            "tbl_pc_terminal_waypoints" => AnyFix::Waypoint(Waypoint {
                icao_code: val.icao_code.as_str().try_into().unwrap(),
                identifier: val.identifier.as_str().try_into().unwrap(),
                latitude: val.latitude,
                longitude: val.longitude,
                kind: WaypointKind::Terminal,
            }),
            k => unreachable!("unexpected kind: {}", k),
        }
    }
}

#[derive(FromRow)]
struct FindAirwayRecord {
    kind: String,
    airport_identifier: Option<String>,
    identifier: String,
    fix_identifiers: sqlx::types::Json<Vec<ArrayString<7>>>,
}

impl From<FindAirwayRecord> for ResolvedIdent {
    fn from(val: FindAirwayRecord) -> Self {
        match val.kind.as_str() {
            "tbl_er_enroute_airways" => ResolvedIdent::Airway(Airway {
                identifier: val.identifier.as_str().try_into().unwrap(),
                fix_identifiers: val.fix_identifiers.0.into_iter().collect(),
            }),
            "tbl_pd_sids" => ResolvedIdent::Sid(
                val.airport_identifier
                    .as_deref()
                    .expect("SID must have an airport identifier")
                    .try_into()
                    .unwrap(),
                val.identifier.as_str().try_into().unwrap(),
            ),
            "tbl_pe_stars" => ResolvedIdent::Star(
                val.airport_identifier
                    .as_deref()
                    .expect("STAR must have an airport identifier")
                    .try_into()
                    .unwrap(),
                val.identifier.as_str().try_into().unwrap(),
            ),
            k => unreachable!("unexpected kind: {}", k),
        }
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
    fn to_leg(&self, prev: &Self) -> NavdataResult<Option<ResolvedLeg>> {
        if prev.waypoint_description_code.chars().nth(1) == Some('E') {
            return Ok(None);
        }

        let leg = ResolvedLeg {
            identifier: Some(self.route_identifier.clone()),
            from: prev.to_fix()?,
            to: self.to_fix()?,
            is_unknown: false,
            is_sid: false,
            is_star: false,
            direction_restriction: match self.direction_restriction.as_str() {
                "F" => DirectionRestriction::Forward,
                "B" => DirectionRestriction::Backward,
                _ => DirectionRestriction::None,
            },
        };
        Ok(Some(leg))
    }

    fn to_fix(&self) -> NavdataResult<AnyFix> {
        let fix = match self.waypoint_ref_table.trim() {
            "EA" => AnyFix::Waypoint(Waypoint {
                icao_code: ArrayString::from(&self.icao_code)?,
                identifier: ArrayString::from(&self.waypoint_identifier)?,
                latitude: self.waypoint_latitude,
                longitude: self.waypoint_longitude,
                kind: WaypointKind::Enroute,
            }),
            "DB" => AnyFix::Ndb(Ndb {
                icao_code: ArrayString::from(&self.icao_code)?,
                identifier: ArrayString::from(&self.waypoint_identifier)?,
                latitude: self.waypoint_latitude,
                longitude: self.waypoint_longitude,
                kind: NdbKind::Enroute,
            }),
            "D" => AnyFix::Vhf(Vhf {
                icao_code: ArrayString::from(&self.icao_code)?,
                identifier: ArrayString::from(&self.waypoint_identifier)?,
                latitude: self.waypoint_latitude,
                longitude: self.waypoint_longitude,
            }),
            _ => {
                return Err(InvalidNavdataError::InternalError(
                    "unsupported airway waypoint reference table",
                ));
            }
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

    const LOCAL_DATA_PATH: &str = "data/navdata.db";
    const PREFERRED_ROUTES_PATH: &str = "assets/test/routes.csv";

    async fn get_navdata_adapter() -> NavdataService {
        NavdataService::with_preferred_routes_path(LOCAL_DATA_PATH, PREFERRED_ROUTES_PATH)
            .await
            .unwrap()
    }

    async fn get_readonly_navdata_adapter() -> NavdataService {
        NavdataService::with_preferred_routes_path(
            format!("{LOCAL_DATA_PATH}?mode=ro"),
            PREFERRED_ROUTES_PATH,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_resolve_identifier() {
        let adapter = get_readonly_navdata_adapter().await;
        let resolved = adapter.resolve_identifier("DOM").await.unwrap();
        assert_eq!(resolved.len(), 2);
        assert!(resolved.iter().any(|item| matches!(item,
            ResolvedIdent::Fix(AnyFix::Vhf(vhf)) if vhf.identifier.as_str() == "DOM" && vhf.icao_code.as_str() == "TD"
        )));
        assert!(resolved.iter().any(|item| matches!(item,
            ResolvedIdent::Fix(AnyFix::Ndb(ndb)) if ndb.identifier.as_str() == "DOM" && ndb.kind == NdbKind::Enroute
        )));
        let airway = adapter.resolve_identifier("L453").await.unwrap();
        assert!(
            matches!(airway.as_slice(), [ResolvedIdent::Airway(airway)] if airway.identifier.as_str() == "L453")
        );
        let sid = adapter.resolve_identifier("GTK2A").await.unwrap();
        assert!(
            matches!(sid.as_slice(), [ResolvedIdent::Sid(airport, ident)] if airport == "MBAC" && ident == "GTK2A")
        );
        let star = adapter.resolve_identifier("ANTE2D").await.unwrap();
        assert_eq!(star.len(), 2);
        assert!(star.iter().any(|item| matches!(item, ResolvedIdent::Star(airport, ident) if airport == "MDLR" && ident == "ANTE2D")));
        assert!(star.iter().any(|item| matches!(item, ResolvedIdent::Sid(airport, ident) if airport == "MMUN" && ident == "ANTE2D")));
        assert!(
            adapter
                .resolve_identifier("UNKNOWN")
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn test_find_fixes() {
        let adapter = get_readonly_navdata_adapter().await;
        let airports = adapter.find_fixes("MBAC").await.unwrap();
        let [AnyFix::Airport(airport)] = airports.as_slice() else {
            panic!("expected MBAC airport");
        };
        assert_eq!(airport.identifier.as_str(), "MBAC");
        approx::assert_abs_diff_eq!(airport.latitude, 21.3006333333333, epsilon = 1e-9);
        approx::assert_abs_diff_eq!(airport.longitude, -71.64115, epsilon = 1e-9);

        // DOM has both an enroute NDB and a VHF record using DME coordinates.
        let fixes = adapter.find_fixes("DOM").await.unwrap();
        assert_eq!(fixes.len(), 2);
        for fix in &fixes {
            assert_eq!(fix.identifier(), Some("DOM"));
            assert_eq!(fix.icao_code(), Some("TD"));
            match fix {
                AnyFix::Vhf(vhf) => {
                    approx::assert_abs_diff_eq!(vhf.latitude, 15.5505555555556, epsilon = 1e-9);
                    approx::assert_abs_diff_eq!(vhf.longitude, -61.2955555555556, epsilon = 1e-9);
                }
                AnyFix::Ndb(ndb) => {
                    assert_eq!(ndb.kind, NdbKind::Enroute);
                    approx::assert_abs_diff_eq!(ndb.latitude, 15.5509333333333, epsilon = 1e-9);
                    approx::assert_abs_diff_eq!(ndb.longitude, -61.295625, epsilon = 1e-9);
                }
                _ => panic!("unexpected DOM fix"),
            }
        }
        let ndbs = adapter.find_fixes("GD").await.unwrap();
        let [AnyFix::Ndb(ndb)] = ndbs.as_slice() else {
            panic!("expected GD terminal NDB");
        };
        assert_eq!(ndb.kind, NdbKind::Terminal);
        assert_eq!(ndb.icao_code.as_str(), "MMGL");
        approx::assert_abs_diff_eq!(ndb.latitude, 20.4685916666667, epsilon = 1e-9);
        approx::assert_abs_diff_eq!(ndb.longitude, -103.174683333333, epsilon = 1e-9);

        let waypoints = adapter.find_fixes("AC07F").await.unwrap();
        let [AnyFix::Waypoint(waypoint)] = waypoints.as_slice() else {
            panic!("expected AC07F terminal waypoint");
        };
        assert_eq!(waypoint.kind, WaypointKind::Terminal);
        assert_eq!(waypoint.icao_code.as_str(), "MBAC");
        approx::assert_abs_diff_eq!(waypoint.latitude, 21.260175, epsilon = 1e-9);
        approx::assert_abs_diff_eq!(waypoint.longitude, -71.7191222222222, epsilon = 1e-9);

        // This identifier is reused in eight regions; retain every candidate.
        let waypoints = adapter.find_fixes("VP001").await.unwrap();
        let mut regions: Vec<_> = waypoints
            .iter()
            .map(|fix| {
                let AnyFix::Waypoint(waypoint) = fix else {
                    panic!("expected enroute waypoint");
                };
                assert_eq!(waypoint.identifier.as_str(), "VP001");
                assert_eq!(waypoint.kind, WaypointKind::Enroute);
                waypoint.icao_code.as_str()
            })
            .collect();
        regions.sort_unstable();
        assert_eq!(regions, ["MB", "MD", "MM", "MU", "TI", "TJ", "TL", "TT"]);
        assert!(adapter.find_fixes("UNKNOWN").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_find_airways() {
        let adapter = get_readonly_navdata_adapter().await;
        // L453 has multiple rows, but resolves to one airway.
        let airways = adapter.find_airways("L453").await.unwrap();
        assert!(
            matches!(airways.as_slice(), [ResolvedIdent::Airway(airway)] if airway.identifier.as_str() == "L453")
        );
        let ResolvedIdent::Airway(airway) = &airways[0] else {
            panic!("expected L453 airway");
        };
        let mut identifiers: Vec<_> = airway
            .fix_identifiers
            .iter()
            .map(ArrayString::as_str)
            .collect();
        identifiers.sort_unstable();
        assert_eq!(identifiers, ["ASIVO", "MACKI"]);
        let asivo = adapter.find_fixes("ASIVO").await.unwrap().remove(0);
        assert!(airway.contains_fix(&asivo));
        let airport = adapter.find_fixes("MBAC").await.unwrap().remove(0);
        assert!(!airway.contains_fix(&airport));
        for (identifier, expected) in [
            ("GTK2A", vec![("sid", "MBAC")]),
            ("ANTE2D", vec![("sid", "MMUN"), ("star", "MDLR")]),
            ("ANEG1A", vec![("sid", "MMCU"), ("sid", "MMSD")]),
            ("BERO1B", vec![("star", "MMIO"), ("star", "TNCC")]),
        ] {
            let procedures = adapter.find_airways(identifier).await.unwrap();
            let mut actual: Vec<_> = procedures
                .iter()
                .map(|item| match item {
                    ResolvedIdent::Sid(airport, ident) => {
                        assert_eq!(ident.as_str(), identifier);
                        ("sid", airport.as_str())
                    }
                    ResolvedIdent::Star(airport, ident) => {
                        assert_eq!(ident.as_str(), identifier);
                        ("star", airport.as_str())
                    }
                    _ => panic!("unexpected procedure kind for {identifier}"),
                })
                .collect();
            actual.sort_unstable();
            assert_eq!(actual, expected);
        }
        assert!(adapter.find_airways("DOM").await.unwrap().is_empty());
        assert!(adapter.find_airways("UNKNOWN").await.unwrap().is_empty());
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
    async fn test_list_airway_legs_between_present2() {
        let adapter = get_navdata_adapter().await;
        sqlx::query(r#"
            WITH rows("area_code","crusing_table_identifier","direction_restriction","flightlevel","icao_code","inbound_course","inbound_distance","maximum_altitude","minimum_altitude1","minimum_altitude2","outbound_course","route_identifier_postfix","route_identifier","route_type","seqno","waypoint_description_code","waypoint_identifier","waypoint_latitude","waypoint_longitude","waypoint_ref_table") AS (
                VALUES
                ('PAC','XX',NULL,'B','VH',0,31.6,NULL,8000,NULL,400,NULL,'A470','O',5510,'E   ','MAGOG',22.296111111111113,115.82500000000002,'EA'),
                ('EEU','EE',NULL,'B','ZG',0,17.3,NULL,NULL,NULL,180,NULL,'A470','R',5520,'E C ','DOTMI',22.718333333333334,116.16833333333334,'EA'),
                ('PAC',NULL,NULL,'B','ZG',400,0,NULL,NULL,NULL,0,NULL,'A470','O',5520,'EEC ','DOTMI',22.718333333333334,116.16833333333334,'EA'),
                ('EEU','EE',NULL,'B','ZG',400,36.7,NULL,NULL,NULL,400,NULL,'A470','R',5521,'E C ','BEBEM',22.95,116.36055555555555,'EA')
            )
            INSERT INTO "tbl_er_enroute_airways"("area_code","crusing_table_identifier","direction_restriction","flightlevel","icao_code","inbound_course","inbound_distance","maximum_altitude","minimum_altitude1","minimum_altitude2","outbound_course","route_identifier_postfix","route_identifier","route_type","seqno","waypoint_description_code","waypoint_identifier","waypoint_latitude","waypoint_longitude","waypoint_ref_table")
            SELECT *
            FROM rows
            WHERE NOT EXISTS (
                SELECT 1
                FROM "tbl_er_enroute_airways"
                WHERE "area_code" IS rows."area_code"
                    AND "icao_code" IS rows."icao_code"
                    AND "route_identifier_postfix" IS rows."route_identifier_postfix"
                    AND "route_identifier" IS rows."route_identifier"
                    AND "route_type" IS rows."route_type"
                    AND "seqno" IS rows."seqno"
                    AND "waypoint_identifier" IS rows."waypoint_identifier"
                    AND "waypoint_ref_table" IS rows."waypoint_ref_table"
            );
            "#).execute(&adapter.db).await.unwrap();
        let legs = adapter
            .list_airway_legs_between("A470", "BEBEM", "DOTMI")
            .await
            .unwrap();
        assert_eq!(legs.len(), 1);

        let leg = &legs[0];
        assert_eq!(leg.identifier.as_deref(), Some("A470"));
        assert_eq!(leg.direction_restriction, DirectionRestriction::None);
        assert_eq!(leg.from.identifier(), Some("DOTMI"));
        assert_eq!(leg.to.identifier(), Some("BEBEM"));
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
