use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use crate::adapter::compat::CompatClientError;
use crate::adapter::database::compat::{self as compat_repository, FutureControllerRow};
use crate::services::Services;

static VATPRC_CONTROLLER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(Z[BSGUHWJPLYM][A-Z0-9]{2}(_[A-Z0-9]*)?_(DEL|GND|TWR|APP|DEP|CTR))|(PRC_FSS)$")
        .expect("VATPRC controller regex should compile")
});
static VATPRC_AIRPORT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^Z[BMSPGJYWLH][A-Z]{2}").expect("VATPRC airport regex should compile")
});

pub fn build_compat_routes() -> Router<Services> {
    Router::new()
        .route("/online-status", get(online_status))
        .route("/euroscope/metar/{icao}", get(get_metar_by_path))
        .route("/euroscope/metar/metar.php", get(get_metar_by_query))
        .route("/homepage/events/vatsim", get(vatsim_events))
        .route("/trackaudio/mandatory_version", get(trackaudio_version))
        .route("/vplaaf/areas.json", get(vplaaf_areas))
}

async fn online_status(
    State(services): State<Services>,
) -> Result<Json<CompatVatprcStatusDto>, CompatError> {
    let vatsim_data = services.compat().get_online_data().await?;
    let future_controllers = compat_repository::future_controllers(services.db())
        .await
        .map_err(CompatError::Database)?
        .into_iter()
        .map(CompatFutureControllerDto::from)
        .collect();

    let pilots = vatsim_data
        .pilots
        .into_iter()
        .filter_map(|pilot| {
            let flight_plan = pilot.flight_plan?;
            let departure_matches = flight_plan
                .departure
                .as_deref()
                .is_some_and(|airport| VATPRC_AIRPORT_REGEX.is_match(airport));
            let arrival_matches = flight_plan
                .arrival
                .as_deref()
                .is_some_and(|airport| VATPRC_AIRPORT_REGEX.is_match(airport));
            if !departure_matches && !arrival_matches {
                return None;
            }

            Some(CompatPilotDto {
                cid: pilot.cid as i32,
                name: pilot.name,
                callsign: pilot.callsign,
                departure: flight_plan.departure,
                arrival: flight_plan.arrival,
                aircraft: flight_plan.aircraft_short,
            })
        })
        .collect();

    let controllers = vatsim_data
        .controllers
        .into_iter()
        .filter(|controller| VATPRC_CONTROLLER_REGEX.is_match(&controller.callsign))
        .filter(|controller| controller.facility > 0)
        .map(|controller| CompatControllerDto {
            cid: controller.cid as i32,
            name: controller.name,
            callsign: controller.callsign,
            frequency: controller.frequency,
        })
        .collect();

    Ok(Json(CompatVatprcStatusDto {
        last_updated: vatsim_data.general.update_timestamp,
        pilots,
        controllers,
        future_controllers,
    }))
}

async fn get_metar_by_path(State(services): State<Services>, Path(icao): Path<String>) -> Response {
    metar_response(services, icao).await
}

async fn get_metar_by_query(
    State(services): State<Services>,
    Query(query): Query<MetarQuery>,
) -> Response {
    metar_response(services, query.id).await
}

async fn metar_response(services: Services, icao: String) -> Response {
    let normalized_icao = icao.to_uppercase();
    let metar = services.compat().get_metar(&normalized_icao).await;
    if metar.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            format!("{normalized_icao} NO METAR"),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        metar,
    )
        .into_response()
}

async fn vatsim_events(State(services): State<Services>) -> Result<Response, CompatError> {
    json_text_response(services.compat().get_vatsim_events().await)
}

async fn trackaudio_version(State(services): State<Services>) -> Result<Response, CompatError> {
    text_response(services.compat().get_track_audio_version().await)
}

async fn vplaaf_areas(State(services): State<Services>) -> Result<Response, CompatError> {
    json_text_response(services.compat().get_vplaaf_areas().await)
}

fn text_response(content: Result<String, CompatClientError>) -> Result<Response, CompatError> {
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        content?,
    )
        .into_response())
}

fn json_text_response(content: Result<String, CompatClientError>) -> Result<Response, CompatError> {
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        content?,
    )
        .into_response())
}

#[derive(Deserialize)]
struct MetarQuery {
    id: String,
}

#[derive(Serialize)]
struct CompatVatprcStatusDto {
    last_updated: DateTime<Utc>,
    pilots: Vec<CompatPilotDto>,
    controllers: Vec<CompatControllerDto>,
    future_controllers: Vec<CompatFutureControllerDto>,
}

#[derive(Serialize)]
struct CompatPilotDto {
    cid: i32,
    name: String,
    callsign: String,
    departure: Option<String>,
    arrival: Option<String>,
    aircraft: Option<String>,
}

#[derive(Serialize)]
struct CompatControllerDto {
    cid: i32,
    name: String,
    callsign: String,
    frequency: String,
}

#[derive(Serialize)]
struct CompatFutureControllerDto {
    callsign: String,
    name: String,
    start: String,
    start_utc: DateTime<Utc>,
    end: String,
    end_utc: DateTime<Utc>,
}

impl From<FutureControllerRow> for CompatFutureControllerDto {
    fn from(row: FutureControllerRow) -> Self {
        Self {
            callsign: row.callsign,
            name: row.name,
            start: row.start_at.format("%d %H:%M").to_string(),
            start_utc: row.start_at,
            end: row.end_at.format("%d %H:%M").to_string(),
            end_utc: row.end_at,
        }
    }
}

#[derive(Debug)]
enum CompatError {
    Client(CompatClientError),
    Database(sqlx::Error),
}

impl From<CompatClientError> for CompatError {
    fn from(error: CompatClientError) -> Self {
        Self::Client(error)
    }
}

impl IntoResponse for CompatError {
    fn into_response(self) -> Response {
        let message = match self {
            CompatError::Client(error) => format!("Upstream compatibility service failed: {error}"),
            CompatError::Database(error) => format!("Database query failed: {error}"),
        };

        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_error::ErrorResponse { message }),
        )
            .into_response()
    }
}

mod serde_error {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct ErrorResponse {
        pub message: String,
    }
}
