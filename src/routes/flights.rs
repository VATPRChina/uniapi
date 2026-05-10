use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::adapter::flight::{Flight, flights_from_vatsim};
use crate::auth::CurrentUser;
use crate::flight_plan::{Leg, parser, validator};
use crate::model::user_role::UserRole;
use crate::repository::auth::user as user_repository;
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
    let legs = parser::parse_route(services.db(), &route).await?;
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
    let user = user_repository::find_detail_by_id(services.db(), user_id)
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
    let legs = parser::parse_route(services.db(), &route).await?;
    let messages = validator::validate_route(services.db(), flight, &legs).await?;
    Ok(Json(messages))
}

impl From<TemporaryFlightQuery> for Flight {
    fn from(query: TemporaryFlightQuery) -> Self {
        Self {
            id: ulid::Ulid::new(),
            cid: String::new(),
            callsign: String::new(),
            last_observed_at: Utc::now(),
            departure: query.departure,
            arrival: query.arrival,
            equipment: query.equipment,
            navigation_performance: query.navigation_performance,
            transponder: query.transponder,
            raw_route: query.raw_route,
            aircraft: query.aircraft,
            altitude: 0,
            cruising_level: query.cruising_level,
        }
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

impl From<Leg> for FlightLeg {
    fn from(leg: Leg) -> Self {
        let leg_identifier = match &leg {
            Leg::Airway(airway) => airway.identifier.clone(),
            Leg::Direct(_) => "DCT".to_owned(),
        };
        Self {
            from: FlightFix::from(leg.from()),
            to: FlightFix::from(leg.to()),
            leg_identifier,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct FlightFix {
    identifier: String,
}

impl From<&crate::flight_plan::Fix> for FlightFix {
    fn from(fix: &crate::flight_plan::Fix) -> Self {
        Self {
            identifier: fix.name(),
        }
    }
}
