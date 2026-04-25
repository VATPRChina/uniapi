use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::routes::auth::build_auth_routes;
use crate::routes::compat::build_compat_routes;
use crate::routes::storage::build_storage_routes;
use crate::services::Services;
use crate::{adapter::database::health as health_repository, auth};

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

pub fn router(services: Services) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/auth", build_auth_routes())
        .nest("/api/compat", build_compat_routes())
        .nest(
            "/api/storage",
            build_storage_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .with_state(services)
}

async fn root() -> &'static str {
    "vatprc uniapi rust service"
}

async fn health(State(services): State<Services>) -> impl IntoResponse {
    let database_is_healthy = health_repository::is_healthy(services.db()).await;

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
