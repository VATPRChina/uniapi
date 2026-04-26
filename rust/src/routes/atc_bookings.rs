use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::{
    adapter::database::atc_booking::{
        self as booking_repository, AtcBookingRecord, AtcBookingSave,
    },
    auth::CurrentUser,
    models::user_role::{UserRole, role_closure_from_strings},
    services::Services,
};

pub fn build_atc_booking_routes() -> Router<Services> {
    Router::new()
        .route("/", get(list_bookings).post(create_booking))
        .route("/mine", get(list_my_bookings))
        .route("/{id}", get(get_booking).put(update_booking).delete(delete_booking))
}

async fn list_bookings(
    State(services): State<Services>,
) -> Result<Json<Vec<AtcBookingDto>>, AtcBookingRouteError> {
    Ok(Json(
        booking_repository::list(services.db())
            .await
            .map_err(AtcBookingRouteError::Database)?
            .into_iter()
            .map(AtcBookingDto::from)
            .collect(),
    ))
}

async fn list_my_bookings(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<AtcBookingDto>>, AtcBookingRouteError> {
    let user_id = current_user
        .user_id
        .ok_or(AtcBookingRouteError::Unauthorized)?;
    Ok(Json(
        booking_repository::list_by_user(services.db(), user_id)
            .await
            .map_err(AtcBookingRouteError::Database)?
            .into_iter()
            .map(AtcBookingDto::from)
            .collect(),
    ))
}

async fn create_booking(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<AtcBookingSaveRequest>,
) -> Result<Json<AtcBookingDto>, AtcBookingRouteError> {
    require_role(&current_user, UserRole::Controller)?;
    let user_id = current_user
        .user_id
        .ok_or(AtcBookingRouteError::Unauthorized)?;
    let booking = booking_repository::create(services.db(), user_id, request.try_into()?)
        .await
        .map_err(AtcBookingRouteError::Database)?;

    Ok(Json(AtcBookingDto::from(booking)))
}

async fn get_booking(
    State(services): State<Services>,
    Path(id): Path<String>,
) -> Result<Json<AtcBookingDto>, AtcBookingRouteError> {
    let id = parse_ulid_uuid(&id)?;
    let booking = booking_repository::find_by_id(services.db(), id)
        .await
        .map_err(AtcBookingRouteError::Database)?
        .ok_or(AtcBookingRouteError::BookingNotFound)?;

    Ok(Json(AtcBookingDto::from(booking)))
}

async fn update_booking(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<AtcBookingSaveRequest>,
) -> Result<Json<AtcBookingDto>, AtcBookingRouteError> {
    require_role(&current_user, UserRole::Controller)?;
    let id = parse_ulid_uuid(&id)?;
    let user_id = current_user
        .user_id
        .ok_or(AtcBookingRouteError::Unauthorized)?;
    let booking = booking_repository::find_by_id(services.db(), id)
        .await
        .map_err(AtcBookingRouteError::Database)?
        .ok_or(AtcBookingRouteError::BookingNotFound)?;
    ensure_owner(&booking, user_id)?;
    ensure_not_event_position_booking(&services, id).await?;

    let booking = booking_repository::update(services.db(), id, request.try_into()?)
        .await
        .map_err(AtcBookingRouteError::Database)?
        .ok_or(AtcBookingRouteError::BookingNotFound)?;

    Ok(Json(AtcBookingDto::from(booking)))
}

async fn delete_booking(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<AtcBookingDto>, AtcBookingRouteError> {
    require_role(&current_user, UserRole::Controller)?;
    let id = parse_ulid_uuid(&id)?;
    let user_id = current_user
        .user_id
        .ok_or(AtcBookingRouteError::Unauthorized)?;
    let booking = booking_repository::find_by_id(services.db(), id)
        .await
        .map_err(AtcBookingRouteError::Database)?
        .ok_or(AtcBookingRouteError::BookingNotFound)?;
    ensure_owner(&booking, user_id)?;
    ensure_not_event_position_booking(&services, id).await?;

    let booking = booking_repository::delete(services.db(), id)
        .await
        .map_err(AtcBookingRouteError::Database)?
        .ok_or(AtcBookingRouteError::BookingNotFound)?;

    Ok(Json(AtcBookingDto::from(booking)))
}

async fn ensure_not_event_position_booking(
    services: &Services,
    booking_id: Uuid,
) -> Result<(), AtcBookingRouteError> {
    if booking_repository::has_event_position_booking(services.db(), booking_id)
        .await
        .map_err(AtcBookingRouteError::Database)?
    {
        return Err(AtcBookingRouteError::BookingIsEventPosition);
    }

    Ok(())
}

fn ensure_owner(
    booking: &AtcBookingRecord,
    user_id: Uuid,
) -> Result<(), AtcBookingRouteError> {
    if booking.user_id == user_id {
        Ok(())
    } else {
        Err(AtcBookingRouteError::BookingForbidden)
    }
}

fn require_role(
    current_user: &CurrentUser,
    role: UserRole,
) -> Result<(), AtcBookingRouteError> {
    if current_user.has_role(role) {
        Ok(())
    } else {
        Err(AtcBookingRouteError::Forbidden)
    }
}

fn parse_ulid_uuid(id: &str) -> Result<Uuid, AtcBookingRouteError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| AtcBookingRouteError::InvalidBookingId)
}

