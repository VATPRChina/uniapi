use axum::extract::State;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router, middleware};
use opentelemetry::KeyValue;
use serde::Serialize;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{Level, instrument};
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::auth;
use crate::error::ApiError;
use crate::openapi::{openapi, openapi_json};
use crate::routes::atc::build_atc_routes;
use crate::routes::atc_applications::build_atc_application_routes;
use crate::routes::audit_logs::build_audit_log_routes;
use crate::routes::auth::build_auth_routes;
use crate::routes::compat::build_compat_routes;
use crate::routes::event_airspaces::build_event_airspace_routes;
use crate::routes::event_atc_positions::build_event_atc_position_routes;
use crate::routes::event_slot_bookings::build_event_slot_booking_routes;
use crate::routes::event_slots::build_event_slot_routes;
use crate::routes::events::build_event_routes;
use crate::routes::flights::build_flight_routes;
use crate::routes::preferred_routes::build_preferred_route_routes;
use crate::routes::sectors::build_sector_routes;
use crate::routes::session::build_session_routes;
use crate::routes::sheets::build_sheet_routes;
use crate::routes::storage::build_storage_routes;
use crate::routes::training_applications::build_training_application_routes;
use crate::routes::trainings::build_training_routes;
use crate::routes::user_atc_permissions::build_user_atc_permission_routes;
use crate::routes::users::build_user_routes;
use crate::services::Services;

#[derive(Serialize, utoipa::ToSchema)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(health))]
pub(crate) struct ApiDoc;

pub fn router(services: Services) -> Router {
    let auth_services = services.clone();
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/auth", build_auth_routes())
        .nest("/api", build_audit_log_routes())
        .nest("/api/atc/controllers", build_atc_routes())
        .nest("/api/atc/applications", build_atc_application_routes())
        .nest("/api/compat", build_compat_routes())
        .nest(
            "/api/navdata/preferred-routes",
            build_preferred_route_routes(),
        )
        .nest("/api/flights", build_flight_routes())
        .nest("/api/events", build_event_routes())
        .nest("/api/events", build_event_airspace_routes())
        .nest("/api/events", build_event_atc_position_routes())
        .nest("/api/events", build_event_slot_booking_routes())
        .nest("/api/events", build_event_slot_routes())
        .nest("/api/session", build_session_routes())
        .nest("/api/sheets", build_sheet_routes())
        .nest("/api/storage", build_storage_routes())
        .nest(
            "/api/atc/trainings/applications",
            build_training_application_routes(),
        )
        .nest("/api/atc/trainings", build_training_routes())
        .nest("/api/users", build_user_routes())
        .nest("/api/users", build_user_atc_permission_routes())
        .nest("/api/sectors", build_sector_routes())
        .with_state(services);
    let (router, openapi) = OpenApiRouter::with_openapi(openapi())
        .merge(app.into())
        .split_for_parts();

    router
        .route("/openapi.json", get(openapi_json))
        .merge(Scalar::with_url("/docs", openapi))
        .layer(middleware::from_fn_with_state(
            auth_services,
            auth::authenticate,
        ))
        .layer(middleware::from_fn(record_request_status))
        .layer(cors_layer())
        .layer(CatchPanicLayer::custom(
            |e: Box<dyn std::any::Any + Send + 'static>| {
                if let Some(s) = e.downcast_ref::<String>() {
                    tracing::error!("service panicked: {}", s);
                } else if let Some(s) = e.downcast_ref::<&str>() {
                    tracing::error!("service panicked: {}", s);
                } else {
                    tracing::error!(
                        "service panicked but `CatchPanic` was unable to downcast the panic info"
                    );
                };

                ApiError::Internal.into_response()
            },
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}

fn cors_layer() -> CorsLayer {
    let allow_origin = AllowOrigin::predicate(|origin, _| is_vatprc_origin(origin));

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods(Any)
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}

fn is_vatprc_origin(origin: &HeaderValue) -> bool {
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    let Ok(url) = url::Url::parse(origin) else {
        return false;
    };

    url.scheme() == "https"
        && url
            .host_str()
            .is_some_and(|host| host.ends_with(".vatprc.net"))
}

#[instrument(skip(request, next), fields(http.method = %request.method(), http.target = %request.uri().path()))]
async fn record_request_status(
    request: axum::extract::Request,
    next: middleware::Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let response = next.run(request).await;
    let status = response.status();

    opentelemetry::global::meter("vatprc-uniapi")
        .u64_counter("http.server.request.status")
        .with_description("Number of HTTP responses by status code")
        .build()
        .add(
            1,
            &[
                KeyValue::new("http.request.method", method.to_string()),
                KeyValue::new("url.path", path),
                KeyValue::new("http.response.status_code", i64::from(status.as_u16())),
            ],
        );

    response
}

#[instrument]
async fn root() -> &'static str {
    "vatprc uniapi rust service"
}

#[utoipa::path(get, path = "health", tag = "Health", responses((status = 200, description = "Successful response", body = HealthResponse)))]
#[instrument(skip(services))]
async fn health(State(services): State<Services>) -> impl IntoResponse {
    tracing::info!("performing health check");
    let database_is_healthy = matches!(
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(services.db())
            .await,
        Ok(1)
    );
    tracing::info!(
        "database health check result: {}",
        if database_is_healthy {
            "ok"
        } else {
            "unavailable"
        }
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
