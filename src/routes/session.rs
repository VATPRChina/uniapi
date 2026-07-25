use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::DateTime;
use ulid::Ulid;

use crate::modules::authentication::middleware::CurrentUser;
use crate::modules::user::dto::{TokenDto, UserDto};
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
    let user = services
        .user()
        .find_by_id(user_id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    let mut user = UserDto::from_user(user, true);
    user.roles = current_user
        .roles()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    Ok(Json(TokenDto {
        user,
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

    if !services.refresh_token().delete(session_id).await? {
        return Err(ApiError::not_found("refresh token", session_id.to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
