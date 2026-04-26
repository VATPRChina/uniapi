use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::{
    adapter::database::{
        sector as sector_repository,
        user::{self as user_repository},
    },
    auth::CurrentUser,
    services::Services,
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(current_permission))]
pub(crate) struct ApiDoc;

pub fn build_sector_routes() -> Router<Services> {
    Router::new().route("/current/permission", get(current_permission))
}

#[utoipa::path(get, path = "api/sectors/current/permission", tag = "Sectors", security(("bearerAuth" = [])), responses((status = 200, description = "Successful response", body = SectorPermissionResponse)))]
async fn current_permission(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<SectorPermissionResponse>, SectorRouteError> {
    let user_id = current_user.user_id.ok_or(SectorRouteError::Unauthorized)?;
    let user = user_repository::find_detail_by_id(services.db(), user_id)
        .await
        .map_err(SectorRouteError::Database)?
        .ok_or(SectorRouteError::UserNotFound)?;
    let has_permission = sector_repository::user_can_online(services.db(), user.id, &user.cid)
        .await
        .map_err(SectorRouteError::Database)?;

    Ok(Json(SectorPermissionResponse {
        has_permission,
        sector_type: "controller",
    }))
}

#[derive(Serialize, utoipa::ToSchema)]
struct SectorPermissionResponse {
    has_permission: bool,
    sector_type: &'static str,
}

#[derive(Debug)]
enum SectorRouteError {
    Database(sqlx::Error),
    Unauthorized,
    UserNotFound,
}

impl IntoResponse for SectorRouteError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            SectorRouteError::Database(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            SectorRouteError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
            SectorRouteError::UserNotFound => (StatusCode::NOT_FOUND, "user not found".into()),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct ErrorResponse {
    message: String,
}
