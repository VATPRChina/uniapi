use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use chrono::Utc;
use std::collections::BTreeSet;

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::audit_log::{AuditLog, AuditLogEntity};
use crate::model::user_role::{UserRole, role_closure_from_strings};
use crate::repository::audit_log as audit_log_repository;
use crate::repository::auth::user::{self as user_repository, UserDetailRecord};
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_users, me, set_roles))]
pub(crate) struct ApiDoc;

pub fn build_user_routes() -> Router<Services> {
    Router::new()
        .route("/", get(list_users))
        .route("/me", get(me))
        .route("/{id}/roles", put(set_roles))
}

#[utoipa::path(get, path = "api/users", tag = "Users", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<UserDto>)))]
async fn list_users(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<UserDto>>, ApiError> {
    current_user.require_role(UserRole::Volunteer)?;
    let show_full_name = current_user.has_role(UserRole::Staff);
    let users = user_repository::list_details_ordered_by_cid(services.db())
        .await?
        .into_iter()
        .map(|user| user_dto(user, None, show_full_name, None))
        .collect();

    Ok(Json(users))
}

#[utoipa::path(put, path = "api/users/{id}/roles", tag = "Users", security(("oauth2" = [])), params(("id" = String, Path, description = "User ULID")), request_body = Vec<String>, responses((status = 200, description = "Successful response", body = UserDto)))]
async fn set_roles(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(roles): Json<BTreeSet<String>>,
) -> Result<Json<UserDto>, ApiError> {
    current_user.require_role(UserRole::Staff)?;
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let id = parse_ulid_uuid("user_id", &id)?;
    let mut transaction = services.db().begin().await?;
    let user = user_repository::find_detail_by_id_for_update(&mut transaction, id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;

    let current_roles = role_closure_from_strings(user.roles.iter().map(String::as_str));
    let new_roles = role_closure_from_strings(roles.iter().map(String::as_str));
    if current_roles.contains(&UserRole::Staff)
        && !new_roles.contains(&UserRole::Staff)
        && !current_user.has_role(UserRole::DivisionDirector)
    {
        return Err(ApiError::RemoveStaffForbidden);
    }

    let before = serde_json::to_value(&user).map_err(|_| ApiError::Internal)?;
    let user = user_repository::set_roles(&mut transaction, id, roles.into_iter().collect())
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    audit_log_repository::create(
        &mut transaction,
        AuditLog {
            entity: AuditLogEntity::User(id),
            child_entity: Some(AuditLogEntity::UserRole(id)),
            before,
            after: serde_json::to_value(&user).map_err(|_| ApiError::Internal)?,
            operated_by,
            created_at: Utc::now(),
        },
    )
    .await?;
    transaction.commit().await?;

    Ok(Json(user_dto(user, None, false, None)))
}

#[utoipa::path(get, path = "api/users/me", tag = "Users", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = UserDto)))]
async fn me(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<UserDto>, ApiError> {
    let user_id = current_user
        .user_id
        .ok_or(ApiError::not_found("user", "unknown"))?;
    let user = user_repository::find_detail_by_id(services.db(), user_id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    let moodle_account = moodle_account(&services, &user.cid).await?;

    Ok(Json(user_dto(user, moodle_account, true, None)))
}

async fn moodle_account(
    services: &Services,
    cid: &str,
) -> Result<Option<UserMoodleInfoDto>, ApiError> {
    Ok(services
        .moodle()
        .get_user_by_cid(cid)
        .await?
        .map(|user| UserMoodleInfoDto {
            id: user.id.to_string(),
        }))
}

fn user_dto(
    user: UserDetailRecord,
    moodle_account: Option<UserMoodleInfoDto>,
    show_full_name: bool,
    roles_override: Option<Vec<UserRole>>,
) -> UserDto {
    UserDto::from_user_detail(user, moodle_account, show_full_name, roles_override)
}
