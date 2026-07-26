use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::modules::controller::dto::SectorPermissionResponse;
use crate::modules::user::middleware::CurrentUser;
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(current_permission))]
pub(crate) struct ApiDoc;

pub fn build_sector_routes() -> Router<Services> {
    Router::new().route("/current/permission", get(current_permission))
}

#[utoipa::path(get, path = "api/sectors/current/permission", tag = "Sectors", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = SectorPermissionResponse)))]
async fn current_permission(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<SectorPermissionResponse>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    Ok(Json(SectorPermissionResponse::new(
        services.controller().has_sector_permission(user_id).await?,
    )))
}
