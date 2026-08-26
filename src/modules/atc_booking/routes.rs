use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use ulid::Ulid;

use crate::error::ApiError;
use crate::modules::user::middleware::CurrentUser;
use crate::modules::user::models::UserRole;
use crate::services::Services;

use super::dto::{AtcBookingDto, AtcBookingSaveRequest};

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_upcoming, list_mine_upcoming, create, update, delete))]
pub(crate) struct ApiDoc;

pub fn build_atc_booking_routes() -> Router<Services> {
    Router::new()
        .route("/upcoming", get(list_upcoming))
        .route("/mine/upcoming", get(list_mine_upcoming))
        .route("/", axum::routing::put(create))
        .route("/{id}", axum::routing::put(update).delete(delete))
}

#[utoipa::path(get, path = "api/atc/bookings/upcoming", tag = "ATC", responses((status = 200, description = "Upcoming ATC bookings", body = Vec<AtcBookingDto>)))]
async fn list_upcoming(
    State(services): State<Services>,
) -> Result<Json<Vec<AtcBookingDto>>, ApiError> {
    Ok(Json(
        services
            .atc_booking()
            .list_upcoming()
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/atc/bookings/mine/upcoming", tag = "ATC", security(("oauth2" = [])), responses((status = 200, description = "Current user's upcoming ATC bookings", body = Vec<AtcBookingDto>)))]
async fn list_mine_upcoming(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<AtcBookingDto>>, ApiError> {
    current_user.require_role(UserRole::Controller)?;
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    Ok(Json(
        services
            .atc_booking()
            .list_mine_upcoming(user_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

#[utoipa::path(put, path = "api/atc/bookings", tag = "ATC", security(("oauth2" = [])), request_body = AtcBookingSaveRequest, responses((status = 200, description = "ATC booking created", body = AtcBookingDto)))]
async fn create(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<AtcBookingSaveRequest>,
) -> Result<Json<AtcBookingDto>, ApiError> {
    current_user.require_role(UserRole::Controller)?;
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    Ok(Json(
        services
            .atc_booking()
            .create(user_id, request.try_into()?)
            .await?
            .into(),
    ))
}

#[utoipa::path(put, path = "api/atc/bookings/{id}", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "ATC booking ULID")), request_body = AtcBookingSaveRequest, responses((status = 200, description = "ATC booking updated", body = AtcBookingDto)))]
async fn update(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<AtcBookingSaveRequest>,
) -> Result<Json<AtcBookingDto>, ApiError> {
    current_user.require_role(UserRole::Controller)?;
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    Ok(Json(
        services
            .atc_booking()
            .update(id.parse::<Ulid>()?.into(), user_id, request.try_into()?)
            .await?
            .into(),
    ))
}

#[utoipa::path(delete, path = "api/atc/bookings/{id}", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "ATC booking ULID")), responses((status = 200, description = "ATC booking cancelled", body = AtcBookingDto)))]
async fn delete(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<AtcBookingDto>, ApiError> {
    current_user.require_role(UserRole::Controller)?;
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    Ok(Json(
        services
            .atc_booking()
            .delete(id.parse::<Ulid>()?.into(), user_id)
            .await?
            .into(),
    ))
}
