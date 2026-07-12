use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::audit_log::AuditLogEntity;
use crate::model::user_role::UserRole;
use crate::repository::event::event::EventRepositoryExt;
use crate::repository::event::event_slot::EventSlotRepositoryExt;
use crate::routes::ApiError;
use crate::services::Services;

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
    let event_id = parse_ulid_uuid("event_id", &eid)?;
    ensure_event_exists(&services, event_id).await?;

    Ok(Json(
        services
            .db()
            .list_event_slot_by_event(event_id)
            .await?
            .into_iter()
            .map(|slot| EventSlotDto::from_record(slot, false))
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
    let event_id = parse_ulid_uuid("event_id", &eid)?;
    ensure_event_exists(&services, event_id).await?;
    let rows = services
        .db()
        .booking_event_slot_export_rows(event_id)
        .await?;

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
    let event_id = parse_ulid_uuid("event_id", &eid)?;
    let mut transaction = services.db().begin().await?;
    let slot = (&mut *transaction)
        .create_event_slot(request.try_into()?)
        .await?;
    if slot.event_id != event_id {
        return Err(ApiError::not_found("event airspace", "unknown"));
    }
    services
        .audit_log()
        .record(
            AuditLogEntity::EventSlot(event_id, slot.id),
            operated_by,
            None,
            Some(&slot),
        )
        .await?;
    transaction.commit().await?;

    Ok(Json(EventSlotDto::from_record(
        slot,
        include_booking_user(&current_user),
    )))
}

async fn ensure_event_exists(services: &Services, event_id: Uuid) -> Result<(), ApiError> {
    if services.db().exists_event(event_id).await? {
        Ok(())
    } else {
        Err(ApiError::not_found("event", "unknown"))
    }
}

fn include_booking_user(current_user: &CurrentUser) -> bool {
    current_user
        .require_any_role(&[UserRole::EventCoordinator, UserRole::Controller])
        .is_ok()
}
