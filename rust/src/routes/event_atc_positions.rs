use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::{
    adapter::database::event_atc_position::{
        self as position_repository, EventAtcPositionRecord, EventAtcPositionSave,
        UserAtcPermissionRecord,
    },
    auth::CurrentUser,
    models::user_role::UserRole,
    services::Services,
};

const POSITION_KINDS: &[&str] = &["DEL", "GND", "TWR", "T2", "APP", "CTR", "FSS", "FMP"];

pub fn build_public_event_atc_position_routes() -> Router<Services> {
    Router::new().route("/{event_id}/controllers", get(list_positions))
}

pub fn build_protected_event_atc_position_routes() -> Router<Services> {
    Router::new()
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

async fn list_positions(
    State(services): State<Services>,
    Path(event_id): Path<String>,
) -> Result<Json<Vec<EventAtcPositionDto>>, EventAtcPositionError> {
    let event_id = parse_ulid_uuid(&event_id, EventAtcPositionError::InvalidEventId)?;
    Ok(Json(
        position_repository::list_by_event(services.db(), event_id)
            .await
            .map_err(EventAtcPositionError::Database)?
            .into_iter()
            .map(EventAtcPositionDto::from)
            .collect(),
    ))
}

async fn create_position(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(event_id): Path<String>,
    Json(request): Json<EventAtcPositionSaveRequest>,
) -> Result<Json<EventAtcPositionDto>, EventAtcPositionError> {
    require_edit_role(&current_user)?;
    let event_id = parse_ulid_uuid(&event_id, EventAtcPositionError::InvalidEventId)?;
    let position = position_repository::create(services.db(), event_id, request.try_into()?)
        .await
        .map_err(EventAtcPositionError::Database)?;

    Ok(Json(EventAtcPositionDto::from(position)))
}

async fn update_position(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((event_id, position_id)): Path<(String, String)>,
    Json(request): Json<EventAtcPositionSaveRequest>,
) -> Result<Json<EventAtcPositionDto>, EventAtcPositionError> {
    require_edit_role(&current_user)?;
    let event_id = parse_ulid_uuid(&event_id, EventAtcPositionError::InvalidEventId)?;
    let position_id = parse_ulid_uuid(&position_id, EventAtcPositionError::InvalidPositionId)?;
    let position =
        position_repository::update(services.db(), event_id, position_id, request.try_into()?)
            .await
            .map_err(EventAtcPositionError::Database)?
            .ok_or(EventAtcPositionError::PositionNotFound)?;

    Ok(Json(EventAtcPositionDto::from(position)))
}

async fn delete_position(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((event_id, position_id)): Path<(String, String)>,
) -> Result<StatusCode, EventAtcPositionError> {
    require_edit_role(&current_user)?;
    let event_id = parse_ulid_uuid(&event_id, EventAtcPositionError::InvalidEventId)?;
    let position_id = parse_ulid_uuid(&position_id, EventAtcPositionError::InvalidPositionId)?;
    if !position_repository::delete(services.db(), event_id, position_id)
        .await
        .map_err(EventAtcPositionError::Database)?
    {
        return Err(EventAtcPositionError::PositionNotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn book_position(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((event_id, position_id)): Path<(String, String)>,
    Json(request): Json<EventAtcPositionBookRequest>,
) -> Result<Json<EventAtcPositionBookingDto>, EventAtcPositionError> {
    require_role(&current_user, UserRole::Controller)?;
    let event_id = parse_ulid_uuid(&event_id, EventAtcPositionError::InvalidEventId)?;
    let position_id = parse_ulid_uuid(&position_id, EventAtcPositionError::InvalidPositionId)?;
    if request.user_id.is_some() && !has_booking_admin_role(&current_user) {
        return Err(EventAtcPositionError::Forbidden);
    }
    let user_id = match request.user_id.as_deref() {
        Some(user_id) => parse_ulid_uuid(user_id, EventAtcPositionError::InvalidUserId)?,
        None => current_user
            .user_id
            .ok_or(EventAtcPositionError::Unauthorized)?,
    };
    let is_admin_booking = request.user_id.is_some();
    let position = load_position(&services, event_id, position_id).await?;
    if position.booking_user_id.is_some() {
        return Err(EventAtcPositionError::PositionBooked);
    }
    if !position.event_is_in_atc_booking_period && !is_admin_booking {
        return Err(EventAtcPositionError::EventNotInBookingTime);
    }
    let permission =
        position_repository::user_permission(services.db(), user_id, &position.position_kind_id)
            .await
            .map_err(EventAtcPositionError::Database)?
            .ok_or(EventAtcPositionError::InsufficientAtcPermission)?;
    if !permission_satisfies(&permission, position.minimum_controller_state) {
        return Err(EventAtcPositionError::InsufficientAtcPermission);
    }

    let mut transaction = services
        .db()
        .begin()
        .await
        .map_err(EventAtcPositionError::Database)?;
    position_repository::create_booking(&mut transaction, &position, user_id)
        .await
        .map_err(EventAtcPositionError::Database)?;
    transaction
        .commit()
        .await
        .map_err(EventAtcPositionError::Database)?;
    let position = load_position(&services, event_id, position_id).await?;

    EventAtcPositionBookingDto::try_from(position).map(Json)
}

async fn cancel_position_booking(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path((event_id, position_id)): Path<(String, String)>,
) -> Result<Json<EventAtcPositionBookingDto>, EventAtcPositionError> {
    require_role(&current_user, UserRole::Controller)?;
    let event_id = parse_ulid_uuid(&event_id, EventAtcPositionError::InvalidEventId)?;
    let position_id = parse_ulid_uuid(&position_id, EventAtcPositionError::InvalidPositionId)?;
    let position = load_position(&services, event_id, position_id).await?;
    let Some(booking_user_id) = position.booking_user_id else {
        return Err(EventAtcPositionError::PositionNotBooked);
    };
    let current_user_id = current_user
        .user_id
        .ok_or(EventAtcPositionError::Unauthorized)?;
    if booking_user_id != current_user_id && !has_booking_admin_role(&current_user) {
        return Err(EventAtcPositionError::PositionBookedByAnotherUser);
    }
    let dto = EventAtcPositionBookingDto::try_from(position.clone())?;
    let mut transaction = services
        .db()
        .begin()
        .await
        .map_err(EventAtcPositionError::Database)?;
    position_repository::delete_booking(&mut transaction, position.id, position.atc_booking_id)
        .await
        .map_err(EventAtcPositionError::Database)?;
    transaction
        .commit()
        .await
        .map_err(EventAtcPositionError::Database)?;

    Ok(Json(dto))
}

async fn load_position(
    services: &Services,
    event_id: Uuid,
    position_id: Uuid,
) -> Result<EventAtcPositionRecord, EventAtcPositionError> {
    position_repository::find_by_event_and_id(services.db(), event_id, position_id)
        .await
        .map_err(EventAtcPositionError::Database)?
        .ok_or(EventAtcPositionError::PositionNotFound)
}

fn require_edit_role(current_user: &CurrentUser) -> Result<(), EventAtcPositionError> {
    if current_user.has_role(UserRole::EventCoordinator)
        || current_user.has_role(UserRole::ControllerTrainingDirectorAssistant)
        || current_user.has_role(UserRole::OperationDirectorAssistant)
    {
        Ok(())
    } else {
        Err(EventAtcPositionError::Forbidden)
    }
}

fn has_booking_admin_role(current_user: &CurrentUser) -> bool {
    current_user.has_role(UserRole::EventCoordinator)
        || current_user.has_role(UserRole::ControllerTrainingDirectorAssistant)
        || current_user.has_role(UserRole::ControllerTrainingMentor)
}

fn require_role(current_user: &CurrentUser, role: UserRole) -> Result<(), EventAtcPositionError> {
    if current_user.has_role(role) {
        Ok(())
    } else {
        Err(EventAtcPositionError::Forbidden)
    }
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

fn parse_ulid_uuid(id: &str, error: EventAtcPositionError) -> Result<Uuid, EventAtcPositionError> {
    id.parse::<Ulid>().map(Uuid::from).map_err(|_| error)
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum UserControllerState {
    Student,
    UnderMentor,
    Solo,
    Certified,
    Mentor,
}

impl UserControllerState {
    fn to_db(self) -> i32 {
        match self {
            Self::Student => 0,
            Self::UnderMentor => 1,
            Self::Solo => 2,
            Self::Certified => 3,
            Self::Mentor => 4,
        }
    }

    fn from_db(value: i32) -> Self {
        match value {
            1 => Self::UnderMentor,
            2 => Self::Solo,
            3 => Self::Certified,
            4 => Self::Mentor,
            _ => Self::Student,
        }
    }
}

#[derive(Deserialize)]
struct EventAtcPositionSaveRequest {
    callsign: String,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    remarks: Option<String>,
    position_kind_id: String,
    minimum_controller_state: UserControllerState,
}

impl TryFrom<EventAtcPositionSaveRequest> for EventAtcPositionSave {
    type Error = EventAtcPositionError;

    fn try_from(request: EventAtcPositionSaveRequest) -> Result<Self, Self::Error> {
        if !POSITION_KINDS.contains(&request.position_kind_id.as_str()) {
            return Err(EventAtcPositionError::InvalidPositionKind);
        }

        Ok(Self {
            callsign: request.callsign,
            start_at: request.start_at,
            end_at: request.end_at,
            remarks: request.remarks,
            position_kind_id: request.position_kind_id,
            minimum_controller_state: request.minimum_controller_state.to_db(),
        })
    }
}

#[derive(Deserialize)]
struct EventAtcPositionBookRequest {
    user_id: Option<String>,
}

#[derive(Serialize)]
struct EventAtcPositionDto {
    id: String,
    event: EventDto,
    callsign: String,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    remarks: Option<String>,
    position_kind_id: String,
    minimum_controller_state: UserControllerState,
    booking: Option<EventAtcPositionBookingDto>,
}

impl From<EventAtcPositionRecord> for EventAtcPositionDto {
    fn from(position: EventAtcPositionRecord) -> Self {
        Self {
            id: Ulid::from(position.id).to_string(),
            event: EventDto::from(&position),
            callsign: position.callsign.clone(),
            start_at: position.start_at,
            end_at: position.end_at,
            remarks: position.remarks.clone(),
            position_kind_id: position.position_kind_id.clone(),
            minimum_controller_state: UserControllerState::from_db(
                position.minimum_controller_state,
            ),
            booking: EventAtcPositionBookingDto::try_from(position).ok(),
        }
    }
}

#[derive(Serialize)]
struct EventAtcPositionBookingDto {
    user_id: String,
    user: UserDto,
    booked_at: DateTime<Utc>,
}

impl TryFrom<EventAtcPositionRecord> for EventAtcPositionBookingDto {
    type Error = EventAtcPositionError;

    fn try_from(position: EventAtcPositionRecord) -> Result<Self, Self::Error> {
        let user_id = position
            .booking_user_id
            .ok_or(EventAtcPositionError::PositionNotBooked)?;
        Ok(Self {
            user_id: Ulid::from(user_id).to_string(),
            user: UserDto {
                id: Ulid::from(user_id).to_string(),
                cid: position.booking_user_cid.unwrap_or_default(),
                full_name: position.booking_user_full_name.unwrap_or_default(),
                created_at: position
                    .booking_user_created_at
                    .ok_or(EventAtcPositionError::PositionNotBooked)?,
                updated_at: position
                    .booking_user_updated_at
                    .ok_or(EventAtcPositionError::PositionNotBooked)?,
                roles: position.booking_user_roles.clone().unwrap_or_default(),
                direct_roles: position.booking_user_roles.unwrap_or_default(),
                moodle_account: None,
            },
            booked_at: position
                .booking_created_at
                .ok_or(EventAtcPositionError::PositionNotBooked)?,
        })
    }
}

#[derive(Serialize)]
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

impl From<&EventAtcPositionRecord> for EventDto {
    fn from(position: &EventAtcPositionRecord) -> Self {
        Self {
            id: Ulid::from(position.event_id).to_string(),
            created_at: position.event_created_at,
            updated_at: position.event_updated_at,
            title: position.event_title.clone(),
            title_en: position.event_title_en.clone(),
            start_at: position.event_start_at,
            end_at: position.event_end_at,
            start_booking_at: position.event_start_booking_at,
            end_booking_at: position.event_end_booking_at,
            start_atc_booking_at: position.event_start_atc_booking_at,
            image_url: position.event_image_url.clone(),
            community_link: position.event_community_link.clone(),
            vatsim_link: position.event_vatsim_link.clone(),
            description: position.event_description.clone(),
        }
    }
}

#[derive(Serialize)]
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

#[derive(Debug)]
enum EventAtcPositionError {
    Database(sqlx::Error),
    EventNotInBookingTime,
    Forbidden,
    InsufficientAtcPermission,
    InvalidEventId,
    InvalidPositionId,
    InvalidPositionKind,
    InvalidUserId,
    PositionBooked,
    PositionBookedByAnotherUser,
    PositionNotBooked,
    PositionNotFound,
    Unauthorized,
}

impl IntoResponse for EventAtcPositionError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            EventAtcPositionError::Database(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            EventAtcPositionError::EventNotInBookingTime => {
                (StatusCode::FORBIDDEN, "event not in booking time".into())
            }
            EventAtcPositionError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            EventAtcPositionError::InsufficientAtcPermission => {
                (StatusCode::FORBIDDEN, "insufficient ATC permission".into())
            }
            EventAtcPositionError::InvalidEventId => {
                (StatusCode::BAD_REQUEST, "invalid event id".into())
            }
            EventAtcPositionError::InvalidPositionId => {
                (StatusCode::BAD_REQUEST, "invalid position id".into())
            }
            EventAtcPositionError::InvalidPositionKind => {
                (StatusCode::BAD_REQUEST, "invalid ATC position kind".into())
            }
            EventAtcPositionError::InvalidUserId => {
                (StatusCode::BAD_REQUEST, "invalid user id".into())
            }
            EventAtcPositionError::PositionBooked => {
                (StatusCode::CONFLICT, "event position booked".into())
            }
            EventAtcPositionError::PositionBookedByAnotherUser => (
                StatusCode::FORBIDDEN,
                "event position booked by another user".into(),
            ),
            EventAtcPositionError::PositionNotBooked => {
                (StatusCode::NOT_FOUND, "event position not booked".into())
            }
            EventAtcPositionError::PositionNotFound => {
                (StatusCode::NOT_FOUND, "event ATC position not found".into())
            }
            EventAtcPositionError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "unauthorized".into())
            }
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}
