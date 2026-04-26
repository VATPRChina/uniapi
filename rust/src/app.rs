use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::openapi::{openapi, openapi_json};
use crate::routes::atc::build_atc_routes;
use crate::routes::atc_applications::build_atc_application_routes;
use crate::routes::atc_bookings::build_atc_booking_routes;
use crate::routes::auth::build_auth_routes;
use crate::routes::compat::build_compat_routes;
use crate::routes::event_airspaces::{
    build_protected_event_airspace_routes, build_public_event_airspace_routes,
};
use crate::routes::event_atc_positions::{
    build_protected_event_atc_position_routes, build_public_event_atc_position_routes,
};
use crate::routes::event_slot_bookings::{
    build_protected_event_slot_booking_routes, build_public_event_slot_booking_routes,
};
use crate::routes::event_slots::{
    build_protected_event_slot_routes, build_public_event_slot_routes,
};
use crate::routes::events::{build_protected_event_routes, build_public_event_routes};
use crate::routes::flights::{build_protected_flight_routes, build_public_flight_routes};
use crate::routes::internal::build_internal_routes;
use crate::routes::notams::build_notam_routes;
use crate::routes::preferred_routes::build_preferred_route_routes;
use crate::routes::sectors::build_sector_routes;
use crate::routes::session::build_session_routes;
use crate::routes::storage::build_storage_routes;
use crate::routes::training_applications::build_training_application_routes;
use crate::routes::trainings::build_training_routes;
use crate::routes::user_atc_permissions::build_user_atc_permission_routes;
use crate::routes::users::build_user_routes;
use crate::services::Services;
use crate::{adapter::database::health as health_repository, auth};

#[derive(Serialize, utoipa::ToSchema)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(health))]
pub(crate) struct ApiDoc;

pub fn router(services: Services) -> Router {
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/auth", build_auth_routes())
        .nest("/api/__internal", build_internal_routes())
        .nest("/api/atc/controllers", build_atc_routes())
        .nest(
            "/api/atc/bookings",
            build_atc_booking_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest(
            "/api/atc/applications",
            build_atc_application_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest("/api/compat", build_compat_routes())
        .nest("/api/notams", build_notam_routes())
        .nest(
            "/api/navdata/preferred-routes",
            build_preferred_route_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest("/api/flights", build_public_flight_routes())
        .nest(
            "/api/flights",
            build_protected_flight_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest("/api/events", build_public_event_routes())
        .nest("/api/events", build_public_event_airspace_routes())
        .nest("/api/events", build_public_event_atc_position_routes())
        .nest("/api/events", build_public_event_slot_booking_routes())
        .nest("/api/events", build_public_event_slot_routes())
        .nest(
            "/api/events",
            build_protected_event_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest(
            "/api/events",
            build_protected_event_slot_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest(
            "/api/events",
            build_protected_event_slot_booking_routes().route_layer(
                middleware::from_fn_with_state(services.clone(), auth::authenticate),
            ),
        )
        .nest(
            "/api/events",
            build_protected_event_atc_position_routes().route_layer(
                middleware::from_fn_with_state(services.clone(), auth::authenticate),
            ),
        )
        .nest(
            "/api/events",
            build_protected_event_airspace_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest(
            "/api/session",
            build_session_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest(
            "/api/sectors",
            build_sector_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest(
            "/api/storage",
            build_storage_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest(
            "/api/atc/trainings/applications",
            build_training_application_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest(
            "/api/atc/trainings",
            build_training_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest(
            "/api/users",
            build_user_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .nest(
            "/api/users",
            build_user_atc_permission_routes().route_layer(middleware::from_fn_with_state(
                services.clone(),
                auth::authenticate,
            )),
        )
        .with_state(services);
    let (router, openapi) = OpenApiRouter::with_openapi(openapi())
        .merge(app.into())
        .split_for_parts();

    router
        .route("/openapi.json", get(openapi_json))
        .merge(Scalar::with_url("/docs", openapi))
}

async fn root() -> &'static str {
    "vatprc uniapi rust service"
}

#[utoipa::path(get, path = "health", tag = "Health", responses((status = 200, description = "Successful response", body = HealthResponse)))]
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
