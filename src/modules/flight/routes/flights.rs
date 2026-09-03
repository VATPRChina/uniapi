use std::collections::BTreeMap;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use tokio::time;

use crate::error::ApiError;
use crate::modules::flight::dto::{FlightDto, TemporaryFlightQuery};
use crate::modules::flight::flight_plan::validator;
use crate::modules::flight::models::Flight;
use crate::modules::flight::service::FlightService;
use crate::modules::user::middleware::CurrentUser;
use crate::modules::user::models::UserRole;
use crate::services::Services;

const VALIDATION_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    active_flights,
    flight_by_callsign,
    warnings_by_callsign,
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
        .route("/mine", get(my_flight))
        .route("/temporary/by-plan/warnings", get(temporary_warnings))
}

#[utoipa::path(get, path = "api/flights/active", tag = "Flights", responses((status = 200, description = "Successful response", body = Vec<FlightDto>)))]
async fn active_flights(
    State(services): State<Services>,
) -> Result<Json<Vec<FlightDto>>, ApiError> {
    Ok(Json(
        services
            .flight()
            .list()
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
    Ok(Json(
        services.flight().find_by_callsign(&callsign).await?.into(),
    ))
}

#[utoipa::path(get, path = "api/flights/by-callsign/{callsign}/warnings", tag = "Flights", params(("callsign" = String, Path, description = "Callsign")), responses((status = 200, description = "Successful response", body = Vec<validator::WarningMessage>)))]
async fn warnings_by_callsign(
    State(services): State<Services>,
    Path(callsign): Path<String>,
) -> Result<Json<Vec<validator::WarningMessage>>, ApiError> {
    Ok(Json(
        services.flight().warnings_by_callsign(&callsign).await?,
    ))
}

async fn warnings_websocket(
    State(services): State<Services>,
    websocket: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let flight = services.flight().clone();
    let initial_snapshot = flight.warnings_for_all().await?;

    Ok(
        websocket
            .on_upgrade(move |socket| stream_warning_changes(socket, flight, initial_snapshot)),
    )
}

#[utoipa::path(get, path = "api/flights/temporary/by-plan/warnings", tag = "Flights", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<validator::WarningMessage>)))]
async fn temporary_warnings(
    current_user: CurrentUser,
    State(services): State<Services>,
    Query(query): Query<TemporaryFlightQuery>,
) -> Result<Json<Vec<validator::WarningMessage>>, ApiError> {
    current_user.require_role(UserRole::ApiClient)?;
    Ok(Json(
        services.flight().warnings(&Flight::from(query)).await?,
    ))
}

#[utoipa::path(get, path = "api/flights/mine", tag = "Flights", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = FlightDto)))]
async fn my_flight(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<FlightDto>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    Ok(Json(services.flight().find_by_user(user_id).await?.into()))
}

async fn stream_warning_changes(
    mut socket: WebSocket,
    flight: FlightService,
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
                match flight.warnings_for_all().await {
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
