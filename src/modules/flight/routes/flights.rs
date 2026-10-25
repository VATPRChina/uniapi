use std::collections::BTreeMap;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use futures::{StreamExt, TryStreamExt};
use tokio::time;

use crate::error::ApiError;
use crate::modules::flight::dto::{
    FlightDto, FlightRouteDto, FlightValidationDto, TemporaryFlightQuery,
};
use crate::modules::flight::models::Flight;
#[allow(deprecated)]
use crate::modules::flight::routes::flights_v1::{warnings_by_callsign, warnings_websocket};
use crate::modules::flight::service::{FlightService, FlightServiceError};
use crate::modules::user::middleware::CurrentUser;
use crate::modules::user::models::UserRole;
use crate::services::Services;

const VALIDATION_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

type ValidationSnapshot = BTreeMap<String, FlightValidationDto>;

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    my_flight,
    my_flight_validation,
    temporary_flight,
    temporary_flight_validation,
    flights_by_departure,
    flights_by_arrival,
    flight_by_callsign,
    flight_validation_by_callsign,
    flight_route_by_callsign,
    departure_validations_websocket
))]
pub(crate) struct ApiDoc;

pub fn build_flight_routes() -> Router<Services> {
    Router::new()
        .route(
            "/warnings/streaming",
            #[allow(deprecated)]
            get(warnings_websocket),
        )
        .route("/mine", get(my_flight))
        .route("/mine/validation", get(my_flight_validation))
        .route("/temporary/by-plan", get(temporary_flight))
        .route(
            "/temporary/by-plan/validation",
            get(temporary_flight_validation),
        )
        .route("/by-departure/{icao}", get(flights_by_departure))
        .route("/by-arrival/{icao}", get(flights_by_arrival))
        .route("/by-callsign/{callsign}", get(flight_by_callsign))
        .route(
            "/by-callsign/{callsign}/route",
            get(flight_route_by_callsign),
        )
        .route(
            "/by-callsign/{callsign}/warnings",
            #[allow(deprecated)]
            get(warnings_by_callsign),
        )
        .route(
            "/by-callsign/{callsign}/validation",
            get(flight_validation_by_callsign),
        )
        .route(
            "/by-departure/{icao}/validations.ws",
            get(departure_validations_websocket),
        )
}

#[utoipa::path(get, path = "api/flights/mine", tag = "Flights", security(("oauth2" = [])), responses(
    (status = 200, description = "Current user's flight", body = FlightDto),
    (status = 401, description = "Authentication required"),
    (status = 404, description = "Flight not found")
))]
async fn my_flight(
    State(flight): State<FlightService>,
    current_user: CurrentUser,
) -> Result<Json<FlightDto>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    Ok(Json(flight.find_by_user(user_id).await?.into()))
}

#[utoipa::path(get, path = "api/flights/mine/validation", tag = "Flights", security(("oauth2" = [])), responses(
    (status = 200, description = "Current user's flight validation", body = FlightValidationDto),
    (status = 401, description = "Authentication required"),
    (status = 404, description = "Flight not found")
))]
async fn my_flight_validation(
    State(service): State<FlightService>,
    current_user: CurrentUser,
) -> Result<Json<FlightValidationDto>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let flight = service.find_by_user(user_id).await?;
    Ok(Json(service.validate(&flight).await?.into()))
}

#[utoipa::path(get, path = "api/flights/temporary/by-plan", tag = "Flights", security(("oauth2" = [])),
    params(TemporaryFlightQuery), responses(
        (status = 200, description = "Temporary flight constructed from the supplied plan", body = FlightDto),
        (status = 400, description = "Invalid flight plan query"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "API client role required")
    )
)]
async fn temporary_flight(
    current_user: CurrentUser,
    Query(query): Query<TemporaryFlightQuery>,
) -> Result<Json<FlightDto>, ApiError> {
    current_user.require_role(UserRole::ApiClient)?;
    Ok(Json(temporary_flight_from_query(query)?.into()))
}

