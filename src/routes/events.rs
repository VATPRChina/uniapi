use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::routes::ApiError;
use crate::{
    auth::CurrentUser,
    models::user_role::UserRole,
    repository::event::event::{self as event_repository, EventRecord, EventSave},
    services::Services,
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    list_events,
    list_past_events,
    create_event,
    get_event,
    update_event
))]
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
        event_repository::list_current(services.db())
            .await
            .map_err(ApiError::Database)?
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
        event_repository::list_past(services.db(), query.until)
            .await
            .map_err(ApiError::Database)?
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
    let id = parse_ulid_uuid(&eid)?;
    let event = event_repository::find_by_id(services.db(), id)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::EventNotFound)?;

    Ok(Json(EventDto::from(event)))
}

#[utoipa::path(post, path = "api/events", tag = "Events", security(("oauth2" = [])), request_body = EventSaveRequest, responses((status = 200, description = "Successful response", body = EventDto)))]
async fn create_event(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<EventSaveRequest>,
) -> Result<Json<EventDto>, ApiError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| ApiError::Forbidden)?;
    let event = event_repository::create(services.db(), request.try_into()?)
        .await
        .map_err(ApiError::Database)?;

    Ok(Json(EventDto::from(event)))
}

#[utoipa::path(put, path = "api/events/{id}", tag = "Events", security(("oauth2" = [])), params(("id" = String, Path, description = "Event ULID")), request_body = EventSaveRequest, responses((status = 200, description = "Successful response", body = EventDto)))]
async fn update_event(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
    Json(request): Json<EventSaveRequest>,
) -> Result<Json<EventDto>, ApiError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| ApiError::Forbidden)?;
    let id = parse_ulid_uuid(&eid)?;
    let event = event_repository::update(services.db(), id, request.try_into()?)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::EventNotFound)?;

    Ok(Json(EventDto::from(event)))
}

#[utoipa::path(delete, path = "api/events/{id}", tag = "Events", security(("oauth2" = [])), params(("id" = String, Path, description = "Event ULID")), responses((status = 200, description = "Successful response", body = EventDto)))]
async fn delete_event(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(eid): Path<String>,
) -> Result<Json<EventDto>, ApiError> {
    current_user
        .require_role(UserRole::EventCoordinator)
        .map_err(|_| ApiError::Forbidden)?;
    let id = parse_ulid_uuid(&eid)?;
    let event = event_repository::delete(services.db(), id)
        .await
        .map_err(ApiError::Database)?
        .ok_or(ApiError::EventNotFound)?;

    Ok(Json(EventDto::from(event)))
}

fn parse_ulid_uuid(id: &str) -> Result<Uuid, ApiError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| ApiError::InvalidEventId)
}

#[derive(Deserialize, utoipa::ToSchema)]
struct ListPastQuery {
    until: Option<DateTime<Utc>>,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct EventSaveRequest {
    title: String,
    title_en: Option<String>,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    start_booking_at: Option<DateTime<Utc>>,
    end_booking_at: Option<DateTime<Utc>>,
    start_atc_booking_at: Option<DateTime<Utc>>,
    image_url: Option<String>,
    community_link: Option<String>,
    vatsim_link: Option<String>,
    description: String,
}

impl TryFrom<EventSaveRequest> for EventSave {
    type Error = ApiError;

    fn try_from(request: EventSaveRequest) -> Result<Self, Self::Error> {
        if request.start_booking_at.is_some() ^ request.end_booking_at.is_some() {
            return Err(ApiError::BadRequest(
                "Both StartBookingAt and EndBookingAt must be set or both must be null.".into(),
            ));
        }

        Ok(Self {
            title: request.title,
            title_en: request.title_en,
            start_at: request.start_at,
            end_at: request.end_at,
            start_booking_at: request.start_booking_at,
            end_booking_at: request.end_booking_at,
            start_atc_booking_at: request.start_atc_booking_at,
            image_url: request.image_url,
            community_link: request.community_link,
            vatsim_link: request.vatsim_link,
            description: request.description,
        })
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct EventDto {
    id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    title: String,
    title_en: Option<String>,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    start_booking_at: Option<DateTime<Utc>>,
    end_booking_at: Option<DateTime<Utc>>,
    start_atc_booking_at: Option<DateTime<Utc>>,
    image_url: Option<String>,
    community_link: Option<String>,
    vatsim_link: Option<String>,
    description: String,
}

impl From<EventRecord> for EventDto {
    fn from(event: EventRecord) -> Self {
        Self {
            id: Ulid::from(event.id).to_string(),
            created_at: event.created_at,
            updated_at: event.updated_at,
            title: event.title,
            title_en: event.title_en,
            start_at: event.start_at,
            end_at: event.end_at,
            start_booking_at: event.start_booking_at,
            end_booking_at: event.end_booking_at,
            start_atc_booking_at: event.start_atc_booking_at,
            image_url: event.image_url,
            community_link: event.community_link,
            vatsim_link: event.vatsim_link,
            description: event.description,
        }
    }
}
