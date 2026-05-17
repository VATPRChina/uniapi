use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::user_role::UserRole;
use crate::repository::atc::user_atc_permission::{self as atc_permission_repository};
use crate::repository::atc::user_atc_status::{self as atc_status_repository, AtcStatusSave};
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_my_status, set_status))]
pub(crate) struct ApiDoc;

pub fn build_user_atc_permission_routes() -> Router<Services> {
    Router::new()
        .route("/me/atc/status", get(get_my_status))
        .route("/{id}/atc/status", axum::routing::put(set_status))
}

#[utoipa::path(get, path = "api/users/me/atc/status", tag = "ATC", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = AtcStatusDto)))]
async fn get_my_status(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<AtcStatusDto>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    get_status_for_user(&services, user_id).await.map(Json)
}

#[utoipa::path(put, path = "api/users/{id}/atc/status", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "User ULID")), request_body = AtcStatusRequest, responses((status = 200, description = "Successful response", body = AtcStatusDto)))]
async fn set_status(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<AtcStatusRequest>,
) -> Result<Json<AtcStatusDto>, ApiError> {
    require_admin_role(&current_user)?;
    let user_id = parse_ulid_uuid("user_id", &id)?;
    let status = AtcStatusSave::try_from(request)?;

    if atc_status_repository::find_by_user_id(services.db(), user_id)
        .await?
        .is_none()
    {
        return Err(ApiError::not_found("user", "unknown"));
    }

    let mut transaction = services.db().begin().await?;
    atc_status_repository::upsert(&mut transaction, user_id, &status).await?;
    transaction.commit().await?;

    get_status_for_user(&services, user_id).await.map(Json)
}

async fn get_status_for_user(services: &Services, user_id: Uuid) -> Result<AtcStatusDto, ApiError> {
    let status = atc_status_repository::find_by_user_id(services.db(), user_id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    let permissions = atc_permission_repository::list_by_user_id(services.db(), user_id).await?;

    Ok(AtcStatusDto::from_records(status, permissions))
}

fn require_admin_role(current_user: &CurrentUser) -> Result<(), ApiError> {
    current_user
        .require_any_role(&[
            UserRole::ControllerTrainingMentor,
            UserRole::ControllerTrainingDirectorAssistant,
        ])
        .map_err(Into::into)
}
