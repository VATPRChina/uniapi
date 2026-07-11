use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::Utc;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::audit_log::{AuditLog, AuditLogEntity};
use crate::model::user_role::UserRole;
use crate::repository::audit_log as audit_log_repository;
use crate::repository::event::event_atc_position::{
    self as position_repository, EventAtcPositionRecord, UserAtcPermissionRecord,
};
use crate::routes::ApiError;
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
    let event_id = parse_ulid_uuid("event_id", &event_id)?;
    Ok(Json(
        position_repository::list_by_event(services.db(), event_id)
            .await?
            .into_iter()
            .map(EventAtcPositionDto::from)
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
    let event_id = parse_ulid_uuid("event_id", &event_id)?;
    let mut transaction = services.db().begin().await?;
    let position =
        position_repository::create(&mut transaction, event_id, request.try_into()?).await?;
    create_position_audit_log(
        &mut transaction,
        &position,
        serde_json::Value::Null,
        serde_json::to_value(&position).map_err(|_| ApiError::Internal)?,
        operated_by,
    )
    .await?;
    transaction.commit().await?;

    Ok(Json(EventAtcPositionDto::from(position)))
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
    let event_id = parse_ulid_uuid("event_id", &event_id)?;
    let position_id = parse_ulid_uuid("position_id", &position_id)?;
    let mut transaction = services.db().begin().await?;
    let before = position_repository::find_by_event_and_id_in_transaction(
        &mut transaction,
        event_id,
        position_id,
        true,
    )
    .await?
    .ok_or(ApiError::not_found("event ATC position", "unknown"))?;
    let position =
        position_repository::update(&mut transaction, event_id, position_id, request.try_into()?)
            .await?
            .ok_or(ApiError::not_found("event ATC position", "unknown"))?;
    create_position_audit_log(
        &mut transaction,
        &position,
        serde_json::to_value(before).map_err(|_| ApiError::Internal)?,
        serde_json::to_value(&position).map_err(|_| ApiError::Internal)?,
        operated_by,
    )
    .await?;
    transaction.commit().await?;

    Ok(Json(EventAtcPositionDto::from(position)))
}

#[utoipa::path(delete, path = "api/events/{event_id}/controllers/{position_id}", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("position_id" = String, Path, description = "Position ULID")), responses((status = 204, description = "No content")))]
async fn delete_position(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((event_id, position_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    require_edit_role(&current_user)?;
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let event_id = parse_ulid_uuid("event_id", &event_id)?;
    let position_id = parse_ulid_uuid("position_id", &position_id)?;
    let mut transaction = services.db().begin().await?;
    let position = position_repository::find_by_event_and_id_in_transaction(
        &mut transaction,
        event_id,
        position_id,
        true,
    )
    .await?
    .ok_or(ApiError::not_found("event ATC position", "unknown"))?;
    if !position_repository::delete(&mut transaction, event_id, position_id).await? {
        return Err(ApiError::not_found("event ATC position", "unknown"));
    }
    create_position_audit_log(
        &mut transaction,
        &position,
        serde_json::to_value(&position).map_err(|_| ApiError::Internal)?,
        serde_json::Value::Null,
        operated_by,
    )
    .await?;
    transaction.commit().await?;

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
    let event_id = parse_ulid_uuid("event_id", &event_id)?;
    let position_id = parse_ulid_uuid("position_id", &position_id)?;
    if request.user_id.is_some() && !has_booking_admin_role(&current_user) {
        return Err(ApiError::forbidden([
            UserRole::EventCoordinator,
            UserRole::ControllerTrainingDirectorAssistant,
            UserRole::ControllerTrainingMentor,
        ]));
    }
    let user_id = match request.user_id.as_deref() {
        Some(user_id) => parse_ulid_uuid("user_id", user_id)?,
        None => current_user.user_id.ok_or(ApiError::Unauthorized)?,
    };
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let is_admin_booking = request.user_id.is_some();
    let mut transaction = services.db().begin().await?;
    let position = position_repository::find_by_event_and_id_in_transaction(
        &mut transaction,
        event_id,
        position_id,
        true,
    )
    .await?
    .ok_or(ApiError::not_found("event ATC position", "unknown"))?;
    if position.booking_user_id.is_some() {
        return Err(ApiError::PositionBooked);
    }
    if !position.event_is_in_atc_booking_period && !is_admin_booking {
        return Err(ApiError::EventNotInBookingTime);
    }
    let permission =
        position_repository::user_permission(services.db(), user_id, &position.position_kind_id)
            .await?
            .ok_or(ApiError::InsufficientAtcPermission)?;
    if !permission_satisfies(&permission, position.minimum_controller_state) {
        return Err(ApiError::InsufficientAtcPermission);
    }

    position_repository::create_booking(&mut transaction, &position, user_id).await?;
    let after = position_repository::find_by_event_and_id_in_transaction(
        &mut transaction,
        event_id,
        position_id,
        false,
    )
    .await?
    .ok_or(ApiError::not_found("event ATC position", "unknown"))?;
    create_position_audit_log(
        &mut transaction,
        &after,
        serde_json::to_value(position).map_err(|_| ApiError::Internal)?,
        serde_json::to_value(&after).map_err(|_| ApiError::Internal)?,
        operated_by,
    )
    .await?;
    transaction.commit().await?;

    EventAtcPositionBookingDto::try_from(after).map(Json)
}

#[utoipa::path(delete, path = "api/events/{event_id}/controllers/{position_id}/booking", tag = "Events", security(("oauth2" = [])), params(("event_id" = String, Path, description = "Event ULID"), ("position_id" = String, Path, description = "Position ULID")), responses((status = 200, description = "Successful response", body = EventAtcPositionBookingDto)))]
async fn cancel_position_booking(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((event_id, position_id)): Path<(String, String)>,
) -> Result<Json<EventAtcPositionBookingDto>, ApiError> {
    current_user.require_role(UserRole::Controller)?;
    let event_id = parse_ulid_uuid("event_id", &event_id)?;
    let position_id = parse_ulid_uuid("position_id", &position_id)?;
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let mut transaction = services.db().begin().await?;
    let position = position_repository::find_by_event_and_id_in_transaction(
        &mut transaction,
        event_id,
        position_id,
        true,
    )
    .await?
    .ok_or(ApiError::not_found("event ATC position", "unknown"))?;
    let Some(booking_user_id) = position.booking_user_id else {
        return Err(ApiError::PositionNotBooked);
    };
    if booking_user_id != current_user_id && !has_booking_admin_role(&current_user) {
        return Err(ApiError::PositionBookedByAnotherUser);
    }
    let dto = EventAtcPositionBookingDto::try_from(position.clone())?;
    position_repository::delete_booking(&mut transaction, position.id, position.atc_booking_id)
        .await?;
    let after = position_repository::find_by_event_and_id_in_transaction(
        &mut transaction,
        event_id,
        position_id,
        false,
    )
    .await?
    .ok_or(ApiError::not_found("event ATC position", "unknown"))?;
    create_position_audit_log(
        &mut transaction,
        &after,
        serde_json::to_value(position).map_err(|_| ApiError::Internal)?,
        serde_json::to_value(&after).map_err(|_| ApiError::Internal)?,
        current_user_id,
    )
    .await?;
    transaction.commit().await?;

    Ok(Json(dto))
}

async fn create_position_audit_log(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    position: &EventAtcPositionRecord,
    before: serde_json::Value,
    after: serde_json::Value,
    operated_by: Uuid,
) -> Result<(), ApiError> {
    audit_log_repository::create(
        transaction,
        AuditLog {
            entity: AuditLogEntity::EventAtcPosition(position.event_id, position.id),
            before,
            after,
            operated_by,
            created_at: Utc::now(),
        },
    )
    .await?;

    Ok(())
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

fn permission_satisfies(permission: &UserAtcPermissionRecord, minimum_state: i32) -> bool {
    let permission_rank = match permission.state.as_str() {
        "Student" => 0,
        "UnderMentor" => 1,
        "Solo" => {
            if permission
                .solo_expires_at
                .is_some_and(|expires_at| expires_at <= Utc::now())
            {
                return minimum_state <= 1;
            }
            2
        }
        "Certified" => 3,
        "Mentor" => return true,
        _ => return false,
    };
    permission_rank >= minimum_state
}