#[utoipa::path(get, path = "api/flights/temporary/by-plan/validation", tag = "Flights", security(("oauth2" = [])),
    params(TemporaryFlightQuery), responses(
        (status = 200, description = "Validation of the supplied flight plan", body = FlightValidationDto),
        (status = 400, description = "Invalid flight plan query or route"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "API client role required")
    )
)]
async fn temporary_flight_validation(
    current_user: CurrentUser,
    State(service): State<FlightService>,
    Query(query): Query<TemporaryFlightQuery>,
) -> Result<Json<FlightValidationDto>, ApiError> {
    current_user.require_role(UserRole::ApiClient)?;
    let flight = temporary_flight_from_query(query)?;
    Ok(Json(service.validate(&flight).await?.into()))
}

fn temporary_flight_from_query(query: TemporaryFlightQuery) -> Result<Flight, ApiError> {
    if !matches!(
        query.flight_rules.trim().to_ascii_uppercase().as_str(),
        "" | "I" | "V"
    ) {
        return Err(ApiError::bad_request("flight_rules", "expected I or V"));
    }
    if !matches!(
        query.wake_category.trim().to_ascii_uppercase().as_str(),
        "" | "J" | "H" | "M" | "L"
    ) {
        return Err(ApiError::bad_request(
            "wake_category",
            "expected J, H, M or L",
        ));
    }
    Ok(query.into())
}

#[utoipa::path(get, path = "api/flights/by-departure/{icao}", tag = "Flights",
    params(("icao" = String, Path, description = "Departure airport ICAO code")),
    responses((status = 200, description = "Flights departing from the airport", body = Vec<FlightDto>))
)]
async fn flights_by_departure(
    State(service): State<FlightService>,
    Path(icao): Path<String>,
) -> Result<Json<Vec<FlightDto>>, ApiError> {
    Ok(Json(
        service
            .by_departure(&icao)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/flights/by-arrival/{icao}", tag = "Flights",
    params(("icao" = String, Path, description = "Arrival airport ICAO code")),
    responses((status = 200, description = "Flights arriving at the airport", body = Vec<FlightDto>))
)]
async fn flights_by_arrival(
    State(service): State<FlightService>,
    Path(icao): Path<String>,
) -> Result<Json<Vec<FlightDto>>, ApiError> {
    Ok(Json(
        service
            .by_arrival(&icao)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/flights/by-callsign/{callsign}", tag = "Flights",
    params(("callsign" = String, Path, description = "Callsign")), responses(
        (status = 200, description = "Flight snapshot", body = FlightDto),
        (status = 404, description = "Flight not found")
    )
)]
async fn flight_by_callsign(
    State(service): State<FlightService>,
    Path(callsign): Path<String>,
) -> Result<Json<FlightDto>, ApiError> {
    Ok(Json(service.find_by_callsign(&callsign).await?.into()))
}

#[utoipa::path(get, path = "api/flights/by-callsign/{callsign}/validation", tag = "Flights",
    params(("callsign" = String, Path, description = "Callsign")), responses(
        (status = 200, description = "Flight validation", body = FlightValidationDto),
        (status = 400, description = "Invalid route"),
        (status = 404, description = "Flight not found")
    )
)]
async fn flight_validation_by_callsign(
    State(service): State<FlightService>,
    Path(callsign): Path<String>,
) -> Result<Json<FlightValidationDto>, ApiError> {
    let flight = service.find_by_callsign(&callsign).await?;
    Ok(Json(service.validate(&flight).await?.into()))
}

#[utoipa::path(get, path = "api/flights/by-callsign/{callsign}/route", tag = "Flights",
    security(("oauth2" = [])),
    params(("callsign" = String, Path, description = "Callsign")), responses(
        (status = 200, description = "Expanded route with globally indexed legs matching validation fields", body = FlightRouteDto),
        (status = 400, description = "Invalid route"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Software engineer role required"),
        (status = 404, description = "Flight not found")
    )
)]
async fn flight_route_by_callsign(
    current_user: CurrentUser,
    State(service): State<FlightService>,
    Path(callsign): Path<String>,
) -> Result<Json<FlightRouteDto>, ApiError> {
    current_user.require_role(UserRole::SoftwareEngineer)?;
    let flight = service.find_by_callsign(&callsign).await?;
    Ok(Json(service.flight_route(&flight).await?))
}

