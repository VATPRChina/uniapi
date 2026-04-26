use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    adapter::{
        compat::CompatClientError,
        database::user::{self as user_repository},
        flight::{Flight, flights_from_vatsim},
    },
    auth::CurrentUser,
    models::user_role::UserRole,
    services::Services,
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    active_flights,
    flight_by_callsign,
    warnings_by_callsign,
    route_by_callsign,
    my_flight,
    temporary_warnings
))]
pub(crate) struct ApiDoc;

pub fn build_public_flight_routes() -> Router<Services> {
    Router::new()
        .route("/active", get(active_flights))
        .route("/by-callsign/{callsign}", get(flight_by_callsign))
        .route(
            "/by-callsign/{callsign}/warnings",
            get(warnings_by_callsign),
        )
        .route("/by-callsign/{callsign}/route", get(route_by_callsign))
}

pub fn build_protected_flight_routes() -> Router<Services> {
    Router::new()
        .route("/mine", get(my_flight))
        .route("/temporary/by-plan/warnings", get(temporary_warnings))
}

#[utoipa::path(get, path = "api/flights/active", tag = "Flights", responses((status = 200, description = "Successful response", body = Vec<FlightDto>)))]
async fn active_flights(
    State(services): State<Services>,
) -> Result<Json<Vec<FlightDto>>, FlightRouteError> {
    Ok(Json(
        list_flights(&services)
            .await?
            .into_iter()
            .map(FlightDto::from)
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/flights/by-callsign/{callsign}", tag = "Flights", params(("callsign" = String, Path, description = "Callsign")), responses((status = 200, description = "Successful response", body = FlightDto)))]
async fn flight_by_callsign(
    State(services): State<Services>,
    Path(callsign): Path<String>,
) -> Result<Json<FlightDto>, FlightRouteError> {
    find_by_callsign(&services, &callsign)
        .await
        .map(FlightDto::from)
        .map(Json)
}

#[utoipa::path(get, path = "api/flights/by-callsign/{callsign}/warnings", tag = "Flights", params(("callsign" = String, Path, description = "Callsign")), responses((status = 200, description = "Successful response", body = Vec<WarningMessage>)))]
async fn warnings_by_callsign(
    State(services): State<Services>,
    Path(callsign): Path<String>,
) -> Result<Response, FlightRouteError> {
    find_by_callsign(&services, &callsign).await?;
    Err(FlightRouteError::RouteParsingNotImplemented)
}

#[utoipa::path(get, path = "api/flights/by-callsign/{callsign}/route", tag = "Flights", params(("callsign" = String, Path, description = "Callsign")), responses((status = 200, description = "Successful response", body = Vec<FlightLeg>)))]
async fn route_by_callsign(
    State(services): State<Services>,
    Path(callsign): Path<String>,
) -> Result<Response, FlightRouteError> {
    find_by_callsign(&services, &callsign).await?;
    Err(FlightRouteError::RouteParsingNotImplemented)
}

#[utoipa::path(get, path = "api/flights/temporary/by-plan/warnings", tag = "Flights", security(("bearerAuth" = [])), responses((status = 200, description = "Successful response", body = Vec<WarningMessage>)))]
async fn temporary_warnings(
    current_user: CurrentUser,
    Query(_query): Query<TemporaryFlightQuery>,
) -> Result<Response, FlightRouteError> {
    require_role(&current_user, UserRole::ApiClient)?;
    Err(FlightRouteError::RouteParsingNotImplemented)
}

#[utoipa::path(get, path = "api/flights/mine", tag = "Flights", security(("bearerAuth" = [])), responses((status = 200, description = "Successful response", body = FlightDto)))]
async fn my_flight(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<FlightDto>, FlightRouteError> {
    let user_id = current_user.user_id.ok_or(FlightRouteError::Unauthorized)?;
    let user = user_repository::find_detail_by_id(services.db(), user_id)
        .await
        .map_err(FlightRouteError::Database)?
        .ok_or(FlightRouteError::UserNotFound)?;
    list_flights(&services)
        .await?
        .into_iter()
        .find(|flight| flight.cid == user.cid)
        .ok_or(FlightRouteError::FlightNotFoundForCid)
        .map(FlightDto::from)
        .map(Json)
}

async fn list_flights(services: &Services) -> Result<Vec<Flight>, FlightRouteError> {
    Ok(flights_from_vatsim(
        services
            .compat()
            .get_online_data()
            .await
            .map_err(FlightRouteError::Compat)?,
    ))
}

async fn find_by_callsign(services: &Services, callsign: &str) -> Result<Flight, FlightRouteError> {
    list_flights(services)
        .await?
        .into_iter()
        .find(|flight| flight.callsign.eq_ignore_ascii_case(callsign))
        .ok_or(FlightRouteError::CallsignNotFound)
}

fn require_role(current_user: &CurrentUser, role: UserRole) -> Result<(), FlightRouteError> {
    if current_user.has_role(role) {
        Ok(())
    } else {
        Err(FlightRouteError::Forbidden)
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
#[allow(dead_code)]
struct TemporaryFlightQuery {
    departure: String,
    arrival: String,
    #[serde(default)]
    aircraft: String,
    #[serde(default)]
    equipment: String,
    #[serde(default)]
    navigation_performance: String,
    #[serde(default)]
    transponder: String,
    #[serde(default)]
    raw_route: String,
    #[serde(default)]
    cruising_level: i64,
}

#[derive(Serialize, utoipa::ToSchema)]
struct FlightDto {
    id: String,
    cid: String,
    callsign: String,
    last_observed_at: DateTime<Utc>,
    departure: String,
    arrival: String,
    equipment: String,
    navigation_performance: String,
    transponder: String,
    raw_route: String,
    aircraft: String,
    altitude: i64,
    cruising_level: i64,
}

impl From<Flight> for FlightDto {
    fn from(flight: Flight) -> Self {
        Self {
            id: flight.id.to_string(),
            cid: flight.cid,
            callsign: flight.callsign,
            last_observed_at: flight.last_observed_at,
            departure: flight.departure,
            arrival: flight.arrival,
            equipment: flight.equipment,
            navigation_performance: flight.navigation_performance,
            transponder: flight.transponder,
            raw_route: flight.raw_route,
            aircraft: flight.aircraft,
            altitude: flight.altitude,
            cruising_level: flight.cruising_level,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct FlightLeg {
    from: FlightFix,
    to: FlightFix,
    leg_identifier: String,
}

#[derive(Serialize, utoipa::ToSchema)]
struct FlightFix {
    identifier: String,
}

#[derive(Serialize, utoipa::ToSchema)]
struct WarningMessage {
    level: Option<String>,
    message: Option<String>,
}

#[derive(Debug)]
enum FlightRouteError {
    CallsignNotFound,
    Compat(CompatClientError),
    Database(sqlx::Error),
    FlightNotFoundForCid,
    Forbidden,
    RouteParsingNotImplemented,
    Unauthorized,
    UserNotFound,
}

impl IntoResponse for FlightRouteError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            FlightRouteError::CallsignNotFound => {
                (StatusCode::NOT_FOUND, "callsign not found".into())
            }
            FlightRouteError::Compat(error) => (StatusCode::BAD_GATEWAY, error.to_string()),
            FlightRouteError::Database(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            FlightRouteError::FlightNotFoundForCid => {
                (StatusCode::NOT_FOUND, "flight not found for cid".into())
            }
            FlightRouteError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            FlightRouteError::RouteParsingNotImplemented => (
                StatusCode::NOT_IMPLEMENTED,
                "flight route parsing is not implemented in Rust yet".into(),
            ),
            FlightRouteError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
            FlightRouteError::UserNotFound => (StatusCode::NOT_FOUND, "user not found".into()),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct ErrorResponse {
    message: String,
}
