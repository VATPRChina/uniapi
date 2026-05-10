use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;
use ulid::Ulid;

use crate::auth::CurrentUser;
use crate::model::user_role::UserRole;
use crate::repository::auth::{session as session_repository, user as user_repository};
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_current, logout))]
pub(crate) struct ApiDoc;

pub fn build_session_routes() -> Router<Services> {
    Router::new().route("/", get(get_current).delete(logout))
}

#[utoipa::path(get, path = "api/session", tag = "Session", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = TokenDto)))]
async fn get_current(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<TokenDto>, ApiError> {
    let user_id = current_user
        .user_id
        .ok_or(ApiError::not_found("user", "unknown"))?;
    let user = user_repository::find_detail_by_id(services.db(), user_id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    let moodle_account = services
        .moodle()
        .get_user_by_cid(&user.cid)
        .await?
        .map(|user| UserMoodleInfoDto {
            id: user.id.to_string(),
        });

    Ok(Json(TokenDto {
        user: UserDto {
            id: Ulid::from(user.id).to_string(),
            cid: user.cid,
            full_name: user.full_name,
            created_at: user.created_at,
            updated_at: user.updated_at,
            roles: current_user
                .roles()
                .map(role_to_dto)
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
            direct_roles: user
                .roles
                .into_iter()
                .filter_map(|role: String| role.parse::<UserRole>().ok())
                .map(role_to_dto)
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
            moodle_account,
        },
        issued_at: DateTime::from_timestamp(current_user.issued_at, 0).ok_or(
            ApiError::InvalidTokenClaims {
                field: "issued_at".to_string(),
                reason: "out-of-range number of seconds and/or invalid nanosecond".to_string(),
            },
        )?,
        expires_at: DateTime::from_timestamp(current_user.expires_at, 0).ok_or(
            ApiError::InvalidTokenClaims {
                field: "expires_at".to_string(),
                reason: "out-of-range number of seconds and/or invalid nanosecond".to_string(),
            },
        )?,
    }))
}

#[utoipa::path(delete, path = "api/session", tag = "Session", security(("oauth2" = [])), responses((status = 204, description = "No content")))]
async fn logout(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<StatusCode, ApiError> {
    let session_id = current_user
        .session_id
        .ok_or(ApiError::MissingSessionId)?
        .parse::<Ulid>()
        .map_err(|_| ApiError::bad_request("session_id", "invalid ULID"))?;

    if !session_repository::delete(services.db(), session_id).await? {
        return Err(ApiError::not_found("refresh token", session_id.to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, utoipa::ToSchema)]
struct TokenDto {
    user: UserDto,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
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

fn role_to_dto(role: UserRole) -> String {
    role.as_str().to_string()
}
