use std::collections::HashSet;

use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use tracing::{Instrument, instrument};
use ulid::Ulid;
use uuid::Uuid;

use crate::error::ApiError;
use crate::jwt::JwtError;
use crate::model::user_role::{UserRole, role_closure_from_strings};
use crate::repository::atc::user_atc_permission::UserAtcPermissionRepositoryExt;
use crate::repository::auth::user::UserRepositoryExt;
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
        tracing::info!(subject = %self.subject, ?role, "asserting required role");
        if self.has_role(role) {
            Ok(())
        } else {
            tracing::warn!(subject = %self.subject, ?role, "required role assertion failed");
            Err(AuthError::MissingRole(role))
        }
    }

    pub fn require_any_role(&self, required_roles: &[UserRole]) -> Result<(), AuthError> {
        tracing::info!(subject = %self.subject, ?required_roles, "asserting any required role");
        if required_roles.iter().any(|role| self.has_role(*role)) {
            return Ok(());
        }
        tracing::warn!(subject = %self.subject, ?required_roles, "required role assertion failed");
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
    let span = tracing::info_span!("authenticate", http.target = %request.uri().path());
    async {
        if let Some(token) = bearer_token(request.headers())? {
            let user = authenticate_token(&services, token).await?;
            tracing::info!(subject = %user.subject, user_id = ?user.user_id, "authenticated bearer token");
            request.extensions_mut().insert(user);
        }
        Ok::<_, AuthError>(())
    }.instrument(span).await?;
    Ok(next.run(request).await)
}

#[instrument(skip(headers))]
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

#[instrument(skip(services, token))]
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

    if let Some(user) = services.db().find_user_by_id(user_id).await? {
        roles.insert(UserRole::User);
        roles.extend(role_closure_from_strings(
            user.roles.iter().map(String::as_str),
        ));

        if services
            .db()
            .has_user_atc_permission_any_by_user_id(user.id)
            .await?
        {
            roles.insert(UserRole::Controller);
        }

        if services
            .db()
            .has_user_atc_permission_mentor_by_user_id(user.id)
            .await?
        {
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
