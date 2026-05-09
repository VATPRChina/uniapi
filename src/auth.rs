use std::collections::HashSet;

use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use ulid::Ulid;
use uuid::Uuid;

use crate::error::ApiError;
use crate::jwt::JwtError;
use crate::models::user_role::{UserRole, role_closure_from_strings};
use crate::repository::atc::user_atc_permission;
use crate::repository::auth::user;
use crate::services::Services;

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

    pub fn require_role(&self, role: UserRole) -> Result<(), AuthError> {
        if self.has_role(role) {
            Ok(())
        } else {
            Err(AuthError::MissingRole(role))
        }
    }

    pub fn require_any_role(&self, required_roles: &[UserRole]) -> Result<(), AuthError> {
        if required_roles.iter().any(|role| self.has_role(*role)) {
            return Ok(());
        }
        Err(AuthError::MissingAnyRole(required_roles.to_vec()))
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("missing bearer token")]
    MissingBearerToken,

    #[error("invalid bearer token")]
    InvalidBearerToken,

    #[error(transparent)]
    Jwt(#[from] JwtError),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("missing role {0}")]
    MissingRole(UserRole),

    #[error("missing any role of {0:?}")]
    MissingAnyRole(Vec<UserRole>),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        ApiError::from(self).into_response()
    }
}

pub async fn authenticate(
    State(services): State<Services>,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, AuthError> {
    if let Some(token) = bearer_token(request.headers())? {
        let user = authenticate_token(&services, token).await?;
        request.extensions_mut().insert(user);
    }
    Ok(next.run(request).await)
}

fn bearer_token(headers: &HeaderMap) -> Result<Option<&str>, AuthError> {
    let Some(authorization) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    else {
        return Ok(None);
    };

    authorization
        .strip_prefix("Bearer ")
        .map(Some)
        .ok_or(AuthError::InvalidBearerToken)
}

async fn authenticate_token(services: &Services, token: &str) -> Result<CurrentUser, AuthError> {
    let token = services.jwt().validate_access_token_claims(token)?;
    let mut roles = HashSet::new();
    let Ok(user_ulid) = token.subject.parse::<Ulid>() else {
        roles.insert(UserRole::ApiClient);
        return Ok(CurrentUser {
            subject: token.subject,
            issued_at: token.issued_at,
            expires_at: token.expires_at,
            session_id: token.session_id,
            user_id: None,
            roles,
        });
    };
    let user_id = Uuid::from(user_ulid);

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
