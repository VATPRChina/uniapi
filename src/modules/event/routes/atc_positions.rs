use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use ulid::Ulid;

use crate::error::ApiError;
use crate::modules::user::models::UserRole;
use crate::modules::event::dto::{
    EventAtcPositionBookRequest, EventAtcPositionBookingDto, EventAtcPositionDto,
    EventAtcPositionSaveRequest,
};
use crate::modules::event::service::EventAtcPositionView;
use crate::modules::user::middleware::CurrentUser;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    list_positions,
    create_position,
    update_position,
    delete_position,
    book_position,
    cancel_position_booking
))]
pub(crate) struct ApiDoc;

pub fn build_event_atc_position_routes() -> Router<Services> {
    Router::new()
        .route("/{event_id}/controllers", get(list_positions))
        .route("/{event_id}/controllers", post(create_position))
        .route(
            "/{event_id}/controllers/{position_id}",
            put(update_position).delete(delete_position),
        )
        .route(
            "/{event_id}/controllers/{position_id}/booking",
            put(book_position).delete(cancel_position_booking),
        )
}

#[utoipa::path(get, path = "api/events/{event_id}/controllers", tag = "Events", params(("event_id" = String, Path, description = "Event ULID")), responses((status = 200, description = "Successful response", body = Vec<EventAtcPositionDto>)))]
async fn list_positions(
    State(services): State<Services>,
    Path(event_id): Path<String>,
) -> Result<Json<Vec<EventAtcPositionDto>>, ApiError> {
    let event_id = event_id.parse::<Ulid>()?.into();
    Ok(Json(
        services
            .event()
            .list_atc_positions(event_id)
            .await?
            .into_iter()
            .map(position_to_dto)
            .collect(),
    ))
}

#[utoipa::path(post, path = "api/events/{event_id}/controllers", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID")), request_body = EventAtcPositionSaveRequest, responses((status = 200, description = "Successful response", body = EventAtcPositionDto)))]
async fn create_position(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(event_id): Path<String>,
    Json(request): Json<EventAtcPositionSaveRequest>,
) -> Result<Json<EventAtcPositionDto>, ApiError> {
    require_edit_role(&current_user)?;
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let event_id = event_id.parse::<Ulid>()?.into();
    let position = services
        .event()
        .create_atc_position(event_id, request.try_into()?, operated_by)
        .await?;

    Ok(Json(position_to_dto(position)))
}

#[utoipa::path(put, path = "api/events/{event_id}/controllers/{position_id}", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("position_id" = String, Path, description = "Position ULID")), request_body = EventAtcPositionSaveRequest, responses((status = 200, description = "Successful response", body = EventAtcPositionDto)))]
async fn update_position(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((event_id, position_id)): Path<(String, String)>,
    Json(request): Json<EventAtcPositionSaveRequest>,
) -> Result<Json<EventAtcPositionDto>, ApiError> {
    require_edit_role(&current_user)?;
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let event_id = event_id.parse::<Ulid>()?.into();
    let position_id = position_id.parse::<Ulid>()?.into();
    let position = services
        .event()
        .update_atc_position(event_id, position_id, request.try_into()?, operated_by)
        .await?;

    Ok(Json(position_to_dto(position)))
}

#[utoipa::path(delete, path = "api/events/{event_id}/controllers/{position_id}", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("position_id" = String, Path, description = "Position ULID")), responses((status = 204, description = "No content")))]
async fn delete_position(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((event_id, position_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    require_edit_role(&current_user)?;
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let event_id = event_id.parse::<Ulid>()?.into();
    let position_id = position_id.parse::<Ulid>()?.into();
    services
        .event()
        .delete_atc_position(event_id, position_id, operated_by)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(put, path = "api/events/{event_id}/controllers/{position_id}/booking", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("position_id" = String, Path, description = "Position ULID")), request_body = EventAtcPositionBookRequest, responses((status = 200, description = "Successful response", body = EventAtcPositionBookingDto)))]
async fn book_position(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((event_id, position_id)): Path<(String, String)>,
    Json(request): Json<EventAtcPositionBookRequest>,
) -> Result<Json<EventAtcPositionBookingDto>, ApiError> {
    current_user.require_role(UserRole::Controller)?;
    let event_id = event_id.parse::<Ulid>()?.into();
    let position_id = position_id.parse::<Ulid>()?.into();
    if request.user_id.is_some() && !has_booking_admin_role(&current_user) {
        return Err(ApiError::forbidden([
            UserRole::EventCoordinator,
            UserRole::ControllerTrainingDirectorAssistant,
            UserRole::ControllerTrainingMentor,
        ]));
    }
    let user_id = match request.user_id.as_deref() {
        Some(user_id) => (user_id).parse::<Ulid>()?.into(),
        None => current_user.user_id.ok_or(ApiError::Unauthorized)?,
    };
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let is_admin_booking = request.user_id.is_some();
    let position = services
        .event()
        .book_atc_position(
            event_id,
            position_id,
            user_id,
            operated_by,
            is_admin_booking,
        )
        .await?;
    position_booking_to_dto(position).map(Json)
}

#[utoipa::path(delete, path = "api/events/{event_id}/controllers/{position_id}/booking", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("position_id" = String, Path, description = "Position ULID")), responses((status = 200, description = "Successful response", body = EventAtcPositionBookingDto)))]
async fn cancel_position_booking(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((event_id, position_id)): Path<(String, String)>,
) -> Result<Json<EventAtcPositionBookingDto>, ApiError> {
    current_user.require_role(UserRole::Controller)?;
    let event_id = event_id.parse::<Ulid>()?.into();
    let position_id = position_id.parse::<Ulid>()?.into();
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let position = services
        .event()
        .cancel_atc_position_booking(
            event_id,
            position_id,
            current_user_id,
            has_booking_admin_role(&current_user),
        )
        .await?;
    Ok(Json(position_booking_to_dto(position)?))
}

fn position_to_dto(view: EventAtcPositionView) -> EventAtcPositionDto {
    EventAtcPositionDto::from_entity(
        view.position,
        view.event,
        view.booking.map(|booking| (booking.booking, booking.user)),
    )
}

fn position_booking_to_dto(
    view: EventAtcPositionView,
) -> Result<EventAtcPositionBookingDto, ApiError> {
    let booking = view.booking.ok_or(ApiError::PositionNotBooked)?;
    EventAtcPositionBookingDto::from_entity(booking.booking, booking.user)
}

fn require_edit_role(current_user: &CurrentUser) -> Result<(), ApiError> {
    current_user
        .require_any_role(&[
            UserRole::EventCoordinator,
            UserRole::ControllerTrainingDirectorAssistant,
            UserRole::OperationDirectorAssistant,
        ])
        .map_err(Into::into)
}

fn has_booking_admin_role(current_user: &CurrentUser) -> bool {
    current_user
        .require_any_role(&[
            UserRole::EventCoordinator,
            UserRole::ControllerTrainingDirectorAssistant,
            UserRole::ControllerTrainingMentor,
        ])
        .is_ok()
}
