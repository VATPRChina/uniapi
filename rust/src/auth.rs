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
    adapter::database::{user, user_atc_permission},
    jwt::JwtError,
    models::user_role::UserRole,
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
        roles.extend(role_closure(user.roles));

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

fn role_closure(roles: Vec<String>) -> HashSet<UserRole> {
    let mut all_roles = HashSet::new();
    let mut stack = roles
        .into_iter()
        .filter_map(|role| role.parse::<UserRole>().ok())
        .collect::<Vec<_>>();

    while let Some(role) = stack.pop() {
        if !all_roles.insert(role) {
            continue;
        }

        stack.extend(implied_roles(role));
    }

    all_roles
}

fn implied_roles(role: UserRole) -> &'static [UserRole] {
    match role {
        UserRole::Staff => &[UserRole::Volunteer],
        UserRole::DivisionDirector => &[
            UserRole::Staff,
            UserRole::ControllerTrainingDirector,
            UserRole::OperationDirector,
            UserRole::EventDirector,
            UserRole::TechDirector,
        ],
        UserRole::ControllerTrainingDirector => &[
            UserRole::Staff,
            UserRole::ControllerTrainingDirectorAssistant,
            UserRole::ControllerTrainingInstructor,
            UserRole::ControllerTrainingMentor,
            UserRole::ControllerTrainingSopEditor,
        ],
        UserRole::ControllerTrainingDirectorAssistant => &[UserRole::Volunteer],
        UserRole::ControllerTrainingInstructor => {
            &[UserRole::Volunteer, UserRole::ControllerTrainingMentor]
        }
        UserRole::ControllerTrainingMentor => &[UserRole::Volunteer],
        UserRole::ControllerTrainingSopEditor => &[UserRole::Volunteer],
        UserRole::CommunityDirector => &[UserRole::Staff],
        UserRole::OperationDirector => &[
            UserRole::Staff,
            UserRole::OperationDirectorAssistant,
            UserRole::OperationSectorEditor,
            UserRole::OperationLoaEditor,
        ],
        UserRole::OperationDirectorAssistant => &[UserRole::Volunteer],
        UserRole::OperationSectorEditor => &[UserRole::Volunteer],
        UserRole::OperationLoaEditor => &[UserRole::Volunteer],
        UserRole::EventDirector => &[
            UserRole::Staff,
            UserRole::EventCoordinator,
            UserRole::LeadEventCoordinator,
            UserRole::EventGraphicsDesigner,
        ],
        UserRole::LeadEventCoordinator => &[UserRole::EventCoordinator],
        UserRole::EventCoordinator => &[UserRole::Volunteer],
        UserRole::EventGraphicsDesigner => &[UserRole::Volunteer],
        UserRole::TechDirector => &[
            UserRole::Staff,
            UserRole::TechDirectorAssistant,
            UserRole::TechAfvFacilityEngineer,
        ],
        UserRole::TechDirectorAssistant => &[UserRole::Volunteer],
        UserRole::TechAfvFacilityEngineer => &[UserRole::Volunteer],
        _ => &[],
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
