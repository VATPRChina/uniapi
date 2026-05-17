use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::user_role::UserRole;
use crate::repository::event::event as event_repository;
use crate::repository::event::event_airspace::{self as airspace_repository};
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
    let event_id = parse_ulid_uuid("event_id", &eid)?;
    ensure_event_exists(&services, event_id).await?;
    let airspace = airspace_repository::create(services.db(), event_id, request.into()).await?;

    Ok(Json(EventAirspaceDto::from(airspace)))
}

async fn ensure_event_exists(services: &Services, event_id: Uuid) -> Result<(), ApiError> {
    if event_repository::exists(services.db(), event_id).await? {
        Ok(())
    } else {
        Err(ApiError::not_found("event", "unknown"))
    }
}
