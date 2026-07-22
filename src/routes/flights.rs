use std::collections::BTreeMap;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use tokio::time;

use crate::adapter::flight::{Flight, flights_from_vatsim};
use crate::auth::CurrentUser;
use crate::dto::*;
use crate::flight_plan::{parser, validator};
use crate::model::user_role::UserRole;
use crate::repository::auth::user::UserRepositoryExt;
use crate::routes::ApiError;
use crate::services::Services;

const VALIDATION_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

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
        .route("/warnings/streaming", get(warnings_websocket))
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
    warnings_for_flight(&services, &flight).await.map(Json)
}

async fn warnings_websocket(
    State(services): State<Services>,
    websocket: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let initial_snapshot = warnings_for_all_flights(&services).await?;

    Ok(websocket
        .on_upgrade(move |socket| stream_warning_changes(socket, services, initial_snapshot)))
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
    warnings_for_flight(&services, &Flight::from(query))
        .await
        .map(Json)
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
) -> Result<Vec<validator::WarningMessage>, ApiError> {
    let route = route_string(flight);
    let legs = parser::parse_route(services.navdata(), &route).await?;
    let messages = validator::validate_route(services.navdata(), flight, &legs).await?;
    Ok(messages)
}

async fn warnings_for_all_flights(
    services: &Services,
) -> Result<BTreeMap<String, Vec<validator::WarningMessage>>, ApiError> {
    let validations = futures::future::join_all(list_flights(services).await?.into_iter().map(
        |flight| async move {
            let callsign = flight.callsign.clone();
            warnings_for_flight(services, &flight)
                .await
                .map(|warnings| (callsign, warnings))
        },
    ))
    .await;

    validations.into_iter().collect()
}

async fn stream_warning_changes(
    mut socket: WebSocket,
    services: Services,
    mut snapshot: BTreeMap<String, Vec<validator::WarningMessage>>,
) {
    if send_validation_snapshot(&mut socket, &snapshot)
        .await
        .is_err()
    {
        return;
    }

    let mut refresh = time::interval(VALIDATION_REFRESH_INTERVAL);
    refresh.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    refresh.tick().await;

    loop {
        tokio::select! {
            message = socket.recv() => match message {
                Some(Ok(Message::Close(_))) | None => return,
                Some(Err(error)) => {
                    tracing::debug!(%error, "flight validation websocket closed");
                    return;
                }
                Some(Ok(_)) => {}
            },
            _ = refresh.tick() => {
                match warnings_for_all_flights(&services).await {
                    Ok(updated) if updated != snapshot => {
                        if send_validation_snapshot(&mut socket, &updated).await.is_err() {
                            return;
                        }
                        snapshot = updated;
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(%error, "failed to refresh flight validation websocket");
                    }
                }
            }
        }
    }
}

async fn send_validation_snapshot(
    socket: &mut WebSocket,
    snapshot: &BTreeMap<String, Vec<validator::WarningMessage>>,
) -> Result<(), axum::Error> {
    let payload =
        serde_json::to_string(snapshot).expect("flight validation snapshot should serialize");
    socket.send(Message::Text(payload.into())).await
}
