use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::routes::ApiError;
use crate::{
    auth::CurrentUser,
    models::user_role::UserRole,
    repository::{
        event::event as event_repository,
        event::event_airspace::{
            self as airspace_repository, EventAirspaceRecord, EventAirspaceSave,
        },
    },
    services::Services,
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_airspaces, create_airspace, update_airspace, delete_airspace))]
pub(crate) struct ApiDoc;

pub fn build_event_airspace_routes() -> Router<Services> {
    Router::new()
        .route("/{eid}/airspaces", get(list_airspaces))
        .route("/{eid}/airspaces/{aid}", get(get_airspace))
        .route("/{eid}/airspaces", post(create_airspace))
        .route(
            "/{eid}/airspaces/{aid}",
            put(update_airspace).delete(delete_airspace),
        )
}

#[utoipa::path(get, path = "api/events/{event_id}/airspaces", tag = "Events", params(("event_id" = String, Path, description = "Event ULID")), responses((status = 200, description = "Successful response", body = Vec<EventAirspaceDto>)))]
async fn list_airspaces(
    State(services): State<Services>,
    Path(eid): Path<String>,
) -> Result<Json<Vec<EventAirspaceDto>>, ApiError> {
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    ensure_event_exists(&services, event_id).await?;

    Ok(Json(
        airspace_repository::list_by_event(services.db(), event_id)
            .await
            .map_err(ApiError::Database)?
            .into_iter()
            .map(EventAirspaceDto::from)
            .collect(),
    ))
}

async fn get_airspace(
    State(services): State<Services>,
    Path((eid, aid)): Path<(String, String)>,
) -> Result<Json<EventAirspaceDto>, ApiError> {
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    let airspace_id = parse_ulid_uuid(&aid, ApiError::InvalidAirspaceId)?;
    let airspace = airspace_repository::find_by_event_and_id(services.db(), event_id, airspace_id)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::AirspaceNotFound)?;

    Ok(Json(EventAirspaceDto::from(airspace)))
}

#[utoipa::path(post, path = "api/events/{event_id}/airspaces", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID")), request_body = EventAirspaceSaveRequest, responses((status = 200, description = "Successful response", body = EventAirspaceDto)))]
async fn create_airspace(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
    Json(request): Json<EventAirspaceSaveRequest>,
) -> Result<Json<EventAirspaceDto>, ApiError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| ApiError::Forbidden)?;
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    ensure_event_exists(&services, event_id).await?;
    let airspace =
        airspace_repository::create(services.db(), event_id, EventAirspaceSave::from(request))
            .await
            .map_err(ApiError::Database)?;

    Ok(Json(EventAirspaceDto::from(airspace)))
}

#[utoipa::path(put, path = "api/events/{event_id}/airspaces/{airspace_id}", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("airspace_id" = String, Path, description = "Airspace ULID")), responses((status = 200, description = "Successful response", body = EventAirspaceDto)))]
async fn update_airspace(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((eid, aid)): Path<(String, String)>,
    Json(request): Json<EventAirspaceSaveRequest>,
) -> Result<Json<EventAirspaceDto>, ApiError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| ApiError::Forbidden)?;
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    let airspace_id = parse_ulid_uuid(&aid, ApiError::InvalidAirspaceId)?;
    let airspace = airspace_repository::update(
        services.db(),
        event_id,
        airspace_id,
        EventAirspaceSave::from(request),
    )
    .await
    .map_err(ApiError::Database)?
    .ok_or(ApiError::AirspaceNotFound)?;

    Ok(Json(EventAirspaceDto::from(airspace)))
}

#[utoipa::path(delete, path = "api/events/{event_id}/airspaces/{airspace_id}", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("airspace_id" = String, Path, description = "Airspace ULID")), responses((status = 204, description = "No content")))]
async fn delete_airspace(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((eid, aid)): Path<(String, String)>,
) -> Result<Json<EventAirspaceDto>, ApiError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| ApiError::Forbidden)?;
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    let airspace_id = parse_ulid_uuid(&aid, ApiError::InvalidAirspaceId)?;
    let airspace = airspace_repository::delete(services.db(), event_id, airspace_id)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::AirspaceNotFound)?;

    Ok(Json(EventAirspaceDto::from(airspace)))
}

async fn ensure_event_exists(services: &Services, event_id: Uuid) -> Result<(), ApiError> {
    if event_repository::exists(services.db(), event_id)
        .await
        .map_err(ApiError::Database)?
    {
        Ok(())
    } else {
        Err(ApiError::EventNotFound)
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
