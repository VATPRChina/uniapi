use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{auth::CurrentUser, models::user_role::UserRole, services::Services};

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    list_preferred_routes,
    create_preferred_route,
    get_preferred_route,
    update_preferred_route,
    delete_preferred_route
))]
pub(crate) struct ApiDoc;

pub fn build_preferred_route_routes() -> Router<Services> {
    Router::new()
        .route("/", get(list_preferred_routes).post(create_preferred_route))
        .route(
            "/{id}",
            get(get_preferred_route)
                .put(update_preferred_route)
                .delete(delete_preferred_route),
        )
}

#[utoipa::path(get, path = "api/navdata/preferred-routes", tag = "Navdata", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<PreferredRouteDto>)))]
async fn list_preferred_routes(current_user: CurrentUser) -> Result<Response, PreferredRouteError> {
    current_user
        .require_role(UserRole::Volunteer)
        .map_err(|_| PreferredRouteError::Forbidden)?;
    Err(PreferredRouteError::NotImplemented)
}

#[utoipa::path(get, path = "api/navdata/preferred-routes/{id}", tag = "Navdata", security(("oauth2" = [])), params(("id" = String, Path, description = "Preferred route ULID")), responses((status = 200, description = "Successful response", body = PreferredRouteDto)))]
async fn get_preferred_route(
    current_user: CurrentUser,
    Path(_id): Path<String>,
) -> Result<Response, PreferredRouteError> {
    current_user
        .require_role(UserRole::Volunteer)
        .map_err(|_| PreferredRouteError::Forbidden)?;
    Err(PreferredRouteError::NotImplemented)
}

#[utoipa::path(post, path = "api/navdata/preferred-routes", tag = "Navdata", security(("oauth2" = [])), request_body = PreferredRouteSaveRequest, responses((status = 200, description = "Successful response", body = PreferredRouteDto)))]
async fn create_preferred_route(
    current_user: CurrentUser,
    Json(_request): Json<PreferredRouteSaveRequest>,
) -> Result<Response, PreferredRouteError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| PreferredRouteError::Forbidden)?;
    Err(PreferredRouteError::NotImplemented)
}

#[utoipa::path(put, path = "api/navdata/preferred-routes/{id}", tag = "Navdata", security(("oauth2" = [])), params(("id" = String, Path, description = "Preferred route ULID")), request_body = PreferredRouteSaveRequest, responses((status = 200, description = "Successful response", body = PreferredRouteDto)))]
async fn update_preferred_route(
    current_user: CurrentUser,
    Path(_id): Path<String>,
    Json(_request): Json<PreferredRouteSaveRequest>,
) -> Result<Response, PreferredRouteError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| PreferredRouteError::Forbidden)?;
    Err(PreferredRouteError::NotImplemented)
}

#[utoipa::path(delete, path = "api/navdata/preferred-routes/{id}", tag = "Navdata", security(("oauth2" = [])), params(("id" = String, Path, description = "Preferred route ULID")), responses((status = 200, description = "Successful response", body = PreferredRouteDto)))]
async fn delete_preferred_route(
    current_user: CurrentUser,
    Path(_id): Path<String>,
) -> Result<Response, PreferredRouteError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| PreferredRouteError::Forbidden)?;
    Err(PreferredRouteError::NotImplemented)
}

#[derive(Deserialize, utoipa::ToSchema)]
#[allow(dead_code)]
struct PreferredRouteSaveRequest {
    departure: String,
    arrival: String,
    raw_route: String,
    cruising_level_restriction: LevelRestrictionType,
    #[serde(default)]
    allowed_altitudes: Vec<i32>,
    minimal_altitude: i32,
    remarks: String,
    valid_from: Option<DateTime<Utc>>,
    valid_until: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
enum LevelRestrictionType {
    StandardEven,
    StandardOdd,
    Standard,
    FlightLevelEven,
    FlightLevelOdd,
    FlightLevel,
}

#[derive(Serialize, utoipa::ToSchema)]
#[allow(dead_code)]
struct PreferredRouteDto {
    id: String,
    departure: String,
    arrival: String,
    raw_route: String,
    cruising_level_restriction: LevelRestrictionType,
    allowed_altitudes: Vec<i32>,
    minimal_altitude: i32,
    remarks: String,
    valid_from: Option<DateTime<Utc>>,
    valid_until: Option<DateTime<Utc>>,
}

#[derive(Debug)]
enum PreferredRouteError {
    Forbidden,
    NotImplemented,
}

impl IntoResponse for PreferredRouteError {
    fn into_response(self) -> Response {
        let (status, message): (StatusCode, String) = match self {
            PreferredRouteError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            PreferredRouteError::NotImplemented => {
                (StatusCode::NOT_IMPLEMENTED, "not implemented".into())
            }
        };

        crate::problem::problem_response(status, message)
    }
}
