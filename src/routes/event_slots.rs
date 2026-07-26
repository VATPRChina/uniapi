use crate::model::user_role::UserRole;
use crate::modules::event::dto::{EventSlotDto, EventSlotSaveRequest};
use crate::modules::event::service::EventSlotView;
use crate::modules::user::middleware::CurrentUser;
use crate::routes::ApiError;
use crate::services::Services;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use ulid::Ulid;

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_slots, export_bookings, create_slot))]
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
    let event_id = eid.parse::<Ulid>()?.into();
    Ok(Json(
        services
            .event()
            .list_slots(event_id)
            .await?
            .into_iter()
            .map(|view| slot_to_dto(view, false))
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/events/{event_id}/slots/bookings.csv", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID")), responses((status = 200, description = "CSV export of slot bookings", content_type = "text/csv")))]
async fn export_bookings(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
) -> Result<Response, ApiError> {
    current_user.require_role(UserRole::EventCoordinator)?;
    let event_id = eid.parse::<Ulid>()?.into();
    let rows = services.event().export_slot_bookings(event_id).await?;

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
    current_user.require_role(UserRole::EventCoordinator)?;
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let event_id = eid.parse::<Ulid>()?.into();
    let slot = services
        .event()
        .create_slot(event_id, request.try_into()?, operated_by)
        .await?;

    Ok(Json(slot_to_dto(slot, include_booking_user(&current_user))))
}

fn slot_to_dto(view: EventSlotView, include_booking_user: bool) -> EventSlotDto {
    EventSlotDto::from_entity(
        view.slot,
        view.airspace,
        view.booking.map(|booking| (booking.booking, booking.user)),
        include_booking_user,
    )
}

fn include_booking_user(current_user: &CurrentUser) -> bool {
    current_user
        .require_any_role(&[UserRole::EventCoordinator, UserRole::Controller])
        .is_ok()
}
