use axum::{Json, Router, routing::get};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
}

async fn root() -> &'static str {
    "vatprc uniapi rust service"
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
