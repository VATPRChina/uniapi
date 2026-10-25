#![allow(deprecated)]

use std::collections::BTreeMap;
use std::time::Duration;

use axum::Json;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use tokio::time;

use crate::error::ApiError;
use crate::modules::flight::flight_plan::v1::validator;
use crate::modules::flight::service::FlightService;
use crate::services::Services;

const VALIDATION_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

#[utoipa::path(get, path = "api/flights/by-callsign/{callsign}/warnings", tag = "Flights", params(("callsign" = String, Path, description = "Callsign")), responses((status = 200, description = "Successful response", body = Vec<validator::WarningMessage>)))]
#[deprecated]
pub async fn warnings_by_callsign(
    State(services): State<Services>,
    Path(callsign): Path<String>,
) -> Result<Json<Vec<validator::WarningMessage>>, ApiError> {
    Ok(Json(
        services.flight().warnings_by_callsign(&callsign).await?,
    ))
}

#[deprecated]
pub async fn warnings_websocket(
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

#[deprecated]
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

#[deprecated]
async fn send_validation_snapshot(
    socket: &mut WebSocket,
    snapshot: &BTreeMap<String, Vec<validator::WarningMessage>>,
) -> Result<(), axum::Error> {
    let payload =
        serde_json::to_string(snapshot).expect("flight validation snapshot should serialize");
    socket.send(Message::Text(payload.into())).await
}
