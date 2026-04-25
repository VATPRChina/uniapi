use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use serde::Serialize;

use crate::services::Services;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

pub fn router(services: Services) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .with_state(services)
}

async fn root() -> &'static str {
    "vatprc uniapi rust service"
}

async fn health(State(services): State<Services>) -> impl IntoResponse {
    let database_is_healthy = matches!(
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(services.db())
            .await,
        Ok(1)
    );

    let status = if database_is_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(HealthResponse {
            status: if database_is_healthy {
                "ok"
            } else {
                "unavailable"
            },
            database: if database_is_healthy {
                "ok"
            } else {
                "unavailable"
            },
        }),
    )
        .into_response()
}
