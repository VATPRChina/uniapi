use axum::extract::{Path, State};
use axum::routing::put;
use axum::{Json, Router};

use crate::auth::CurrentUser;
use crate::dto::parse_ulid_uuid;
use crate::model::user_role::UserRole;
use crate::modules::event::dto::{EventBookingDto, EventSlotBookingRequest};
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(put_booking, delete_booking))]
pub(crate) struct ApiDoc;

pub fn build_event_slot_booking_routes() -> Router<Services> {
    Router::new().route(
        "/{eid}/slots/{sid}/booking",
        put(put_booking).delete(delete_booking),
    )
}

#[utoipa::path(put, path = "api/events/{event_id}/slots/{slot_id}/booking", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("slot_id" = String, Path, description = "Slot ULID")), request_body = EventSlotBookingRequest, responses((status = 200, description = "Successful response", body = EventBookingDto)))]
async fn put_booking(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((eid, sid)): Path<(String, String)>,
    Json(request): Json<EventSlotBookingRequest>,
) -> Result<Json<EventBookingDto>, ApiError> {
    let event_id = parse_ulid_uuid("event_id", &eid)?;
    let slot_id = parse_ulid_uuid("slot_id", &sid)?;
    if request.user_id.is_some() && !current_user.has_role(UserRole::EventCoordinator) {
        return Err(ApiError::forbidden([UserRole::EventCoordinator]));
    }
    let is_admin_booking = request.user_id.is_some();
    let user_id = match request.user_id.as_deref() {
        Some(user_id) => parse_ulid_uuid("user_id", user_id)?,
        None => current_user.user_id.ok_or(ApiError::Unauthorized)?,
    };

    let booking = services
        .event()
        .create_slot_booking(event_id, slot_id, user_id, is_admin_booking)
        .await?;

    Ok(Json(EventBookingDto::from_entity(
        booking.booking,
        booking.user,
        include_user(&current_user),
    )))
}

#[utoipa::path(delete, path = "api/events/{event_id}/slots/{slot_id}/booking", operation_id = "delete_event_slot_booking", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("slot_id" = String, Path, description = "Slot ULID")), responses((status = 200, description = "Successful response", body = EventBookingDto)))]
async fn delete_booking(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((eid, sid)): Path<(String, String)>,
) -> Result<Json<EventBookingDto>, ApiError> {
    let event_id = parse_ulid_uuid("event_id", &eid)?;
    let slot_id = parse_ulid_uuid("slot_id", &sid)?;
    let is_admin = current_user.has_role(UserRole::EventCoordinator);
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let booking = services
        .event()
        .delete_slot_booking(event_id, slot_id, current_user_id, is_admin)
        .await?;

    Ok(Json(EventBookingDto::from_entity(
        booking.booking,
        booking.user,
        include_user(&current_user),
    )))
}

fn include_user(current_user: &CurrentUser) -> bool {
    current_user
        .require_any_role(&[UserRole::EventCoordinator, UserRole::Controller])
        .is_ok()
}
