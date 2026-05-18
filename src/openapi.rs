#![allow(dead_code)]

use axum::Json;
use utoipa::{OpenApi, PartialSchema, ToSchema};

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = crate::app::ApiDoc),
        (path = "/", api = crate::routes::auth::ApiDoc),
        (path = "/", api = crate::routes::session::ApiDoc),
        (path = "/", api = crate::routes::users::ApiDoc),
        (path = "/", api = crate::routes::atc::ApiDoc),
        (path = "/", api = crate::routes::user_atc_permissions::ApiDoc),
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
        (path = "/", api = crate::routes::preferred_routes::ApiDoc),
        (path = "/", api = crate::routes::sheets::ApiDoc),
        (path = "/", api = crate::routes::storage::ApiDoc),
        (path = "/", api = crate::routes::sectors::ApiDoc),
    ),
    modifiers(&SecurityAddon, &InternalServerErrorAddon),
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
        (name = "Sheets", description = "Sheet definitions"),
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

pub(crate) struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{
            AuthorizationCode, ClientCredentials, Flow, OAuth2, Scopes, SecurityScheme,
        };

        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "oauth2",
            SecurityScheme::OAuth2(OAuth2::new([
                Flow::AuthorizationCode(AuthorizationCode::with_refresh_url(
                    "/auth/authorize",
                    "/auth/token",
                    Scopes::new(),
                    "/auth/token",
                )),
                Flow::ClientCredentials(ClientCredentials::new("/auth/token", Scopes::new())),
            ])),
        );
    }
}

pub(crate) struct InternalServerErrorAddon;

impl utoipa::Modify for InternalServerErrorAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::path::Operation;
        use utoipa::openapi::response::Response;
        use utoipa::openapi::{Content, Ref, RefOr};

        const RESPONSE_NAME: &str = "InternalServerError";
        const STATUS_CODE: &str = "500";

        let components = openapi.components.get_or_insert_with(Default::default);
        components.schemas.insert(
            crate::error::ProblemDetails::name().into_owned(),
            crate::error::ProblemDetails::schema(),
        );
        components.responses.insert(
            RESPONSE_NAME.to_string(),
            RefOr::T(
                Response::builder()
                    .description("Internal server error")
                    .content(
                        "application/problem+json",
                        Content::new(Some(RefOr::Ref(Ref::from_schema_name(
                            crate::error::ProblemDetails::name(),
                        )))),
                    )
                    .build(),
            ),
        );

        for (path, path_item) in openapi.paths.paths.iter_mut() {
            if path.starts_with("/auth") {
                continue;
            }

            for operation in [
                &mut path_item.get,
                &mut path_item.put,
                &mut path_item.post,
                &mut path_item.delete,
                &mut path_item.options,
                &mut path_item.head,
                &mut path_item.patch,
                &mut path_item.trace,
            ]
            .into_iter()
            .flatten()
            {
                add_internal_server_error_response(operation);
            }
        }

        fn add_internal_server_error_response(operation: &mut Operation) {
            operation
                .responses
                .responses
                .entry(STATUS_CODE.to_string())
                .or_insert_with(|| RefOr::Ref(Ref::from_response_name(RESPONSE_NAME)));
        }
    }
}

#[cfg(test)]
mod tests {
    use utoipa::openapi::path::{Operation, PathItem};

    use super::openapi;

    #[test]
    fn non_auth_operations_include_internal_server_error_response() {
        let openapi = openapi();

        for (path, path_item) in &openapi.paths.paths {
            if path.starts_with("/auth") {
                continue;
            }

            for operation in operations(path_item) {
                assert!(
                    operation.responses.responses.contains_key("500"),
                    "{path} is missing a 500 response"
                );
            }
        }
    }

    #[test]
    fn auth_operations_do_not_include_internal_server_error_response() {
        let openapi = openapi();

        for (path, path_item) in &openapi.paths.paths {
            if !path.starts_with("/auth") {
                continue;
            }

            for operation in operations(path_item) {
                assert!(
                    !operation.responses.responses.contains_key("500"),
                    "{path} should not include a 500 response"
                );
            }
        }
    }

    #[test]
    fn sheets_operations_are_documented() {
        let openapi = serde_json::to_value(openapi()).expect("OpenAPI should serialize");

        assert!(openapi.pointer("/paths/~1api~1sheets/get").is_some());
        assert!(
            openapi
                .pointer("/paths/~1api~1sheets~1{sheetId}/get")
                .is_some()
        );
        assert!(
            openapi
                .pointer("/paths/~1api~1sheets~1{sheetId}/put")
                .is_some()
        );
        assert_eq!(
            openapi
                .pointer("/components/schemas/SheetSaveRequest/required")
                .and_then(|required| required.as_array())
                .map(|required| {
                    required
                        .iter()
                        .filter_map(|field| field.as_str())
                        .collect::<Vec<_>>()
                }),
            Some(vec!["name", "fields"])
        );
        assert_eq!(
            openapi.pointer("/components/schemas/SheetFieldDto/properties/sequence/format"),
            Some(&serde_json::Value::String("uint32".to_owned()))
        );
    }

    fn operations(path_item: &PathItem) -> impl Iterator<Item = &Operation> {
        [
            &path_item.get,
            &path_item.put,
            &path_item.post,
            &path_item.delete,
            &path_item.options,
            &path_item.head,
            &path_item.patch,
            &path_item.trace,
        ]
        .into_iter()
        .flatten()
    }
}
