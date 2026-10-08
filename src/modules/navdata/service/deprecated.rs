use arrayvec::ArrayString;
use itertools::Itertools;
use ordered_float::NotNan;
use sqlx::prelude::FromRow;
use tracing::instrument;

use crate::modules::navdata::models::*;
use crate::modules::navdata::service::InvalidNavdataError;
use crate::modules::navdata::service::NavdataResult;
use crate::modules::navdata::service::NavdataService;

impl NavdataService {
    #[deprecated]
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

    #[deprecated]
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

    #[deprecated]
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

    #[deprecated]
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

    #[deprecated]
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

    #[deprecated]
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

    #[deprecated]
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

    #[deprecated]
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

fn geo_distance_ordering(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let dlat = lat2 - lat1;

    let mut dlon = lon2 - lon1;
    dlon = (dlon + 180.0).rem_euclid(360.0) - 180.0;

    let mean_lat = ((lat1 + lat2) / 2.0).to_radians();
    let x = dlon * mean_lat.cos();
    let y = dlat;

    x * x + y * y
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
            kind: LegKind::Airway,
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
