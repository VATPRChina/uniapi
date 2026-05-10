use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::model::user_role::UserRole;
use crate::repository::event::event as event_repository;
use crate::repository::event::event_airspace::{
    self as airspace_repository, EventAirspaceRecord, EventAirspaceSave,
};
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(create_airspace))]
pub(crate) struct ApiDoc;

pub fn build_event_airspace_routes() -> Router<Services> {
    Router::new().route("/{eid}/airspaces", post(create_airspace))
}

#[utoipa::path(post, path = "api/events/{event_id}/airspaces", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID")), request_body = EventAirspaceSaveRequest, responses((status = 200, description = "Successful response", body = EventAirspaceDto)))]
async fn create_airspace(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
    Json(request): Json<EventAirspaceSaveRequest>,
) -> Result<Json<EventAirspaceDto>, ApiError> {
    current_user.require_role(UserRole::EventCoordinator)?;
    let event_id = parse_ulid_uuid(&eid, ApiError::bad_request("event_id", "invalid ULID"))?;
    ensure_event_exists(&services, event_id).await?;
    let airspace =
        airspace_repository::create(services.db(), event_id, EventAirspaceSave::from(request))
            .await?;

    Ok(Json(EventAirspaceDto::from(airspace)))
}

async fn ensure_event_exists(services: &Services, event_id: Uuid) -> Result<(), ApiError> {
    if event_repository::exists(services.db(), event_id).await? {
        Ok(())
    } else {
        Err(ApiError::not_found("event", "unknown"))
    }
}

fn parse_ulid_uuid(id: &str, error: ApiError) -> Result<Uuid, ApiError> {
    id.parse::<Ulid>().map(Uuid::from).map_err(|_| error)
}

#[derive(Deserialize, utoipa::ToSchema)]
struct EventAirspaceSaveRequest {
    name: String,
    icao_codes: Vec<String>,
    description: String,
}

impl From<EventAirspaceSaveRequest> for EventAirspaceSave {
    fn from(request: EventAirspaceSaveRequest) -> Self {
        Self {
            name: request.name,
            icao_codes: request.icao_codes,
            description: request.description,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct EventAirspaceDto {
    id: String,
    event_id: String,
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    icao_codes: Vec<String>,
    description: String,
}

impl From<EventAirspaceRecord> for EventAirspaceDto {
    fn from(airspace: EventAirspaceRecord) -> Self {
        Self {
            id: Ulid::from(airspace.id).to_string(),
            event_id: Ulid::from(airspace.event_id).to_string(),
            name: airspace.name,
            created_at: airspace.created_at,
            updated_at: airspace.updated_at,
            icao_codes: airspace.icao_codes,
            description: airspace.description,
        }
    }
}
