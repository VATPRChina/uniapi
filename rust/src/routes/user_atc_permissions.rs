use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::{
    adapter::database::user_atc_permission::{
        self as atc_permission_repository, AtcPermissionRecord, AtcPermissionSave,
        AtcStatusRecord, AtcStatusSave,
    },
    auth::CurrentUser,
    models::{
        user_controller_state::UserControllerState,
        user_role::{UserRole, role_closure_from_strings},
    },
    services::Services,
};

const ALLOWED_RATINGS: &[&str] = &["OBS", "S1", "S2", "S3", "C1", "C3", "I1", "I3"];

pub fn build_user_atc_permission_routes() -> Router<Services> {
    Router::new()
        .route("/me/atc/status", get(get_my_status))
        .route(
            "/{id}/atc/status",
            get(get_status).put(set_status).delete(delete_status),
        )
}

async fn get_my_status(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<AtcStatusDto>, UserAtcPermissionRouteError> {
    let user_id = current_user
        .user_id
        .ok_or(UserAtcPermissionRouteError::Unauthorized)?;
    get_status_for_user(&services, user_id).await.map(Json)
}

async fn get_status(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<AtcStatusDto>, UserAtcPermissionRouteError> {
    require_admin_role(&current_user)?;
    let user_id = parse_ulid_uuid(&id)?;
    get_status_for_user(&services, user_id).await.map(Json)
}

async fn set_status(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<AtcStatusRequest>,
) -> Result<Json<AtcStatusDto>, UserAtcPermissionRouteError> {
    require_admin_role(&current_user)?;
    let user_id = parse_ulid_uuid(&id)?;
    let status = AtcStatusSave::try_from(request)?;

    if atc_permission_repository::find_status_by_user_id(services.db(), user_id)
        .await
        .map_err(UserAtcPermissionRouteError::Database)?
        .is_none()
    {
        return Err(UserAtcPermissionRouteError::UserNotFound);
    }

    let mut transaction = services
        .db()
        .begin()
        .await
        .map_err(UserAtcPermissionRouteError::Database)?;
    atc_permission_repository::upsert_status(&mut transaction, user_id, &status)
        .await
        .map_err(UserAtcPermissionRouteError::Database)?;
    transaction
        .commit()
        .await
        .map_err(UserAtcPermissionRouteError::Database)?;

    get_status_for_user(&services, user_id).await.map(Json)
}

async fn delete_status(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<StatusCode, UserAtcPermissionRouteError> {
    require_admin_role(&current_user)?;
    let user_id = parse_ulid_uuid(&id)?;

    if atc_permission_repository::find_status_by_user_id(services.db(), user_id)
        .await
        .map_err(UserAtcPermissionRouteError::Database)?
        .is_none()
    {
        return Err(UserAtcPermissionRouteError::UserNotFound);
    }

    let mut transaction = services
        .db()
        .begin()
        .await
        .map_err(UserAtcPermissionRouteError::Database)?;
    let deleted =
        atc_permission_repository::delete_status_and_permissions(&mut transaction, user_id)
            .await
            .map_err(UserAtcPermissionRouteError::Database)?;
    transaction
        .commit()
        .await
        .map_err(UserAtcPermissionRouteError::Database)?;
    if !deleted {
        return Err(UserAtcPermissionRouteError::AtcStatusNotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn get_status_for_user(
    services: &Services,
    user_id: Uuid,
) -> Result<AtcStatusDto, UserAtcPermissionRouteError> {
    let status = atc_permission_repository::find_status_by_user_id(services.db(), user_id)
        .await
        .map_err(UserAtcPermissionRouteError::Database)?
        .ok_or(UserAtcPermissionRouteError::UserNotFound)?;
    let permissions = atc_permission_repository::list_permissions_by_user_id(services.db(), user_id)
        .await
        .map_err(UserAtcPermissionRouteError::Database)?;

    Ok(AtcStatusDto::from_records(status, permissions))
}

fn require_admin_role(current_user: &CurrentUser) -> Result<(), UserAtcPermissionRouteError> {
    if current_user.has_role(UserRole::ControllerTrainingMentor)
        || current_user.has_role(UserRole::ControllerTrainingDirectorAssistant)
    {
        Ok(())
    } else {
        Err(UserAtcPermissionRouteError::Forbidden)
    }
}

fn parse_ulid_uuid(id: &str) -> Result<Uuid, UserAtcPermissionRouteError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| UserAtcPermissionRouteError::InvalidUserId)
}

#[derive(Deserialize)]
struct AtcStatusRequest {
    is_visiting: bool,
    is_absent: bool,
    rating: String,
    permissions: Vec<AtcPermissionRequest>,
}

impl TryFrom<AtcStatusRequest> for AtcStatusSave {
    type Error = UserAtcPermissionRouteError;

    fn try_from(request: AtcStatusRequest) -> Result<Self, Self::Error> {
        if !ALLOWED_RATINGS.contains(&request.rating.as_str()) {
            return Err(UserAtcPermissionRouteError::InvalidAtcRating);
        }

        if request.permissions.iter().any(|permission| {
            permission.state == UserControllerState::Solo
                && permission.solo_expires_at.is_none()
        }) {
            return Err(UserAtcPermissionRouteError::SoloExpirationNotProvided);
        }

        Ok(Self {
            is_visiting: request.is_visiting,
            is_absent: request.is_absent,
            rating: request.rating,
            permissions: request
                .permissions
                .into_iter()
                .map(AtcPermissionSave::from)
                .collect(),
        })
    }
}

#[derive(Deserialize)]
struct AtcPermissionRequest {
    position_kind_id: String,
    state: UserControllerState,
    solo_expires_at: Option<DateTime<Utc>>,
}

impl From<AtcPermissionRequest> for AtcPermissionSave {
    fn from(permission: AtcPermissionRequest) -> Self {
        Self {
            position_kind_id: permission.position_kind_id,
            state: permission.state.as_db_str().to_owned(),
            solo_expires_at: permission.solo_expires_at,
        }
    }
}

#[derive(Serialize)]
struct AtcStatusDto {
    user_id: String,
    user: UserDto,
    is_visiting: bool,
    is_absent: bool,
    rating: String,
    permissions: Vec<AtcPermissionDto>,
}

impl AtcStatusDto {
    fn from_records(status: AtcStatusRecord, permissions: Vec<AtcPermissionRecord>) -> Self {
        Self {
            user_id: Ulid::from(status.user_id).to_string(),
            user: UserDto {
                id: Ulid::from(status.user_id).to_string(),
                cid: status.user_cid,
                full_name: status.user_full_name,
                created_at: status.user_created_at,
                updated_at: status.user_updated_at,
                roles: roles_to_dto(&status.user_roles),
                direct_roles: direct_roles_to_dto(&status.user_roles),
                moodle_account: None,
            },
            is_visiting: status.is_visiting.unwrap_or(false),
            is_absent: status.is_absent.unwrap_or(false),
            rating: status.rating.unwrap_or_else(|| "OBS".to_owned()),
            permissions: permissions.into_iter().map(AtcPermissionDto::from).collect(),
        }
    }
}

#[derive(Serialize)]
struct AtcPermissionDto {
    position_kind_id: String,
    state: UserControllerState,
    solo_expires_at: Option<DateTime<Utc>>,
}

impl From<AtcPermissionRecord> for AtcPermissionDto {
    fn from(permission: AtcPermissionRecord) -> Self {
        Self {
            position_kind_id: permission.position_kind_id,
            state: permission
                .state
                .parse()
                .unwrap_or(UserControllerState::Student),
            solo_expires_at: permission.solo_expires_at,
        }
    }
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
    moodle_account: Option<serde_json::Value>,
}

fn direct_roles_to_dto(roles: &[String]) -> Vec<String> {
    roles
        .iter()
        .filter_map(|role| role.parse::<UserRole>().ok())
        .map(|role| role.as_str().to_owned())
        .collect()
}

fn roles_to_dto(roles: &[String]) -> Vec<String> {
    let mut roles = role_closure_from_strings(roles.iter().map(String::as_str))
        .into_iter()
        .map(|role| role.as_str().to_owned())
        .collect::<Vec<_>>();
    roles.sort();
    roles
}

#[derive(Debug)]
enum UserAtcPermissionRouteError {
    AtcStatusNotFound,
    Database(sqlx::Error),
    Forbidden,
    InvalidAtcRating,
    InvalidUserId,
    SoloExpirationNotProvided,
    Unauthorized,
    UserNotFound,
}

impl IntoResponse for UserAtcPermissionRouteError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            UserAtcPermissionRouteError::AtcStatusNotFound => {
                (StatusCode::NOT_FOUND, "ATC status not found".into())
            }
            UserAtcPermissionRouteError::Database(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            UserAtcPermissionRouteError::Forbidden => {
                (StatusCode::FORBIDDEN, "forbidden".into())
            }
            UserAtcPermissionRouteError::InvalidAtcRating => {
                (StatusCode::BAD_REQUEST, "invalid ATC rating".into())
            }
            UserAtcPermissionRouteError::InvalidUserId => {
                (StatusCode::BAD_REQUEST, "invalid user id".into())
            }
            UserAtcPermissionRouteError::SoloExpirationNotProvided => (
                StatusCode::BAD_REQUEST,
                "solo expiration not provided".into(),
            ),
            UserAtcPermissionRouteError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "unauthorized".into())
            }
            UserAtcPermissionRouteError::UserNotFound => {
                (StatusCode::NOT_FOUND, "user not found".into())
            }
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}
