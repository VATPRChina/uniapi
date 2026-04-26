use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::BTreeSet;
use ulid::Ulid;
use uuid::Uuid;

use crate::{
    adapter::{
        database::user::{self as user_repository, UserDetailRecord},
        moodle::MoodleError,
    },
    auth::CurrentUser,
    models::user_role::{UserRole, role_closure_from_strings},
    services::Services,
};

pub fn build_user_routes() -> Router<Services> {
    Router::new()
        .route("/", get(list_users))
        .route("/me", get(me))
        .route("/{id}", get(get_user))
        .route("/{id}/roles", put(set_roles))
        .route("/by-cid/{cid}", get(get_user_by_cid).post(assume_by_cid))
}

async fn list_users(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<UserDto>>, UserRouteError> {
    require_role(&current_user, UserRole::Volunteer)?;
    let show_full_name = current_user.has_role(UserRole::Staff);
    let users = user_repository::list_details_ordered_by_cid(services.db())
        .await
        .map_err(UserRouteError::Database)?
        .into_iter()
        .map(|user| user_dto(user, None, show_full_name, None))
        .collect();

    Ok(Json(users))
}

async fn get_user(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<UserDto>, UserRouteError> {
    require_role(&current_user, UserRole::Volunteer)?;
    let id = parse_ulid_uuid(&id)?;
    let user = user_repository::find_detail_by_id(services.db(), id)
        .await
        .map_err(UserRouteError::Database)?
        .ok_or(UserRouteError::UserNotFound)?;
    let moodle_account = moodle_account(&services, &user.cid).await?;

    Ok(Json(user_dto(
        user,
        moodle_account,
        current_user.has_role(UserRole::Staff),
        None,
    )))
}

async fn get_user_by_cid(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(cid): Path<String>,
) -> Result<Json<UserDto>, UserRouteError> {
    require_role(&current_user, UserRole::Volunteer)?;
    let user = user_repository::find_detail_by_cid(services.db(), &cid)
        .await
        .map_err(UserRouteError::Database)?
        .ok_or(UserRouteError::UserNotFoundByCid)?;

    Ok(Json(user_dto(
        user,
        None,
        current_user.has_role(UserRole::Staff),
        None,
    )))
}

async fn assume_by_cid(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(cid): Path<String>,
) -> Result<Json<UserDto>, UserRouteError> {
    require_role(&current_user, UserRole::Staff)?;
    let user = user_repository::create_assumed_user(services.db(), Uuid::from(Ulid::new()), &cid)
        .await
        .map_err(UserRouteError::Database)?;

    Ok(Json(user_dto(user, None, false, None)))
}

async fn set_roles(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(roles): Json<BTreeSet<String>>,
) -> Result<Json<UserDto>, UserRouteError> {
    require_role(&current_user, UserRole::Staff)?;
    let id = parse_ulid_uuid(&id)?;
    let user = user_repository::find_detail_by_id(services.db(), id)
        .await
        .map_err(UserRouteError::Database)?
        .ok_or(UserRouteError::UserNotFound)?;

    let current_roles = role_closure_from_strings(user.roles.iter().map(String::as_str));
    let new_roles = role_closure_from_strings(roles.iter().map(String::as_str));
    if current_roles.contains(&UserRole::Staff)
        && !new_roles.contains(&UserRole::Staff)
        && !current_user.has_role(UserRole::DivisionDirector)
    {
        return Err(UserRouteError::RemoveStaffForbidden);
    }

    let user = user_repository::set_roles(services.db(), id, roles.into_iter().collect())
        .await
        .map_err(UserRouteError::Database)?
        .ok_or(UserRouteError::UserNotFound)?;

    Ok(Json(user_dto(user, None, false, None)))
}

async fn me(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<UserDto>, UserRouteError> {
    let user_id = current_user.user_id.ok_or(UserRouteError::UserNotFound)?;
    let user = user_repository::find_detail_by_id(services.db(), user_id)
        .await
        .map_err(UserRouteError::Database)?
        .ok_or(UserRouteError::UserNotFound)?;
    let moodle_account = moodle_account(&services, &user.cid).await?;

    Ok(Json(user_dto(user, moodle_account, true, None)))
}

async fn moodle_account(
    services: &Services,
    cid: &str,
) -> Result<Option<UserMoodleInfoDto>, UserRouteError> {
    Ok(services
        .moodle()
        .get_user_by_cid(cid)
        .await
        .map_err(UserRouteError::Moodle)?
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

fn require_role(current_user: &CurrentUser, role: UserRole) -> Result<(), UserRouteError> {
    if current_user.has_role(role) {
        Ok(())
    } else {
        Err(UserRouteError::Forbidden)
    }
}

fn parse_ulid_uuid(id: &str) -> Result<Uuid, UserRouteError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| UserRouteError::InvalidUserId)
}

fn role_to_dto(role: UserRole) -> String {
    role.as_str().to_string()
}

#[derive(Serialize)]
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

#[derive(Serialize)]
struct UserMoodleInfoDto {
    id: String,
}

#[derive(Debug)]
enum UserRouteError {
    Database(sqlx::Error),
    Forbidden,
    InvalidUserId,
    Moodle(MoodleError),
    RemoveStaffForbidden,
    UserNotFound,
    UserNotFoundByCid,
}

impl IntoResponse for UserRouteError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            UserRouteError::Database(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            UserRouteError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            UserRouteError::InvalidUserId => (StatusCode::BAD_REQUEST, "invalid user id".into()),
            UserRouteError::Moodle(error) => (StatusCode::BAD_GATEWAY, error.to_string()),
            UserRouteError::RemoveStaffForbidden => (
                StatusCode::FORBIDDEN,
                "only division director can remove staff role".into(),
            ),
            UserRouteError::UserNotFound => (StatusCode::NOT_FOUND, "user not found".into()),
            UserRouteError::UserNotFoundByCid => {
                (StatusCode::NOT_FOUND, "user with cid not found".into())
            }
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}
