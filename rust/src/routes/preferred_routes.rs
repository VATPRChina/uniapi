use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{auth::CurrentUser, models::user_role::UserRole, services::Services};

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

async fn list_preferred_routes(current_user: CurrentUser) -> Result<Response, PreferredRouteError> {
    require_role(&current_user, UserRole::Volunteer)?;
    Err(PreferredRouteError::NotImplemented)
}

async fn get_preferred_route(
    current_user: CurrentUser,
    Path(_id): Path<String>,
) -> Result<Response, PreferredRouteError> {
    require_role(&current_user, UserRole::Volunteer)?;
    Err(PreferredRouteError::NotImplemented)
}

async fn create_preferred_route(
    current_user: CurrentUser,
    Json(_request): Json<PreferredRouteSaveRequest>,
) -> Result<Response, PreferredRouteError> {
    require_role(&current_user, UserRole::EventCoordinator)?;
    Err(PreferredRouteError::NotImplemented)
}

async fn update_preferred_route(
    current_user: CurrentUser,
    Path(_id): Path<String>,
    Json(_request): Json<PreferredRouteSaveRequest>,
) -> Result<Response, PreferredRouteError> {
    require_role(&current_user, UserRole::EventCoordinator)?;
    Err(PreferredRouteError::NotImplemented)
}

async fn delete_preferred_route(
    current_user: CurrentUser,
    Path(_id): Path<String>,
) -> Result<Response, PreferredRouteError> {
    require_role(&current_user, UserRole::EventCoordinator)?;
    Err(PreferredRouteError::NotImplemented)
}

fn require_role(current_user: &CurrentUser, role: UserRole) -> Result<(), PreferredRouteError> {
    if current_user.has_role(role) {
        Ok(())
    } else {
        Err(PreferredRouteError::Forbidden)
    }
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum LevelRestrictionType {
    StandardEven,
    StandardOdd,
    Standard,
    FlightLevelEven,
    FlightLevelOdd,
    FlightLevel,
}

#[derive(Debug)]
enum PreferredRouteError {
    Forbidden,
    NotImplemented,
}

impl IntoResponse for PreferredRouteError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            PreferredRouteError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            PreferredRouteError::NotImplemented => {
                (StatusCode::NOT_IMPLEMENTED, "not implemented".into())
            }
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}
