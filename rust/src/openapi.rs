#![allow(dead_code)]

use axum::Json;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = crate::app::ApiDoc),
        (path = "/", api = crate::routes::auth::ApiDoc),
        (path = "/", api = crate::routes::session::ApiDoc),
        (path = "/", api = crate::routes::users::ApiDoc),
        (path = "/", api = crate::routes::atc::ApiDoc),
        (path = "/", api = crate::routes::user_atc_permissions::ApiDoc),
        (path = "/", api = crate::routes::atc_bookings::ApiDoc),
        (path = "/", api = crate::routes::atc_applications::ApiDoc),
        (path = "/", api = crate::routes::trainings::ApiDoc),
        (path = "/", api = crate::routes::training_applications::ApiDoc),
        (path = "/", api = crate::routes::events::ApiDoc),
        (path = "/", api = crate::routes::event_airspaces::ApiDoc),
        (path = "/", api = crate::routes::event_slots::ApiDoc),
        (path = "/", api = crate::routes::event_slot_bookings::ApiDoc),
        (path = "/", api = crate::routes::event_atc_positions::ApiDoc),
        (path = "/", api = crate::routes::flights::ApiDoc),
        (path = "/", api = crate::routes::compat::ApiDoc),
        (path = "/", api = crate::routes::notams::ApiDoc),
        (path = "/", api = crate::routes::preferred_routes::ApiDoc),
        (path = "/", api = crate::routes::sectors::ApiDoc),
        (path = "/", api = crate::routes::storage::ApiDoc),
        (path = "/", api = crate::routes::internal::ApiDoc)
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "OAuth and session endpoints"),
        (name = "ATC", description = "ATC status, bookings, applications, and training"),
        (name = "Compat", description = "Compatibility endpoints"),
        (name = "Events", description = "Events, slots, airspaces, and event ATC positions"),
        (name = "Flights", description = "VATSIM flight information"),
        (name = "Internal", description = "Internal fallback endpoints"),
        (name = "Navdata", description = "Navigation data endpoints"),
        (name = "NOTAM", description = "NOTAM endpoints"),
        (name = "Sectors", description = "Sector permission endpoints"),
        (name = "Session", description = "Current session endpoints"),
        (name = "Storage", description = "File storage endpoints"),
        (name = "Users", description = "User and role endpoints")
    )
)]
struct ApiDoc;

pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(openapi())
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

#[cfg(test)]
mod tests {
    #[test]
    fn generated_paths_are_normalized() {
        let openapi = super::openapi();

        assert!(
            openapi
                .paths
                .paths
                .keys()
                .all(|path| !path.starts_with("//")),
            "OpenAPI paths must not contain a double leading slash"
        );
    }
}

pub(crate) struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}
