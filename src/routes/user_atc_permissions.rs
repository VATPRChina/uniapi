use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::audit_log::AuditLogEntity;
use crate::model::user_role::UserRole;
use crate::repository::atc::user_atc_permission::AtcPermissionRecord;
use crate::repository::atc::user_atc_permission::UserAtcPermissionRepositoryExt;
use crate::repository::atc::user_atc_status::UserAtcStatusRepositoryExt;
use crate::repository::atc::user_atc_status::UserAtcStatusTransactionExt;
use crate::repository::atc::user_atc_status::{AtcStatusRecord, AtcStatusSave};
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
    let status = AtcStatusSave::try_from(request)?;

    let mut transaction = services.db().begin().await?;
    let before = atc_status_audit_snapshot(&mut transaction, user_id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    (&mut transaction)
        .upsert_user_atc_status(user_id, &status)
        .await?;
    let after = atc_status_audit_snapshot(&mut transaction, user_id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    services
        .audit_log()
        .record(
            AuditLogEntity::UserAtcPermission(user_id, user_id),
            operated_by,
            Some(&before),
            Some(&after),
        )
        .await?;
    transaction.commit().await?;

    get_status_for_user(&services, user_id).await.map(Json)
}

#[derive(Serialize)]
struct AtcStatusAuditSnapshot {
    status: AtcStatusRecord,
    permissions: Vec<AtcPermissionRecord>,
}

async fn atc_status_audit_snapshot(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<Option<AtcStatusAuditSnapshot>, ApiError> {
    let Some(status) = (&mut **transaction)
        .find_user_atc_status_by_user_id_for_update(user_id)
        .await?
    else {
        return Ok(None);
    };
    let permissions = (&mut **transaction)
        .list_user_atc_permission_by_user_id_in_transaction(user_id)
        .await?;

    Ok(Some(AtcStatusAuditSnapshot {
        status,
        permissions,
    }))
}

async fn get_status_for_user(services: &Services, user_id: Uuid) -> Result<AtcStatusDto, ApiError> {
    let status = services
        .db()
        .find_user_atc_status_by_user_id(user_id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    let permissions = services
        .db()
        .list_user_atc_permission_by_user_id(user_id)
        .await?;

    AtcStatusDto::from_records(status, permissions)
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
