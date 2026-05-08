use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::BTreeSet;
use ulid::Ulid;
use uuid::Uuid;

use crate::routes::ApiError;
use crate::{
    auth::CurrentUser,
    models::user_role::{UserRole, role_closure_from_strings},
    repository::auth::user::{self as user_repository, UserDetailRecord},
    services::Services,
};

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
    current_user
        .require_role(UserRole::Volunteer)
        .map_err(|_| ApiError::Forbidden)?;
    let show_full_name = current_user.has_role(UserRole::Staff);
    let users = user_repository::list_details_ordered_by_cid(services.db())
        .await
        .map_err(ApiError::Database)?
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
    current_user
        .require_role(UserRole::Staff)
        .map_err(|_| ApiError::Forbidden)?;
    let id = parse_ulid_uuid(&id)?;
    let user = user_repository::find_detail_by_id(services.db(), id)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::UserNotFound)?;

    let current_roles = role_closure_from_strings(user.roles.iter().map(String::as_str));
    let new_roles = role_closure_from_strings(roles.iter().map(String::as_str));
    if current_roles.contains(&UserRole::Staff)
        && !new_roles.contains(&UserRole::Staff)
        && !current_user.has_role(UserRole::DivisionDirector)
    {
        return Err(ApiError::RemoveStaffForbidden);
    }

    let user = user_repository::set_roles(services.db(), id, roles.into_iter().collect())
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::UserNotFound)?;

    Ok(Json(user_dto(user, None, false, None)))
}

#[utoipa::path(get, path = "api/users/me", tag = "Users", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = UserDto)))]
async fn me(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<UserDto>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::UserNotFound)?;
    let user = user_repository::find_detail_by_id(services.db(), user_id)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::UserNotFound)?;
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
        .await
        .map_err(ApiError::Moodle)?
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
    let direct_roles = user
        .roles
        .iter()
        .filter_map(|role| role.parse::<UserRole>().ok())
        .map(role_to_dto)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let roles = roles_override
        .unwrap_or_else(|| {
            role_closure_from_strings(user.roles.iter().map(String::as_str))
                .into_iter()
                .collect()
        })
        .into_iter()
        .map(role_to_dto)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    UserDto {
        id: Ulid::from(user.id).to_string(),
        cid: user.cid,
        full_name: if show_full_name {
            user.full_name
        } else {
            String::new()
        },
        created_at: user.created_at,
        updated_at: user.updated_at,
        roles,
        direct_roles,
        moodle_account,
    }
}

fn parse_ulid_uuid(id: &str) -> Result<Uuid, ApiError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| ApiError::InvalidUserId)
}

fn role_to_dto(role: UserRole) -> String {
    role.as_str().to_string()
}

#[derive(Serialize, utoipa::ToSchema)]
struct UserDto {
    id: String,
    cid: String,
    full_name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    roles: Vec<String>,
    direct_roles: Vec<String>,
    moodle_account: Option<UserMoodleInfoDto>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct UserMoodleInfoDto {
    id: String,
}
