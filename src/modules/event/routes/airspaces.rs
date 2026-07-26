use crate::error::ApiError;
use crate::model::user_role::UserRole;
use crate::modules::event::dto::{EventAirspaceDto, EventAirspaceSaveRequest};
use crate::modules::user::middleware::CurrentUser;
use crate::services::Services;
use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use ulid::Ulid;

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
    let event_id = eid.parse::<Ulid>()?.into();
    let airspace = services
        .event()
        .create_airspace(event_id, request.into())
        .await?;

    Ok(Json(EventAirspaceDto::from(airspace)))
}
