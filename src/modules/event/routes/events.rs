use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use ulid::Ulid;

use crate::error::ApiError;
use crate::model::user_role::UserRole;
use crate::modules::event::dto::{EventDto, EventSaveRequest, ListPastQuery};
use crate::modules::user::middleware::CurrentUser;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_events, list_past_events, create_event, get_event, update_event))]
pub(crate) struct ApiDoc;

pub fn build_event_routes() -> Router<Services> {
    Router::new()
        .route("/", get(list_events))
        .route("/past", get(list_past_events))
        .route("/{eid}", get(get_event))
        .route("/", post(create_event))
        .route("/{eid}", put(update_event))
}

#[utoipa::path(get, path = "api/events", tag = "Events", responses((status = 200, description = "Successful response", body = Vec<EventDto>)))]
async fn list_events(State(services): State<Services>) -> Result<Json<Vec<EventDto>>, ApiError> {
    Ok(Json(
        services
            .event()
            .list_current()
            .await?
            .into_iter()
            .map(EventDto::from)
            .collect(),
    ))
}

#[utoipa::path(
    get,
    path = "api/events/past",
    tag = "Events",
    params(("until" = Option<DateTime<Utc>>, Query, description = "Latest event start time to include")),
    responses((status = 200, description = "Successful response", body = Vec<EventDto>))
)]
async fn list_past_events(
    State(services): State<Services>,
    Query(query): Query<ListPastQuery>,
) -> Result<Json<Vec<EventDto>>, ApiError> {
    Ok(Json(
        services
            .event()
            .list_past(query.until)
            .await?
            .into_iter()
            .map(EventDto::from)
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/events/{id}", tag = "Events", params(("id" = String, Path, description = "Event ULID")), responses((status = 200, description = "Successful response", body = EventDto)))]
async fn get_event(
    State(services): State<Services>,
    Path(eid): Path<String>,
) -> Result<Json<EventDto>, ApiError> {
    let id = eid.parse::<Ulid>()?.into();
    let event = services.event().find(id).await?;

    Ok(Json(EventDto::from(event)))
}

#[utoipa::path(post, path = "api/events", tag = "Events", security(("oauth2" = [])), request_body = EventSaveRequest, responses((status = 200, description = "Successful response", body = EventDto)))]
async fn create_event(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<EventSaveRequest>,
) -> Result<Json<EventDto>, ApiError> {
    current_user.require_role(UserRole::EventCoordinator)?;
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let event = services
        .event()
        .create(request.try_into()?, operated_by)
        .await?;

    Ok(Json(EventDto::from(event)))
}

#[utoipa::path(put, path = "api/events/{id}", tag = "Events", security(("oauth2" = [])), params(("id" = String, Path, description = "Event ULID")), request_body = EventSaveRequest, responses((status = 200, description = "Successful response", body = EventDto)))]
async fn update_event(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
    Json(request): Json<EventSaveRequest>,
) -> Result<Json<EventDto>, ApiError> {
    current_user.require_role(UserRole::EventCoordinator)?;
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let id = eid.parse::<Ulid>()?.into();
    let event = services
        .event()
        .update(id, request.try_into()?, operated_by)
        .await?;

    Ok(Json(EventDto::from(event)))
}
