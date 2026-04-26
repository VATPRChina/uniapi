use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::{
    adapter::database::{
        event as event_repository,
        event_airspace::{self as airspace_repository, EventAirspaceRecord, EventAirspaceSave},
    },
    auth::CurrentUser,
    models::user_role::UserRole,
    services::Services,
};

pub fn build_public_event_airspace_routes() -> Router<Services> {
    Router::new()
        .route("/{eid}/airspaces", get(list_airspaces))
        .route("/{eid}/airspaces/{aid}", get(get_airspace))
}

pub fn build_protected_event_airspace_routes() -> Router<Services> {
    Router::new()
        .route("/{eid}/airspaces", post(create_airspace))
        .route(
            "/{eid}/airspaces/{aid}",
            put(update_airspace).delete(delete_airspace),
        )
}

async fn list_airspaces(
    State(services): State<Services>,
    Path(eid): Path<String>,
) -> Result<Json<Vec<EventAirspaceDto>>, EventAirspaceError> {
    let event_id = parse_ulid_uuid(&eid, EventAirspaceError::InvalidEventId)?;
    ensure_event_exists(&services, event_id).await?;

    Ok(Json(
        airspace_repository::list_by_event(services.db(), event_id)
            .await
            .map_err(EventAirspaceError::Database)?
            .into_iter()
            .map(EventAirspaceDto::from)
            .collect(),
    ))
}

async fn get_airspace(
    State(services): State<Services>,
    Path((eid, aid)): Path<(String, String)>,
) -> Result<Json<EventAirspaceDto>, EventAirspaceError> {
    let event_id = parse_ulid_uuid(&eid, EventAirspaceError::InvalidEventId)?;
    let airspace_id = parse_ulid_uuid(&aid, EventAirspaceError::InvalidAirspaceId)?;
    let airspace = airspace_repository::find_by_event_and_id(services.db(), event_id, airspace_id)
        .await
        .map_err(EventAirspaceError::Database)?
        .ok_or(EventAirspaceError::AirspaceNotFound)?;

    Ok(Json(EventAirspaceDto::from(airspace)))
}

async fn create_airspace(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
    Json(request): Json<EventAirspaceSaveRequest>,
) -> Result<Json<EventAirspaceDto>, EventAirspaceError> {
    require_event_coordinator(&current_user)?;
    let event_id = parse_ulid_uuid(&eid, EventAirspaceError::InvalidEventId)?;
    ensure_event_exists(&services, event_id).await?;
    let airspace =
        airspace_repository::create(services.db(), event_id, EventAirspaceSave::from(request))
            .await
            .map_err(EventAirspaceError::Database)?;

    Ok(Json(EventAirspaceDto::from(airspace)))
}

async fn update_airspace(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((eid, aid)): Path<(String, String)>,
    Json(request): Json<EventAirspaceSaveRequest>,
) -> Result<Json<EventAirspaceDto>, EventAirspaceError> {
    require_event_coordinator(&current_user)?;
    let event_id = parse_ulid_uuid(&eid, EventAirspaceError::InvalidEventId)?;
    let airspace_id = parse_ulid_uuid(&aid, EventAirspaceError::InvalidAirspaceId)?;
    let airspace = airspace_repository::update(
        services.db(),
        event_id,
        airspace_id,
        EventAirspaceSave::from(request),
    )
    .await
    .map_err(EventAirspaceError::Database)?
    .ok_or(EventAirspaceError::AirspaceNotFound)?;

    Ok(Json(EventAirspaceDto::from(airspace)))
}

async fn delete_airspace(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((eid, aid)): Path<(String, String)>,
) -> Result<Json<EventAirspaceDto>, EventAirspaceError> {
    require_event_coordinator(&current_user)?;
    let event_id = parse_ulid_uuid(&eid, EventAirspaceError::InvalidEventId)?;
    let airspace_id = parse_ulid_uuid(&aid, EventAirspaceError::InvalidAirspaceId)?;
    let airspace = airspace_repository::delete(services.db(), event_id, airspace_id)
        .await
        .map_err(EventAirspaceError::Database)?
        .ok_or(EventAirspaceError::AirspaceNotFound)?;

    Ok(Json(EventAirspaceDto::from(airspace)))
}

async fn ensure_event_exists(
    services: &Services,
    event_id: Uuid,
) -> Result<(), EventAirspaceError> {
    if event_repository::exists(services.db(), event_id)
        .await
        .map_err(EventAirspaceError::Database)?
    {
        Ok(())
    } else {
        Err(EventAirspaceError::EventNotFound)
    }
}

fn require_event_coordinator(current_user: &CurrentUser) -> Result<(), EventAirspaceError> {
    if current_user.has_role(UserRole::EventCoordinator) {
        Ok(())
    } else {
        Err(EventAirspaceError::Forbidden)
    }
}

fn parse_ulid_uuid(id: &str, error: EventAirspaceError) -> Result<Uuid, EventAirspaceError> {
    id.parse::<Ulid>().map(Uuid::from).map_err(|_| error)
}

#[derive(Deserialize)]
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

#[derive(Serialize)]
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

#[derive(Debug)]
enum EventAirspaceError {
    AirspaceNotFound,
    Database(sqlx::Error),
    EventNotFound,
    Forbidden,
    InvalidAirspaceId,
    InvalidEventId,
}

impl IntoResponse for EventAirspaceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            EventAirspaceError::AirspaceNotFound => {
                (StatusCode::NOT_FOUND, "event airspace not found".into())
            }
            EventAirspaceError::Database(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            EventAirspaceError::EventNotFound => (StatusCode::NOT_FOUND, "event not found".into()),
            EventAirspaceError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            EventAirspaceError::InvalidAirspaceId => {
                (StatusCode::BAD_REQUEST, "invalid airspace id".into())
            }
            EventAirspaceError::InvalidEventId => {
                (StatusCode::BAD_REQUEST, "invalid event id".into())
            }
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}
