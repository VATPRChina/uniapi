use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::services::Services;

pub fn build_internal_routes() -> Router<Services> {
    Router::new().route("/not_found", get(endpoint_not_found))
}

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

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}
