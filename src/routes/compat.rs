use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};

use crate::error::ApiError;
use crate::modules::compat::dto::{CompatVatprcStatusDto, MetarQuery};
use crate::modules::compat::service::CompatServiceError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    vplaaf_areas,
    trackaudio_version,
    get_metar_by_query,
    get_metar_by_path,
    online_status
))]
pub(crate) struct ApiDoc;

pub fn build_compat_routes() -> Router<Services> {
    Router::new()
        .route("/online-status", get(online_status))
        .route("/euroscope/metar/{icao}", get(get_metar_by_path))
        .route("/euroscope/metar/metar.php", get(get_metar_by_query))
        .route("/trackaudio/mandatory_version", get(trackaudio_version))
        .route("/vplaaf/areas.json", get(vplaaf_areas))
}

#[utoipa::path(get, path = "api/compat/online-status", tag = "Compat", responses((status = 200, description = "Successful response", body = CompatVatprcStatusDto)))]
async fn online_status(
    State(services): State<Services>,
) -> Result<Json<CompatVatprcStatusDto>, ApiError> {
    Ok(Json(services.compat().online_status().await?.into()))
}

#[utoipa::path(get, path = "api/compat/euroscope/metar/{icao}", tag = "Compat", params(("icao" = String, Path, description = "ICAO code")), responses((status = 200, description = "Successful response", body = String)))]
async fn get_metar_by_path(State(services): State<Services>, Path(icao): Path<String>) -> Response {
    metar_response(services, icao).await
}

#[utoipa::path(get, path = "api/compat/euroscope/metar/metar.php", tag = "Compat", params(("icao" = Option<String>, Query, description = "ICAO code")), responses((status = 200, description = "Successful response", body = String)))]
async fn get_metar_by_query(
    State(services): State<Services>,
    Query(query): Query<MetarQuery>,
) -> Response {
    metar_response(services, query.id).await
}

async fn metar_response(services: Services, icao: String) -> Response {
    let normalized_icao = icao.to_uppercase();
    let metar = services.compat().metar(&normalized_icao).await;
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

#[utoipa::path(get, path = "api/compat/trackaudio/mandatory_version", tag = "Compat", responses((status = 200, description = "Successful response", body = String)))]
async fn trackaudio_version(State(services): State<Services>) -> Result<Response, ApiError> {
    text_response(services.compat().track_audio_version().await)
}

#[utoipa::path(get, path = "api/compat/vplaaf/areas.json", tag = "Compat", responses((status = 200, description = "Successful response", body = serde_json::Value)))]
async fn vplaaf_areas(State(services): State<Services>) -> Result<Response, ApiError> {
    json_text_response(services.compat().vplaaf_areas().await)
}

fn text_response(content: Result<String, CompatServiceError>) -> Result<Response, ApiError> {
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        content?,
    )
        .into_response())
}

fn json_text_response(content: Result<String, CompatServiceError>) -> Result<Response, ApiError> {
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        content?,
    )
        .into_response())
}
