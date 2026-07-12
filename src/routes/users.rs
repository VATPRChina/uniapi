use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use std::collections::BTreeSet;

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::audit_log::AuditLogEntity;
use crate::model::user_role::{UserRole, role_closure_from_strings};
use crate::repository::auth::user::UserRepositoryExt;
use crate::repository::auth::user::{UserDetailRecord, UserMoodleProvisionRecord};
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_users, me, set_roles, ensure_moodle_account))]
pub(crate) struct ApiDoc;

pub fn build_user_routes() -> Router<Services> {
    Router::new()
        .route("/", get(list_users))
        .route("/me", get(me))
        .route("/{id}/roles", put(set_roles))
        .route("/{id}/moodle-account", put(ensure_moodle_account))
}

#[utoipa::path(get, path = "api/users", tag = "Users", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<UserDto>)))]
async fn list_users(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<UserDto>>, ApiError> {
    current_user.require_role(UserRole::Volunteer)?;
    let show_full_name = current_user.has_role(UserRole::Staff);
    let users = services
        .db()
        .list_user_details_ordered_by_cid()
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
    let user = (&mut *transaction)
        .find_user_detail_by_id_for_update(id)
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

    let before = user;
    let user = (&mut *transaction)
        .set_user_roles(id, roles.into_iter().collect())
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    transaction.commit().await?;
    services
        .audit_log()
        .record(
            AuditLogEntity::UserRole(id, id),
            operated_by,
            Some(&before),
            Some(&user),
        )
        .await?;

    Ok(Json(user_dto(user, None, false, None)))
}

#[utoipa::path(put, path = "api/users/{id}/moodle-account", tag = "Users", security(("oauth2" = [])), params(("id" = String, Path, description = "User ULID")), responses((status = 200, description = "Successful response", body = UserDto)))]
async fn ensure_moodle_account(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<UserDto>, ApiError> {
    current_user.require_role(UserRole::TechDirector)?;
    let id = parse_ulid_uuid("user_id", &id)?;
    let user = services
        .db()
        .find_user_detail_by_id(id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    let provision = services
        .db()
        .find_user_moodle_provision_by_id(id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    let moodle_account = ensure_moodle_user(&services, &provision).await?;

    Ok(Json(user_dto(user, moodle_account, true, None)))
}

#[utoipa::path(get, path = "api/users/me", tag = "Users", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = UserDto)))]
async fn me(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<UserDto>, ApiError> {
    let user_id = current_user
        .user_id
        .ok_or(ApiError::not_found("user", "unknown"))?;
    let user = services
        .db()
        .find_user_detail_by_id(user_id)
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

async fn ensure_moodle_user(
    services: &Services,
    user: &UserMoodleProvisionRecord,
) -> Result<Option<UserMoodleInfoDto>, ApiError> {
    if let Some(moodle_user) = services.moodle().get_user_by_cid(&user.cid).await? {
        tracing::info!(
            user_id = %user.id,
            moodle_user_id = moodle_user.id,
            cid = %user.cid,
            "Moodle user found for CID, skipping user creation"
        );
        return Ok(Some(UserMoodleInfoDto {
            id: moodle_user.id.to_string(),
        }));
    }

    tracing::info!(
        user_id = %user.id,
        cid = %user.cid,
        "No Moodle user found for CID, creating new user"
    );
    let created_user = services
        .moodle()
        .create_user(&user.cid, &user.full_name, user.email.as_deref())
        .await?
        .into_iter()
        .next();

    if let Some(created_user) = created_user {
        tracing::info!(
            user_id = %user.id,
            moodle_user_id = created_user.id,
            moodle_username = %created_user.username,
            cid = %user.cid,
            "Created Moodle user"
        );
        return Ok(Some(UserMoodleInfoDto {
            id: created_user.id.to_string(),
        }));
    }

    moodle_account(services, &user.cid).await
}

fn user_dto(
    user: UserDetailRecord,
    moodle_account: Option<UserMoodleInfoDto>,
    show_full_name: bool,
    roles_override: Option<Vec<UserRole>>,
) -> UserDto {
    UserDto::from_user_detail(user, moodle_account, show_full_name, roles_override)
}
