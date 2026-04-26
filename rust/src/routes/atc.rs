use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::BTreeMap;
use ulid::Ulid;
use uuid::Uuid;

use crate::{
    adapter::database::atc::{self as atc_repository, AtcControllerPermissionRecord},
    models::user_role::{UserRole, role_closure_from_strings},
    services::Services,
};

pub fn build_atc_routes() -> Router<Services> {
    Router::new().route("/", get(list_controllers))
}

async fn list_controllers(
    State(services): State<Services>,
) -> Result<Json<Vec<AtcStatusDto>>, AtcRouteError> {
    let rows = atc_repository::list_controllers(services.db())
        .await
        .map_err(AtcRouteError::Database)?;

    let mut statuses = BTreeMap::<Uuid, AtcStatusBuilder>::new();
    for row in rows {
        statuses
            .entry(row.user_id)
            .or_insert_with(|| AtcStatusBuilder::from(&row))
            .permissions
            .push(AtcPermissionDto::from(&row));
    }

    Ok(Json(
        statuses
            .into_values()
            .map(AtcStatusDto::from)
            .collect::<Vec<_>>(),
    ))
}

struct AtcStatusBuilder {
    user_id: Uuid,
    user: UserDto,
    is_visiting: bool,
    is_absent: bool,
    rating: String,
    permissions: Vec<AtcPermissionDto>,
}

impl AtcStatusBuilder {
    fn from(row: &AtcControllerPermissionRecord) -> Self {
        Self {
            user_id: row.user_id,
            user: UserDto {
                id: Ulid::from(row.user_id).to_string(),
                cid: row.user_cid.clone(),
                full_name: row.user_full_name.clone(),
                created_at: row.user_created_at,
                updated_at: row.user_updated_at,
                roles: roles_to_dto(&row.user_roles),
                direct_roles: direct_roles_to_dto(&row.user_roles),
                moodle_account: None,
            },
            is_visiting: row.is_visiting.unwrap_or(false),
            is_absent: row.is_absent.unwrap_or(false),
            rating: row.rating.clone().unwrap_or_else(|| "OBS".to_owned()),
            permissions: Vec::new(),
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

impl From<AtcStatusBuilder> for AtcStatusDto {
    fn from(status: AtcStatusBuilder) -> Self {
        Self {
            user_id: Ulid::from(status.user_id).to_string(),
            user: status.user,
            is_visiting: status.is_visiting,
            is_absent: status.is_absent,
            rating: status.rating,
            permissions: status.permissions,
        }
    }
}

#[derive(Serialize)]
struct AtcPermissionDto {
    position_kind_id: String,
    state: String,
    solo_expires_at: Option<DateTime<Utc>>,
}

impl From<&AtcControllerPermissionRecord> for AtcPermissionDto {
    fn from(permission: &AtcControllerPermissionRecord) -> Self {
        Self {
            position_kind_id: permission.position_kind_id.clone(),
            state: permission.state.clone(),
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
enum AtcRouteError {
    Database(sqlx::Error),
}

impl IntoResponse for AtcRouteError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AtcRouteError::Database(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}
