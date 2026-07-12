use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::adapter::flight::{Flight, flights_from_vatsim};
use crate::auth::CurrentUser;
use crate::dto::*;
use crate::flight_plan::{parser, validator};
use crate::model::user_role::UserRole;
use crate::repository::auth::user::UserRepositoryExt;
use crate::routes::ApiError;
use crate::services::Services;

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

pub fn build_flight_routes() -> Router<Services> {
    Router::new()
        .route("/active", get(active_flights))
        .route("/by-callsign/{callsign}", get(flight_by_callsign))
        .route(
            "/by-callsign/{callsign}/warnings",
            get(warnings_by_callsign),
        )
        .route("/by-callsign/{callsign}/route", get(route_by_callsign))
        .route("/mine", get(my_flight))
        .route("/temporary/by-plan/warnings", get(temporary_warnings))
}

#[utoipa::path(get, path = "api/flights/active", tag = "Flights", responses((status = 200, description = "Successful response", body = Vec<FlightDto>)))]
async fn active_flights(
    State(services): State<Services>,
) -> Result<Json<Vec<FlightDto>>, ApiError> {
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
) -> Result<Json<FlightDto>, ApiError> {
    find_by_callsign(&services, &callsign)
        .await
        .map(FlightDto::from)
        .map(Json)
}

#[utoipa::path(get, path = "api/flights/by-callsign/{callsign}/warnings", tag = "Flights", params(("callsign" = String, Path, description = "Callsign")), responses((status = 200, description = "Successful response", body = Vec<validator::WarningMessage>)))]
async fn warnings_by_callsign(
    State(services): State<Services>,
    Path(callsign): Path<String>,
) -> Result<Json<Vec<validator::WarningMessage>>, ApiError> {
    let flight = find_by_callsign(&services, &callsign).await?;
    warnings_for_flight(&services, &flight).await
}

#[utoipa::path(get, path = "api/flights/by-callsign/{callsign}/route", tag = "Flights", params(("callsign" = String, Path, description = "Callsign")), responses((status = 200, description = "Successful response", body = Vec<FlightLeg>)))]
async fn route_by_callsign(
    State(services): State<Services>,
    Path(callsign): Path<String>,
) -> Result<Json<Vec<FlightLeg>>, ApiError> {
    let flight = find_by_callsign(&services, &callsign).await?;
    let route = route_string(&flight);
    let legs = parser::parse_route(services.navdata(), &route).await?;
    Ok(Json(legs.into_iter().map(FlightLeg::from).collect()))
}

#[utoipa::path(get, path = "api/flights/temporary/by-plan/warnings", tag = "Flights", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<validator::WarningMessage>)))]
async fn temporary_warnings(
    current_user: CurrentUser,
    State(services): State<Services>,
    Query(query): Query<TemporaryFlightQuery>,
) -> Result<Json<Vec<validator::WarningMessage>>, ApiError> {
    current_user.require_role(UserRole::ApiClient)?;
    warnings_for_flight(&services, &Flight::from(query)).await
}

#[utoipa::path(get, path = "api/flights/mine", tag = "Flights", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = FlightDto)))]
async fn my_flight(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<FlightDto>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let user = services
        .db()
        .find_user_detail_by_id(user_id)
        .await?
        .ok_or(ApiError::not_found("user", "unknown"))?;
    list_flights(&services)
        .await?
        .into_iter()
        .find(|flight| flight.cid == user.cid)
        .ok_or(ApiError::FlightNotFoundForCid)
        .map(FlightDto::from)
        .map(Json)
}

async fn list_flights(services: &Services) -> Result<Vec<Flight>, ApiError> {
    Ok(flights_from_vatsim(
        services.compat().get_online_data().await?,
    ))
}

async fn find_by_callsign(services: &Services, callsign: &str) -> Result<Flight, ApiError> {
    list_flights(services)
        .await?
        .into_iter()
        .find(|flight| flight.callsign.eq_ignore_ascii_case(callsign))
        .ok_or_else(|| ApiError::not_found("callsign", callsign))
}

fn route_string(flight: &Flight) -> String {
    format!(
        "{} {} {}",
        flight.departure, flight.raw_route, flight.arrival
    )
}

async fn warnings_for_flight(
    services: &Services,
    flight: &Flight,
) -> Result<Json<Vec<validator::WarningMessage>>, ApiError> {
    let route = route_string(flight);
    let legs = parser::parse_route(services.navdata(), &route).await?;
    let messages = validator::validate_route(services.navdata(), flight, &legs).await?;
    Ok(Json(messages))
}
