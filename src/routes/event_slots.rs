use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
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
        event::event_slot::{self as slot_repository, EventSlotRecord, EventSlotSave},
    },
    services::Services,
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_slots, create_slot))]
pub(crate) struct ApiDoc;

pub fn build_event_slot_routes() -> Router<Services> {
    Router::new()
        .route("/{eid}/slots", get(list_slots))
        .route("/{eid}/slots/bookings.csv", get(export_bookings))
        .route("/{eid}/slots", post(create_slot))
}

#[utoipa::path(get, path = "api/events/{event_id}/slots", tag = "Events", params(("event_id" = String, Path, description = "Event ULID")), responses((status = 200, description = "Successful response", body = Vec<EventSlotDto>)))]
async fn list_slots(
    State(services): State<Services>,
    Path(eid): Path<String>,
) -> Result<Json<Vec<EventSlotDto>>, ApiError> {
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    ensure_event_exists(&services, event_id).await?;

    Ok(Json(
        slot_repository::list_by_event(services.db(), event_id)
            .await
            .map_err(ApiError::Database)?
            .into_iter()
            .map(|slot| event_slot_dto(slot, false))
            .collect(),
    ))
}

async fn get_slot(
    State(services): State<Services>,
    Path((eid, sid)): Path<(String, String)>,
) -> Result<Json<EventSlotDto>, ApiError> {
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    let slot_id = parse_ulid_uuid(&sid, ApiError::InvalidSlotId)?;
    let slot = slot_repository::find_by_event_and_id(services.db(), event_id, slot_id)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::SlotNotFound)?;

    Ok(Json(event_slot_dto(slot, false)))
}

async fn get_my_slot(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
) -> Result<Json<EventSlotDto>, ApiError> {
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let include_booking_user = include_booking_user(&current_user);
    let slot = slot_repository::find_mine_by_event(services.db(), event_id, user_id)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::SlotNotFound)?;

    Ok(Json(event_slot_dto(slot, include_booking_user)))
}

async fn export_bookings(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
) -> Result<Response, ApiError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| ApiError::Forbidden)?;
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    let rows = slot_repository::booking_export_rows(services.db(), event_id)
        .await
        .map_err(ApiError::Database)?;

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

#[utoipa::path(post, path = "api/events/{event_id}/slots", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID")), request_body = EventSlotSaveRequest, responses((status = 200, description = "Successful response", body = EventSlotDto)))]
async fn create_slot(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
    Json(request): Json<EventSlotSaveRequest>,
) -> Result<Json<EventSlotDto>, ApiError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| ApiError::Forbidden)?;
    let _event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    let slot = slot_repository::create(services.db(), request.try_into()?)
        .await
        .map_err(ApiError::Database)?;

    Ok(Json(event_slot_dto(
        slot,
        include_booking_user(&current_user),
    )))
}

#[utoipa::path(put, path = "api/events/{event_id}/slots/{slot_id}", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("slot_id" = String, Path, description = "Slot ULID")), responses((status = 200, description = "Successful response", body = EventSlotDto)))]
async fn update_slot(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((eid, sid)): Path<(String, String)>,
    Json(request): Json<EventSlotSaveRequest>,
) -> Result<Json<EventSlotDto>, ApiError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| ApiError::Forbidden)?;
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    let slot_id = parse_ulid_uuid(&sid, ApiError::InvalidSlotId)?;
    let slot = slot_repository::update(services.db(), event_id, slot_id, request.try_into()?)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::SlotNotFound)?;

    Ok(Json(event_slot_dto(
        slot,
        include_booking_user(&current_user),
    )))
}

#[utoipa::path(delete, path = "api/events/{event_id}/slots/{slot_id}", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("slot_id" = String, Path, description = "Slot ULID")), responses((status = 204, description = "No content")))]
async fn delete_slot(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((eid, sid)): Path<(String, String)>,
) -> Result<Json<EventSlotDto>, ApiError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| ApiError::Forbidden)?;
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    let slot_id = parse_ulid_uuid(&sid, ApiError::InvalidSlotId)?;
    let slot = slot_repository::delete(services.db(), event_id, slot_id)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::SlotNotFound)?;

    Ok(Json(event_slot_dto(
        slot,
        include_booking_user(&current_user),
    )))
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

fn include_booking_user(current_user: &CurrentUser) -> bool {
    current_user
        .require_any_role(&[UserRole::EventCoordinator, UserRole::Controller])
        .is_ok()
}

fn parse_ulid_uuid(id: &str, error: ApiError) -> Result<Uuid, ApiError> {
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

#[derive(Deserialize, utoipa::ToSchema)]
struct EventSlotSaveRequest {
    airspace_id: String,
    enter_at: DateTime<Utc>,
    leave_at: Option<DateTime<Utc>>,
    callsign: Option<String>,
    aircraft_type_icao: Option<String>,
}

impl TryFrom<EventSlotSaveRequest> for EventSlotSave {
    type Error = ApiError;

    fn try_from(request: EventSlotSaveRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            airspace_id: parse_ulid_uuid(&request.airspace_id, ApiError::InvalidAirspaceId)?,
            enter_at: request.enter_at,
            leave_at: request.leave_at,
            callsign: request.callsign,
            aircraft_type_icao: request.aircraft_type_icao,
        })
    }
}

#[derive(Serialize, utoipa::ToSchema)]
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

#[derive(Serialize, utoipa::ToSchema)]
struct EventBookingDto {
    id: String,
    user_id: String,
    user: Option<UserDto>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize, utoipa::ToSchema)]
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
