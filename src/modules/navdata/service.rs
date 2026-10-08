use arrayvec::CapacityError;
use sqlx::SqlitePool;

use crate::modules::navdata::repository::{
    PreferredRouteRepository, PreferredRouteRepositoryError,
};

mod airport;
mod airway;
mod deprecated;
mod fix_record;
mod ndb;
mod procedure;
mod sid;
mod star;
mod vhf;
mod waypoint;

pub use procedure::FindProcedureMode;

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
    #[error("procedure record missing fix identifier")]
    ProcedureRecordMissingFixIdentifier,
    #[error(
        "{boundary} {transition:?} at sequence {seqno} has non-fix termination {path_termination:?}"
    )]
    ProcedureEndpointNotFix {
        boundary: &'static str,
        transition: String,
        seqno: u32,
        path_termination: String,
    },
    #[error("procedure has no {0} transition or usable common-route fallback")]
    ProcedureMissingTransition(&'static str),
    #[error("runway transitions do not share the same connection fix")]
    ProcedureRunwayEndpointsDiffer,
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
