use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use std::collections::BTreeSet;

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::user_role::UserRole;
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
        .user()
        .list()
        .await?
        .into_iter()
        .map(|user| UserDto::from_user_summary(user, show_full_name))
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
    let roles = roles
        .into_iter()
        .map(|role| {
            role.parse::<UserRole>()
                .map_err(|_| ApiError::bad_request("roles", format!("invalid role {role}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let user = services
        .user()
        .set_roles(
            id,
            roles,
            operated_by,
            current_user.has_role(UserRole::DivisionDirector),
        )
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;

    Ok(Json(UserDto::from_user_summary(user, false)))
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
        .user()
        .ensure_moodle_account(id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;

    Ok(Json(UserDto::from_user(user, true)))
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
        .user()
        .find_by_id(user_id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;

    Ok(Json(UserDto::from_user(user, true)))
}
