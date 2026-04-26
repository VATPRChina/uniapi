use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
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
        event_slot::{self as slot_repository, EventSlotRecord, EventSlotSave},
    },
    auth::CurrentUser,
    models::user_role::UserRole,
    services::Services,
};

pub fn build_public_event_slot_routes() -> Router<Services> {
    Router::new()
        .route("/{eid}/slots", get(list_slots))
        .route("/{eid}/slots/{sid}", get(get_slot))
}

pub fn build_protected_event_slot_routes() -> Router<Services> {
    Router::new()
        .route("/{eid}/slots/bookings.csv", get(export_bookings))
        .route("/{eid}/slots/mine", get(get_my_slot))
        .route("/{eid}/slots", post(create_slot))
        .route("/{eid}/slots/{sid}", put(update_slot).delete(delete_slot))
}

async fn list_slots(
    State(services): State<Services>,
    Path(eid): Path<String>,
) -> Result<Json<Vec<EventSlotDto>>, EventSlotError> {
    let event_id = parse_ulid_uuid(&eid, EventSlotError::InvalidEventId)?;
    ensure_event_exists(&services, event_id).await?;

    Ok(Json(
        slot_repository::list_by_event(services.db(), event_id)
            .await
            .map_err(EventSlotError::Database)?
            .into_iter()
            .map(|slot| event_slot_dto(slot, false))
            .collect(),
    ))
}

async fn get_slot(
    State(services): State<Services>,
    Path((eid, sid)): Path<(String, String)>,
) -> Result<Json<EventSlotDto>, EventSlotError> {
    let event_id = parse_ulid_uuid(&eid, EventSlotError::InvalidEventId)?;
    let slot_id = parse_ulid_uuid(&sid, EventSlotError::InvalidSlotId)?;
    let slot = slot_repository::find_by_event_and_id(services.db(), event_id, slot_id)
        .await
        .map_err(EventSlotError::Database)?
        .ok_or(EventSlotError::SlotNotFound)?;

    Ok(Json(event_slot_dto(slot, false)))
}

async fn get_my_slot(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
) -> Result<Json<EventSlotDto>, EventSlotError> {
    let event_id = parse_ulid_uuid(&eid, EventSlotError::InvalidEventId)?;
    let user_id = current_user.user_id.ok_or(EventSlotError::Unauthorized)?;
    let include_booking_user = include_booking_user(&current_user);
    let slot = slot_repository::find_mine_by_event(services.db(), event_id, user_id)
        .await
        .map_err(EventSlotError::Database)?
        .ok_or(EventSlotError::SlotNotFound)?;

    Ok(Json(event_slot_dto(slot, include_booking_user)))
}

async fn export_bookings(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
) -> Result<Response, EventSlotError> {
    require_event_coordinator(&current_user)?;
    let event_id = parse_ulid_uuid(&eid, EventSlotError::InvalidEventId)?;
    let rows = slot_repository::booking_export_rows(services.db(), event_id)
        .await
        .map_err(EventSlotError::Database)?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"bookings.csv\"",
            ),
        ],
        Body::from(rows.join("\n")),
    )
        .into_response())
}

async fn create_slot(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
    Json(request): Json<EventSlotSaveRequest>,
) -> Result<Json<EventSlotDto>, EventSlotError> {
    require_event_coordinator(&current_user)?;
    let _event_id = parse_ulid_uuid(&eid, EventSlotError::InvalidEventId)?;
    let slot = slot_repository::create(services.db(), request.try_into()?)
        .await
        .map_err(EventSlotError::Database)?;

    Ok(Json(event_slot_dto(
        slot,
        include_booking_user(&current_user),
    )))
}

async fn update_slot(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((eid, sid)): Path<(String, String)>,
    Json(request): Json<EventSlotSaveRequest>,
) -> Result<Json<EventSlotDto>, EventSlotError> {
    require_event_coordinator(&current_user)?;
    let event_id = parse_ulid_uuid(&eid, EventSlotError::InvalidEventId)?;
    let slot_id = parse_ulid_uuid(&sid, EventSlotError::InvalidSlotId)?;
    let slot = slot_repository::update(services.db(), event_id, slot_id, request.try_into()?)
        .await
        .map_err(EventSlotError::Database)?
        .ok_or(EventSlotError::SlotNotFound)?;

    Ok(Json(event_slot_dto(
        slot,
        include_booking_user(&current_user),
    )))
}

async fn delete_slot(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((eid, sid)): Path<(String, String)>,
) -> Result<Json<EventSlotDto>, EventSlotError> {
    require_event_coordinator(&current_user)?;
    let event_id = parse_ulid_uuid(&eid, EventSlotError::InvalidEventId)?;
    let slot_id = parse_ulid_uuid(&sid, EventSlotError::InvalidSlotId)?;
    let slot = slot_repository::delete(services.db(), event_id, slot_id)
        .await
        .map_err(EventSlotError::Database)?
        .ok_or(EventSlotError::SlotNotFound)?;

    Ok(Json(event_slot_dto(
        slot,
        include_booking_user(&current_user),
    )))
}

