use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::repository::auth::user as user_repository;
use crate::repository::sector as sector_repository;
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
    let user = user_repository::find_detail_by_id(services.db(), user_id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    let has_permission =
        sector_repository::user_can_online(services.db(), user.id, &user.cid).await?;

    Ok(Json(SectorPermissionResponse {
        has_permission,
        sector_type: "controller",
    }))
}
