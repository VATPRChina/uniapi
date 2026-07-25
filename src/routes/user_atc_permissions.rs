use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::dto::parse_ulid_uuid;
use crate::model::user_role::UserRole;
use crate::modules::controller::dto::{AtcStatusDto, AtcStatusRequest};
use crate::modules::controller::models::ControllerSave;
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_my_status, get_status, set_status))]
pub(crate) struct ApiDoc;

pub fn build_user_atc_permission_routes() -> Router<Services> {
    Router::new()
        .route("/me/atc/status", get(get_my_status))
        .route("/{id}/atc/status", get(get_status).put(set_status))
}

#[utoipa::path(get, path = "api/users/me/atc/status", tag = "ATC", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = AtcStatusDto)))]
async fn get_my_status(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<AtcStatusDto>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    get_status_for_user(&services, user_id).await.map(Json)
}

#[utoipa::path(get, path = "api/users/{id}/atc/status", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "User ULID")), responses((status = 200, description = "Successful response", body = AtcStatusDto)))]
async fn get_status(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<AtcStatusDto>, ApiError> {
    let user_id = parse_ulid_uuid("user_id", &id)?;
    require_read_access(&current_user, user_id)?;
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
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let user_id = parse_ulid_uuid("user_id", &id)?;
    let status = ControllerSave::try_from(request)?;
    Ok(Json(
        services
            .controller()
            .update(user_id, status, operated_by)
            .await?
            .into(),
    ))
}

async fn get_status_for_user(services: &Services, user_id: Uuid) -> Result<AtcStatusDto, ApiError> {
    Ok(services.controller().find(user_id).await?.into())
}

fn require_admin_role(current_user: &CurrentUser) -> Result<(), ApiError> {
    current_user
        .require_any_role(&[
            UserRole::ControllerTrainingMentor,
            UserRole::ControllerTrainingDirectorAssistant,
        ])
        .map_err(Into::into)
}

fn require_read_access(current_user: &CurrentUser, user_id: Uuid) -> Result<(), ApiError> {
    if current_user.user_id == Some(user_id) {
        return Ok(());
    }

    require_admin_role(current_user)
}