/// Sends a JSON object mapping callsigns to FlightValidationDto. The initial
/// snapshot is sent immediately; changed snapshots follow at 30-second intervals.
/// Each snapshot replaces the previous one, including flights that disappeared.
#[utoipa::path(get, path = "api/flights/by-departure/{icao}/validations.ws", tag = "Flights",
    params(("icao" = String, Path, description = "Departure airport ICAO code")),
    responses((status = 101, description = "WebSocket sending complete callsign-to-validation maps on change"))
)]
async fn departure_validations_websocket(
    State(service): State<FlightService>,
    Path(icao): Path<String>,
    websocket: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let snapshot = departure_validation_snapshot(&service, &icao).await?;
    Ok(websocket
        .on_upgrade(move |socket| stream_validation_changes(socket, service, icao, snapshot)))
}

async fn departure_validation_snapshot(
    service: &FlightService,
    icao: &str,
) -> Result<ValidationSnapshot, FlightServiceError> {
    let flights = service.by_departure(icao).await?;
    futures::stream::iter(flights)
        .map(|flight| async move {
            let validation = service.validate(&flight).await?;
            Ok((flight.flight_plan.callsign, validation.into()))
        })
        .buffer_unordered(8)
        .try_collect()
        .await
}

async fn stream_validation_changes(
    mut socket: WebSocket,
    service: FlightService,
    icao: String,
    snapshot: ValidationSnapshot,
) {
    let mut payload =
        serde_json::to_string(&snapshot).expect("validation snapshot should serialize");
    if socket
        .send(Message::Text(payload.clone().into()))
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
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return,
                Some(Ok(_)) => {}
            },
            _ = refresh.tick() => {
                match departure_validation_snapshot(&service, &icao).await {
                    Ok(updated) => {
                        let updated = serde_json::to_string(&updated).expect("validation snapshot should serialize");
                        if updated != payload {
                            if socket.send(Message::Text(updated.clone().into())).await.is_err() {
                                return;
                            }
                            payload = updated;
                        }
                    }
                    Err(error) => tracing::warn!(%error, %icao, "failed to refresh flight validation websocket"),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use serde_json::json;

    fn query(value: serde_json::Value) -> TemporaryFlightQuery {
        serde_json::from_value(value).unwrap()
    }

    #[tokio::test]
    async fn temporary_flight_requires_api_client_permission() {
        let error = temporary_flight(
            CurrentUser::for_test_roles([UserRole::User]),
            Query(query(json!({ "departure": "ZBAA", "arrival": "ZSPD" }))),
        )
        .await
        .unwrap_err();
        assert_eq!(error.status_code(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn temporary_flight_returns_the_normalized_nested_plan() {
        let Json(dto) = temporary_flight(
            CurrentUser::for_test_roles([UserRole::ApiClient]),
            Query(query(json!({
                "departure": " zbaa ", "arrival": "zspd", "aircraft": "a320",
                "flight_rules": "v", "wake_category": "l", "equipment": "sgrw",
                "transponder": "lb1", "navigation_performance": "d1", "raw_route": "dct",
                "cruising_level": 33100
            }))),
        )
        .await
        .unwrap();
        let value = serde_json::to_value(dto).unwrap();
        assert_eq!(
            value["flight_plan"],
            json!({
                "callsign": "", "departure": "ZBAA", "arrival": "ZSPD", "aircraft": "A320",
                "flight_rules": "V", "wake_category": "L", "equipment": "SGRW",
                "transponder": "LB1", "navigation_performance": "D1", "raw_route": "DCT",
                "cruising_level": 33100
            })
        );
        assert_eq!(value["cid"], "");
        assert_eq!(value["altitude"], 0);
        assert!(value["last_observed_at"].is_string());
        assert!(value.get("departure").is_none());
    }

    #[test]
    fn temporary_flight_rejects_unrepresentable_rule_and_wake_codes() {
        for (field, code) in [("flight_rules", "X"), ("wake_category", "S")] {
            let mut input = json!({ "departure": "ZBAA", "arrival": "ZSPD" });
            input[field] = code.into();
            let error = temporary_flight_from_query(query(input)).unwrap_err();
            assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
        }
    }
}
