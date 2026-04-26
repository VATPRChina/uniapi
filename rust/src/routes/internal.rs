use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(endpoint_not_found))]
pub(crate) struct ApiDoc;

pub fn build_internal_routes() -> Router<Services> {
    Router::new().route("/not_found", get(endpoint_not_found))
}

#[utoipa::path(get, path = "api/__internal/not_found", tag = "Internal", responses((status = 404, description = "Endpoint not found", body = ErrorResponse)))]
async fn endpoint_not_found() -> InternalRouteError {
    InternalRouteError::EndpointNotFound
}

#[derive(Debug)]
enum InternalRouteError {
    EndpointNotFound,
}

impl IntoResponse for InternalRouteError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            InternalRouteError::EndpointNotFound => {
                (StatusCode::NOT_FOUND, "endpoint not found".into())
            }
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct ErrorResponse {
    message: String,
}