#[derive(Deserialize)]
struct AtcBookingSaveRequest {
    callsign: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

impl TryFrom<AtcBookingSaveRequest> for AtcBookingSave {
    type Error = AtcBookingRouteError;

    fn try_from(request: AtcBookingSaveRequest) -> Result<Self, Self::Error> {
        if request.start_time >= request.end_time {
            return Err(AtcBookingRouteError::StartMustBeBeforeEnd);
        }

        Ok(Self {
            callsign: request.callsign,
            start_at: request.start_time,
            end_at: request.end_time,
        })
    }
}

#[derive(Serialize)]
struct AtcBookingDto {
    id: String,
    user: UserDto,
    callsign: String,
    booked_at: DateTime<Utc>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

impl From<AtcBookingRecord> for AtcBookingDto {
    fn from(booking: AtcBookingRecord) -> Self {
        Self {
            id: Ulid::from(booking.id).to_string(),
            user: UserDto {
                id: Ulid::from(booking.user_id).to_string(),
                cid: booking.user_cid,
                full_name: String::new(),
                created_at: booking.user_created_at,
                updated_at: booking.user_updated_at,
                roles: roles_to_dto(&booking.user_roles),
                direct_roles: direct_roles_to_dto(&booking.user_roles),
                moodle_account: None,
            },
            callsign: booking.callsign,
            booked_at: booking.booked_at,
            start_time: booking.start_at,
            end_time: booking.end_at,
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

fn direct_roles_to_dto(roles: &[String]) -> Vec<String> {
    roles
        .iter()
        .filter_map(|role| role.parse::<UserRole>().ok())
        .map(|role| role.as_str().to_owned())
        .collect()
}

fn roles_to_dto(roles: &[String]) -> Vec<String> {
    let mut roles = role_closure_from_strings(roles.iter().map(String::as_str))
        .into_iter()
        .map(|role| role.as_str().to_owned())
        .collect::<Vec<_>>();
    roles.sort();
    roles
}

#[derive(Debug)]
enum AtcBookingRouteError {
    BookingForbidden,
    BookingIsEventPosition,
    BookingNotFound,
    Database(sqlx::Error),
    Forbidden,
    InvalidBookingId,
    StartMustBeBeforeEnd,
    Unauthorized,
}

impl IntoResponse for AtcBookingRouteError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AtcBookingRouteError::BookingForbidden => {
                (StatusCode::FORBIDDEN, "ATC booking forbidden".into())
            }
            AtcBookingRouteError::BookingIsEventPosition => (
                StatusCode::CONFLICT,
                "ATC booking is linked to an event position".into(),
            ),
            AtcBookingRouteError::BookingNotFound => {
                (StatusCode::NOT_FOUND, "ATC booking not found".into())
            }
            AtcBookingRouteError::Database(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            AtcBookingRouteError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            AtcBookingRouteError::InvalidBookingId => {
                (StatusCode::BAD_REQUEST, "invalid ATC booking id".into())
            }
            AtcBookingRouteError::StartMustBeBeforeEnd => (
                StatusCode::BAD_REQUEST,
                "start time must be before end time".into(),
            ),
            AtcBookingRouteError::Unauthorized => {
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
