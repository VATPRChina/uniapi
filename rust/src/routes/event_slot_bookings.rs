use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::routes::ApiError;
use crate::{
    auth::CurrentUser,
    models::user_role::UserRole,
    repository::event::event_slot_booking::{self as booking_repository, EventBookingRecord},
    services::Services,
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_booking, put_booking, delete_booking))]
pub(crate) struct ApiDoc;

pub fn build_event_slot_booking_routes() -> Router<Services> {
    Router::new().route(
        "/{eid}/slots/{sid}/booking",
        get(get_booking).put(put_booking).delete(delete_booking),
    )
}

#[utoipa::path(get, path = "api/events/{event_id}/slots/{slot_id}/booking", tag = "Events", params(("event_id" = String, Path, description = "Event ULID"), ("slot_id" = String, Path, description = "Slot ULID")), responses((status = 200, description = "Successful response", body = EventBookingDto)))]
async fn get_booking(
    State(services): State<Services>,
    Path((eid, sid)): Path<(String, String)>,
) -> Result<Json<EventBookingDto>, ApiError> {
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    let slot_id = parse_ulid_uuid(&sid, ApiError::InvalidSlotId)?;
    let booking = booking_repository::find_booking(services.db(), event_id, slot_id)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::SlotNotBooked)?;

    Ok(Json(event_booking_dto(booking, false)))
}

#[utoipa::path(put, path = "api/events/{event_id}/slots/{slot_id}/booking", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("slot_id" = String, Path, description = "Slot ULID")), responses((status = 200, description = "Successful response", body = EventBookingDto)))]
async fn put_booking(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((eid, sid)): Path<(String, String)>,
    Json(request): Json<EventSlotBookingRequest>,
) -> Result<Json<EventBookingDto>, ApiError> {
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    let slot_id = parse_ulid_uuid(&sid, ApiError::InvalidSlotId)?;
    if request.user_id.is_some() && !current_user.has_role(UserRole::EventCoordinator) {
        return Err(ApiError::Forbidden);
    }
    let is_admin_booking = request.user_id.is_some();
    let user_id = match request.user_id.as_deref() {
        Some(user_id) => parse_ulid_uuid(user_id, ApiError::InvalidUserId)?,
        None => current_user.user_id.ok_or(ApiError::Unauthorized)?,
    };

    let mut transaction = services.db().begin().await.map_err(ApiError::Database)?;
    let state = booking_repository::load_state_for_update(&mut transaction, event_id, slot_id)
        .await
        .map_err(ApiError::Database)?;
    if !state.event_exists {
        return Err(ApiError::EventNotFound);
    }
    if !state.slot_exists {
        return Err(ApiError::SlotNotFound);
    }
    if state.booking_id.is_some() {
        return Err(ApiError::SlotBooked);
    }
    if !state.is_in_booking_period && !is_admin_booking {
        return Err(ApiError::EventNotInBookingTime);
    }

    booking_repository::create_booking(&mut transaction, slot_id, user_id)
        .await
        .map_err(ApiError::Database)?;
    transaction.commit().await.map_err(ApiError::Database)?;

    let booking = booking_repository::find_booking(services.db(), event_id, slot_id)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::SlotNotBooked)?;

    Ok(Json(event_booking_dto(
        booking,
        include_user(&current_user),
    )))
}

#[utoipa::path(delete, path = "api/events/{event_id}/slots/{slot_id}/booking", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("slot_id" = String, Path, description = "Slot ULID")), responses((status = 200, description = "Successful response", body = EventBookingDto)))]
async fn delete_booking(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((eid, sid)): Path<(String, String)>,
) -> Result<Json<EventBookingDto>, ApiError> {
    let event_id = parse_ulid_uuid(&eid, ApiError::InvalidEventId)?;
    let slot_id = parse_ulid_uuid(&sid, ApiError::InvalidSlotId)?;
    let mut transaction = services.db().begin().await.map_err(ApiError::Database)?;
    let state = booking_repository::load_state_for_update(&mut transaction, event_id, slot_id)
        .await
        .map_err(ApiError::Database)?;
    if !state.slot_exists {
        return Err(ApiError::SlotNotFound);
    }
    let Some(booking_id) = state.booking_id else {
        return Err(ApiError::SlotNotBooked);
    };
    if !state.is_in_booking_period {
        return Err(ApiError::EventNotInBookingTime);
    }
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let is_admin = current_user.has_role(UserRole::EventCoordinator);
    if state.booking_user_id != Some(current_user_id) && !is_admin {
        return Err(ApiError::SlotBookedByAnotherUser);
    }

    let booking = booking_repository::find_booking(services.db(), event_id, slot_id)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::SlotNotBooked)?;
    booking_repository::delete_booking(&mut transaction, booking_id)
        .await
        .map_err(ApiError::Database)?;
    transaction.commit().await.map_err(ApiError::Database)?;

    Ok(Json(event_booking_dto(
        booking,
        include_user(&current_user),
    )))
}

fn include_user(current_user: &CurrentUser) -> bool {
    current_user
        .require_any_role(&[UserRole::EventCoordinator, UserRole::Controller])
        .is_ok()
}

fn parse_ulid_uuid(id: &str, error: ApiError) -> Result<Uuid, ApiError> {
    id.parse::<Ulid>().map(Uuid::from).map_err(|_| error)
}

fn event_booking_dto(booking: EventBookingRecord, include_user: bool) -> EventBookingDto {
    EventBookingDto {
        id: Ulid::from(booking.id).to_string(),
        user_id: Ulid::from(booking.user_id).to_string(),
        user: if include_user {
            Some(UserDto {
                id: Ulid::from(booking.user_id).to_string(),
                cid: booking.user_cid.unwrap_or_default(),
                full_name: String::new(),
                created_at: booking.user_created_at.unwrap_or(booking.created_at),
                updated_at: booking.user_updated_at.unwrap_or(booking.updated_at),
                roles: booking.user_roles.clone().unwrap_or_default(),
                direct_roles: booking.user_roles.unwrap_or_default(),
                moodle_account: None,
            })
        } else {
            None
        },
        created_at: booking.created_at,
        updated_at: booking.updated_at,
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
struct EventSlotBookingRequest {
    user_id: Option<String>,
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
