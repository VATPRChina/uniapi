use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;
use ulid::Ulid;

use crate::{
    adapter::{
        database::{auth as auth_repository, user as user_repository},
        moodle::MoodleError,
    },
    auth::CurrentUser,
    models::user_role::UserRole,
    services::Services,
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_current, logout))]
pub(crate) struct ApiDoc;

pub fn build_session_routes() -> Router<Services> {
    Router::new().route("/", get(get_current).delete(logout))
}

#[utoipa::path(get, path = "api/session", tag = "Session", security(("bearerAuth" = [])), responses((status = 200, description = "Successful response", body = TokenDto)))]
async fn get_current(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<TokenDto>, SessionError> {
    let user_id = current_user.user_id.ok_or(SessionError::UserNotFound)?;
    let user = user_repository::find_detail_by_id(services.db(), user_id)
        .await
        .map_err(SessionError::Database)?
        .ok_or(SessionError::UserNotFound)?;
    let moodle_account = services
        .moodle()
        .get_user_by_cid(&user.cid)
        .await
        .map_err(SessionError::Moodle)?
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
                .filter_map(|role| role.parse::<UserRole>().ok())
                .map(role_to_dto)
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
            moodle_account,
        },
        issued_at: DateTime::from_timestamp(current_user.issued_at, 0)
            .ok_or(SessionError::InvalidTokenClaims)?,
        expires_at: DateTime::from_timestamp(current_user.expires_at, 0)
            .ok_or(SessionError::InvalidTokenClaims)?,
    }))
}

#[utoipa::path(delete, path = "api/session", tag = "Session", security(("bearerAuth" = [])), responses((status = 204, description = "No content")))]
async fn logout(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<StatusCode, SessionError> {
    let session_id = current_user
        .session_id
        .ok_or(SessionError::MissingSessionId)?
        .parse::<Ulid>()
        .map_err(|_| SessionError::InvalidSessionId)?;

    if !auth_repository::delete_refresh_session(services.db(), session_id)
        .await
        .map_err(SessionError::Database)?
    {
        return Err(SessionError::RefreshTokenNotFound(session_id.to_string()));
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

#[derive(Debug)]
enum SessionError {
    Database(sqlx::Error),
    InvalidSessionId,
    InvalidTokenClaims,
    MissingSessionId,
    Moodle(MoodleError),
    RefreshTokenNotFound(String),
    UserNotFound,
}

impl IntoResponse for SessionError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            SessionError::Database(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            SessionError::Moodle(error) => (StatusCode::BAD_GATEWAY, error.to_string()),
            SessionError::InvalidSessionId => (StatusCode::UNAUTHORIZED, "invalid sid".into()),
            SessionError::InvalidTokenClaims => {
                (StatusCode::UNAUTHORIZED, "invalid token claims".into())
            }
            SessionError::MissingSessionId => (StatusCode::UNAUTHORIZED, "no sid in token".into()),
            SessionError::RefreshTokenNotFound(token) => (
                StatusCode::UNAUTHORIZED,
                format!("refresh token {token} not found"),
            ),
            SessionError::UserNotFound => (StatusCode::UNAUTHORIZED, "user not found".into()),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct ErrorResponse {
    message: String,
}
