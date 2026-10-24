use arrayvec::CapacityError;
use sqlx::SqlitePool;

use crate::modules::navdata::{
    models::ChinaMainlandBoundary,
    repository::{PreferredRouteRepository, PreferredRouteRepositoryError},
};

mod airport;
mod airway;
mod deprecated;
mod fir;
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
    #[error("invalid FIR boundary data: {0}")]
    InvalidFirBoundary(String),
    #[error("string in navdata is too long")]
    StringTooLong,
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("invalid navaid: latitude/longitude is null")]
    InvalidNavaidNullLatLong,
    #[error("invalid coordinate: {0}")]
    InvalidCoordinate(#[from] crate::utils::geo::NonFiniteCoordinateError),
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
    db: SqlitePool,
    routes: PreferredRouteRepository,
    china_boundaries: ChinaMainlandBoundary,
}

impl NavdataService {
    pub async fn with_preferred_routes_path(
        local_data_path: impl AsRef<str>,
        preferred_routes_path: impl AsRef<std::path::Path>,
    ) -> NavdataResult<Self> {
        let local_data_path = local_data_path.as_ref();
        let db = SqlitePool::connect(&format!("sqlite:{local_data_path}")).await?;
        let routes = PreferredRouteRepository::from_db_path(preferred_routes_path).await?;
        let china_boundaries = NavdataService::mainland_fir_boundaries(&db).await?;
        Ok(Self {
            db,
            routes,
            china_boundaries,
        })
    }

    pub async fn list_preferred_routes(
        &self,
        departure: &str,
        arrival: &str,
    ) -> NavdataResult<Vec<crate::modules::navdata::models::PreferredRoute>> {
        Ok(self
            .routes
            .list_preferred_routes(departure, arrival)
            .await?)
    }

    pub fn china_boundaries(&self) -> &ChinaMainlandBoundary {
        &self.china_boundaries
    }
}
