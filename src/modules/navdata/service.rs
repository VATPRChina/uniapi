use arrayvec::{ArrayString, CapacityError};
use sqlx::{SqlitePool, prelude::FromRow};

use tracing::instrument;

use crate::modules::navdata::models::{
    Airport, AnyFix, DirectionRestriction, Ndb, NdbKind, ResolvedLeg, Vhf, Waypoint, WaypointKind,
};
use crate::modules::navdata::repository::{
    PreferredRouteRepository, PreferredRouteRepositoryError,
};

mod airway;
mod fix_record;
mod ndb;
mod procedure;
mod sid;
mod star;
mod vhf;
mod waypoint;

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

#[derive(Debug, Clone, FromRow)]
struct AirportRecord {
    airport_identifier: String,
    airport_ref_latitude: f64,
    airport_ref_longitude: f64,
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

#[cfg(test)]
mod test {
    use super::*;

    const LOCAL_DATA_PATH: &str = "data/navdata.db?mode=ro";
    const PREFERRED_ROUTES_PATH: &str = "assets/test/routes.csv";

    async fn get_navdata_adapter() -> NavdataService {
        NavdataService::with_preferred_routes_path(LOCAL_DATA_PATH, PREFERRED_ROUTES_PATH)
            .await
            .unwrap()
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
}