async fn ensure_event_exists(services: &Services, event_id: Uuid) -> Result<(), EventSlotError> {
    if event_repository::exists(services.db(), event_id)
        .await
        .map_err(EventSlotError::Database)?
    {
        Ok(())
    } else {
        Err(EventSlotError::EventNotFound)
    }
}

fn require_event_coordinator(current_user: &CurrentUser) -> Result<(), EventSlotError> {
    if current_user.has_role(UserRole::EventCoordinator) {
        Ok(())
    } else {
        Err(EventSlotError::Forbidden)
    }
}

fn include_booking_user(current_user: &CurrentUser) -> bool {
    current_user.has_role(UserRole::EventCoordinator) || current_user.has_role(UserRole::Controller)
}

fn parse_ulid_uuid(id: &str, error: EventSlotError) -> Result<Uuid, EventSlotError> {
    id.parse::<Ulid>().map(Uuid::from).map_err(|_| error)
}

fn event_slot_dto(slot: EventSlotRecord, include_booking_user: bool) -> EventSlotDto {
    let booking = event_booking_dto(&slot, include_booking_user);
    EventSlotDto {
        id: Ulid::from(slot.id).to_string(),
        event_id: Ulid::from(slot.event_id).to_string(),
        airspace_id: Ulid::from(slot.airspace_id).to_string(),
        airspace: EventAirspaceDto {
            id: Ulid::from(slot.airspace_id).to_string(),
            event_id: Ulid::from(slot.event_id).to_string(),
            name: slot.airspace_name,
            created_at: slot.airspace_created_at,
            updated_at: slot.airspace_updated_at,
            icao_codes: slot.airspace_icao_codes,
            description: slot.airspace_description,
        },
        enter_at: slot.enter_at,
        leave_at: slot.leave_at,
        created_at: slot.created_at,
        updated_at: slot.updated_at,
        booking,
        callsign: slot.callsign,
        aircraft_type_icao: slot.aircraft_type_icao,
    }
}

fn event_booking_dto(slot: &EventSlotRecord, include_user: bool) -> Option<EventBookingDto> {
    let id = slot.booking_id?;
    Some(EventBookingDto {
        id: Ulid::from(id).to_string(),
        user_id: Ulid::from(slot.booking_user_id?).to_string(),
        user: if include_user {
            Some(UserDto {
                id: Ulid::from(slot.booking_user_id?).to_string(),
                cid: slot.booking_user_cid.clone()?,
                full_name: String::new(),
                created_at: slot.booking_user_created_at?,
                updated_at: slot.booking_user_updated_at?,
                roles: slot.booking_user_roles.clone().unwrap_or_default(),
                direct_roles: slot.booking_user_roles.clone().unwrap_or_default(),
                moodle_account: None,
            })
        } else {
            None
        },
        created_at: slot.booking_created_at?,
        updated_at: slot.booking_updated_at?,
    })
}

#[derive(Deserialize)]
struct EventSlotSaveRequest {
    airspace_id: String,
    enter_at: DateTime<Utc>,
    leave_at: Option<DateTime<Utc>>,
    callsign: Option<String>,
    aircraft_type_icao: Option<String>,
}

impl TryFrom<EventSlotSaveRequest> for EventSlotSave {
    type Error = EventSlotError;

    fn try_from(request: EventSlotSaveRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            airspace_id: parse_ulid_uuid(&request.airspace_id, EventSlotError::InvalidAirspaceId)?,
            enter_at: request.enter_at,
            leave_at: request.leave_at,
            callsign: request.callsign,
            aircraft_type_icao: request.aircraft_type_icao,
        })
    }
}

#[derive(Serialize)]
struct EventSlotDto {
    id: String,
    event_id: String,
    airspace_id: String,
    airspace: EventAirspaceDto,
    enter_at: DateTime<Utc>,
    leave_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    booking: Option<EventBookingDto>,
    callsign: Option<String>,
    aircraft_type_icao: Option<String>,
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

#[derive(Serialize)]
struct EventBookingDto {
    id: String,
    user_id: String,
    user: Option<UserDto>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
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

#[derive(Debug)]
enum EventSlotError {
    Database(sqlx::Error),
    EventNotFound,
    Forbidden,
    InvalidAirspaceId,
    InvalidEventId,
    InvalidSlotId,
    SlotNotFound,
    Unauthorized,
}

impl IntoResponse for EventSlotError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            EventSlotError::Database(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            EventSlotError::EventNotFound => (StatusCode::NOT_FOUND, "event not found".into()),
            EventSlotError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            EventSlotError::InvalidAirspaceId => {
                (StatusCode::BAD_REQUEST, "invalid airspace id".into())
            }
            EventSlotError::InvalidEventId => (StatusCode::BAD_REQUEST, "invalid event id".into()),
            EventSlotError::InvalidSlotId => (StatusCode::BAD_REQUEST, "invalid slot id".into()),
            EventSlotError::SlotNotFound => (StatusCode::NOT_FOUND, "event slot not found".into()),
            EventSlotError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}
