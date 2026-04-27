use std::collections::HashSet;

use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::{HeaderMap, StatusCode, header, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use ulid::Ulid;
use uuid::Uuid;

use crate::{
    jwt::JwtError,
    models::user_role::{UserRole, role_closure_from_strings},
    repository::{user, user_atc_permission},
    services::Services,
};

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub subject: String,
    pub issued_at: i64,
    pub expires_at: i64,
    pub session_id: Option<String>,
    pub user_id: Option<Uuid>,
    roles: HashSet<UserRole>,
}

impl CurrentUser {
    pub fn has_role(&self, role: UserRole) -> bool {
        self.roles.contains(&role)
    }

    pub fn roles(&self) -> impl Iterator<Item = UserRole> + '_ {
        self.roles.iter().copied()
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("missing bearer token")]
    MissingBearerToken,

    #[error("invalid bearer token")]
    InvalidBearerToken,

    #[error(transparent)]
    Jwt(#[from] JwtError),

    #[error("subject is not a ULID")]
    InvalidSubject,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self {
            AuthError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::UNAUTHORIZED,
        };

        (
            status,
            Json(ErrorResponse {
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}

pub async fn authenticate(
    State(services): State<Services>,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = bearer_token(request.headers())?;
    let user = authenticate_token(&services, token).await?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

fn bearer_token(headers: &HeaderMap) -> Result<&str, AuthError> {
    let authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AuthError::MissingBearerToken)?;

    authorization
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidBearerToken)
}

async fn authenticate_token(services: &Services, token: &str) -> Result<CurrentUser, AuthError> {
    let token = services.jwt().validate_access_token_claims(token)?;
    let user_ulid = token
        .subject
        .parse::<Ulid>()
        .map_err(|_| AuthError::InvalidSubject)?;
    let user_id = Uuid::from(user_ulid);

    let mut roles = HashSet::new();

    if let Some(user) = user::find_by_id(services.db(), user_id).await? {
        roles.insert(UserRole::User);
        roles.extend(role_closure_from_strings(
            user.roles.iter().map(String::as_str),
        ));

        if user_atc_permission::has_any_by_user_id(services.db(), user.id).await? {
            roles.insert(UserRole::Controller);
        }

        if user_atc_permission::has_mentor_by_user_id(services.db(), user.id).await? {
            roles.insert(UserRole::ControllerTrainingMentor);
            roles.insert(UserRole::Volunteer);
        }

        Ok(CurrentUser {
            subject: token.subject,
            issued_at: token.issued_at,
            expires_at: token.expires_at,
            session_id: token.session_id,
            user_id: Some(user.id),
            roles,
        })
    } else {
        roles.insert(UserRole::ApiClient);

        Ok(CurrentUser {
            subject: token.subject,
            issued_at: token.issued_at,
            expires_at: token.expires_at,
            session_id: token.session_id,
            user_id: None,
            roles,
        })
    }
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or(AuthError::MissingBearerToken)
    }
}
