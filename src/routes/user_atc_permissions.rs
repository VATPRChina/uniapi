use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::model::user_controller_state::UserControllerState;
use crate::model::user_role::{UserRole, role_closure_from_strings};
use crate::repository::atc::user_atc_permission::{
    self as atc_permission_repository, AtcPermissionRecord, AtcPermissionSave,
};
use crate::repository::atc::user_atc_status::{
    self as atc_status_repository, AtcStatusRecord, AtcStatusSave,
};
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_my_status, set_status))]
pub(crate) struct ApiDoc;

const ALLOWED_RATINGS: &[&str] = &["OBS", "S1", "S2", "S3", "C1", "C3", "I1", "I3"];

pub fn build_user_atc_permission_routes() -> Router<Services> {
    Router::new()
        .route("/me/atc/status", get(get_my_status))
        .route("/{id}/atc/status", axum::routing::put(set_status))
}

#[utoipa::path(get, path = "api/users/me/atc/status", tag = "ATC", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = AtcStatusDto)))]
async fn get_my_status(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<AtcStatusDto>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    get_status_for_user(&services, user_id).await.map(Json)
}

#[utoipa::path(put, path = "api/users/{id}/atc/status", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "User ULID")), request_body = AtcStatusRequest, responses((status = 200, description = "Successful response", body = AtcStatusDto)))]
async fn set_status(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<AtcStatusRequest>,
) -> Result<Json<AtcStatusDto>, ApiError> {
    require_admin_role(&current_user)?;
    let user_id = parse_ulid_uuid(&id)?;
    let status = AtcStatusSave::try_from(request)?;

    if atc_status_repository::find_by_user_id(services.db(), user_id)
        .await?
        .is_none()
    {
        return Err(ApiError::not_found("user", "unknown"));
    }

    let mut transaction = services.db().begin().await?;
    atc_status_repository::upsert(&mut transaction, user_id, &status).await?;
    transaction.commit().await?;

    get_status_for_user(&services, user_id).await.map(Json)
}

async fn get_status_for_user(services: &Services, user_id: Uuid) -> Result<AtcStatusDto, ApiError> {
    let status = atc_status_repository::find_by_user_id(services.db(), user_id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    let permissions = atc_permission_repository::list_by_user_id(services.db(), user_id).await?;

    Ok(AtcStatusDto::from_records(status, permissions))
}

fn require_admin_role(current_user: &CurrentUser) -> Result<(), ApiError> {
    current_user
        .require_any_role(&[
            UserRole::ControllerTrainingMentor,
            UserRole::ControllerTrainingDirectorAssistant,
        ])
        .map_err(Into::into)
}

fn parse_ulid_uuid(id: &str) -> Result<Uuid, ApiError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| ApiError::bad_request("user_id", "invalid ULID"))
}

#[derive(Deserialize, utoipa::ToSchema)]
struct AtcStatusRequest {
    is_visiting: bool,
    is_absent: bool,
    rating: String,
    permissions: Vec<AtcPermissionRequest>,
}

impl TryFrom<AtcStatusRequest> for AtcStatusSave {
    type Error = ApiError;

    fn try_from(request: AtcStatusRequest) -> Result<Self, Self::Error> {
        if !ALLOWED_RATINGS.contains(&request.rating.as_str()) {
            return Err(ApiError::bad_request("rating", "invalid ATC rating"));
        }

        if request.permissions.iter().any(|permission| {
            permission.state == UserControllerState::Solo && permission.solo_expires_at.is_none()
        }) {
            return Err(ApiError::SoloExpirationNotProvided);
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

#[derive(Deserialize, utoipa::ToSchema)]
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

#[derive(Serialize, utoipa::ToSchema)]
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
            permissions: permissions
                .into_iter()
                .map(AtcPermissionDto::from)
                .collect(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
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

#[derive(Serialize, utoipa::ToSchema)]
struct UserDto {
    id: String,
    cid: String,
    full_name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    roles: Vec<UserRole>,
    direct_roles: Vec<UserRole>,
    moodle_account: Option<serde_json::Value>,
}

fn direct_roles_to_dto(roles: &[String]) -> Vec<UserRole> {
    roles
        .iter()
        .filter_map(|role| role.parse::<UserRole>().ok())
        .collect()
}

fn roles_to_dto(roles: &[String]) -> Vec<UserRole> {
    let mut roles = role_closure_from_strings(roles.iter().map(String::as_str))
        .into_iter()
        .collect::<Vec<_>>();
    roles.sort();
    roles
}
