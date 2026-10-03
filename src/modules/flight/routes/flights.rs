use std::collections::BTreeMap;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use tokio::time;

use crate::error::ApiError;
use crate::modules::flight::dto::{FlightDto, FlightLeg, FlightRouteV2Query, TemporaryFlightQuery};
use crate::modules::flight::flight_plan::validator;
use crate::modules::flight::models::Flight;
use crate::modules::flight::service::FlightService;
use crate::modules::user::middleware::CurrentUser;
use crate::modules::user::models::UserRole;
use crate::services::Services;

const VALIDATION_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    active_flights,
    flight_by_callsign,
    warnings_by_callsign,
    my_flight,
    temporary_warnings,
    route_v2
))]
pub(crate) struct ApiDoc;

pub fn build_flight_routes() -> Router<Services> {
    Router::new()
        .route("/active", get(active_flights))
        .route("/warnings/streaming", get(warnings_websocket))
        .route("/by-callsign/{callsign}", get(flight_by_callsign))
        .route(
            "/by-callsign/{callsign}/warnings",
            get(warnings_by_callsign),
        )
        .route("/mine", get(my_flight))
        .route("/temporary/by-plan/warnings", get(temporary_warnings))
        .route("/route/v2", get(route_v2))
}

/// Parse a complete route with v2 and return its expanded leg segments.
/// Requires the software-engineer role.
#[utoipa::path(
    get, path = "api/flights/route/v2", tag = "Flights", security(("oauth2" = [])),
    params(("route" = String, Query, description = "Complete route including departure and arrival")),
    responses(
        (status = 200, description = "Expanded route segments", body = Vec<FlightLeg>),
        (status = 400, description = "Invalid or incomplete route"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Software engineer role required")
    )
)]
async fn route_v2(
    current_user: CurrentUser,
    State(flight): State<FlightService>,
    Query(query): Query<FlightRouteV2Query>,
) -> Result<Json<Vec<FlightLeg>>, ApiError> {
    current_user.require_role(UserRole::SoftwareEngineer)?;
    Ok(Json(
        flight
            .route_v2(&query.route)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/flights/active", tag = "Flights", responses((status = 200, description = "Successful response", body = Vec<FlightDto>)))]
async fn active_flights(
    State(services): State<Services>,
) -> Result<Json<Vec<FlightDto>>, ApiError> {
    Ok(Json(
        services
            .flight()
            .list()
            .await?
            .into_iter()
            .map(FlightDto::from)
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/flights/by-callsign/{callsign}", tag = "Flights", params(("callsign" = String, Path, description = "Callsign")), responses((status = 200, description = "Successful response", body = FlightDto)))]
async fn flight_by_callsign(
    State(services): State<Services>,
    Path(callsign): Path<String>,
) -> Result<Json<FlightDto>, ApiError> {
    Ok(Json(
        services.flight().find_by_callsign(&callsign).await?.into(),
    ))
}

#[utoipa::path(get, path = "api/flights/by-callsign/{callsign}/warnings", tag = "Flights", params(("callsign" = String, Path, description = "Callsign")), responses((status = 200, description = "Successful response", body = Vec<validator::WarningMessage>)))]
async fn warnings_by_callsign(
    State(services): State<Services>,
    Path(callsign): Path<String>,
) -> Result<Json<Vec<validator::WarningMessage>>, ApiError> {
    Ok(Json(
        services.flight().warnings_by_callsign(&callsign).await?,
    ))
}

async fn warnings_websocket(
    State(services): State<Services>,
    websocket: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let flight = services.flight().clone();
    let initial_snapshot = flight.warnings_for_all().await?;

    Ok(
        websocket
            .on_upgrade(move |socket| stream_warning_changes(socket, flight, initial_snapshot)),
    )
}

#[utoipa::path(get, path = "api/flights/temporary/by-plan/warnings", tag = "Flights", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<validator::WarningMessage>)))]
async fn temporary_warnings(
    current_user: CurrentUser,
    State(services): State<Services>,
    Query(query): Query<TemporaryFlightQuery>,
) -> Result<Json<Vec<validator::WarningMessage>>, ApiError> {
    current_user.require_role(UserRole::ApiClient)?;
    Ok(Json(
        services.flight().warnings(&Flight::from(query)).await?,
    ))
}

#[utoipa::path(get, path = "api/flights/mine", tag = "Flights", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = FlightDto)))]
async fn my_flight(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<FlightDto>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    Ok(Json(services.flight().find_by_user(user_id).await?.into()))
}

async fn stream_warning_changes(
    mut socket: WebSocket,
    flight: FlightService,
    mut snapshot: BTreeMap<String, Vec<validator::WarningMessage>>,
) {
    if send_validation_snapshot(&mut socket, &snapshot)
        .await
        .is_err()
    {
        return;
    }

    let mut refresh = time::interval(VALIDATION_REFRESH_INTERVAL);
    refresh.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    refresh.tick().await;

    loop {
        tokio::select! {
            message = socket.recv() => match message {
                Some(Ok(Message::Close(_))) | None => return,
                Some(Err(error)) => {
                    tracing::debug!(%error, "flight validation websocket closed");
                    return;
                }
                Some(Ok(_)) => {}
            },
            _ = refresh.tick() => {
                match flight.warnings_for_all().await {
                    Ok(updated) if updated != snapshot => {
                        if send_validation_snapshot(&mut socket, &updated).await.is_err() {
                            return;
                        }
                        snapshot = updated;
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(%error, "failed to refresh flight validation websocket");
                    }
                }
            }
        }
    }
}

async fn send_validation_snapshot(
    socket: &mut WebSocket,
    snapshot: &BTreeMap<String, Vec<validator::WarningMessage>>,
) -> Result<(), axum::Error> {
    let payload =
        serde_json::to_string(snapshot).expect("flight validation snapshot should serialize");
    socket.send(Message::Text(payload.into())).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adapter::{compat::CompatClient, moodle::MoodleClient},
        modules::{
            audit_log::service::AuditLogService, navdata::service::NavdataService,
            user::service::user::UserService,
        },
    };
    use axum::http::{StatusCode, Uri};

    async fn navdata() -> NavdataService {
        NavdataService::with_preferred_routes_path(
            "data/NavigraphDFDv2-2604.1.0.db?mode=ro",
            "data/Route-Server.csv",
        )
        .await
        .unwrap()
    }

    fn flight_service(navdata: NavdataService) -> FlightService {
        // Parsing uses only local navdata. The other service dependencies stay idle.
        let db = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://test:test@localhost/test")
            .unwrap();
        let user = UserService::new(
            db.clone(),
            MoodleClient::new(String::new()),
            AuditLogService::new(db),
        );
        FlightService::new(CompatClient::new(String::new()), navdata, user)
    }

    fn query(route: &str) -> Query<FlightRouteV2Query> {
        Query(FlightRouteV2Query {
            route: route.to_owned(),
        })
    }

    #[tokio::test]
    async fn route_v2_requires_an_authenticated_current_user() {
        use axum::extract::FromRequestParts;
        let request = axum::http::Request::builder()
            .uri("/api/flights/route/v2?route=ZBAA%20ZSPD")
            .body(())
            .unwrap();
        let error = CurrentUser::from_request_parts(&mut request.into_parts().0, &())
            .await
            .unwrap_err();
        assert_eq!(
            ApiError::from(error).status_code(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn route_v2_rejects_non_developers_before_accessing_navdata() {
        let navdata = navdata().await;
        navdata.db.close().await;
        let flight = flight_service(navdata);
        for role in [UserRole::ApiClient, UserRole::User, UserRole::Volunteer] {
            let result = route_v2(
                CurrentUser::for_test_roles([role]),
                State(flight.clone()),
                query("ZBAA ELKUR W40 YQG ZSPD"),
            )
            .await;
            assert!(
                matches!(result, Err(ApiError::Forbidden { allowed_roles }) if allowed_roles == [UserRole::SoftwareEngineer].into_iter().collect())
            );
        }
    }

    #[tokio::test]
    async fn route_v2_returns_expanded_route_for_developers() {
        let flight = flight_service(navdata().await);
        let uri: Uri = "/route/v2?route=ZBAA%20ELKUR%20W40%20YQG%20ZSPD"
            .parse()
            .unwrap();
        for role in [UserRole::SoftwareEngineer, UserRole::TechDirectorAssistant] {
            let query = Query::<FlightRouteV2Query>::try_from_uri(&uri).unwrap();
            let Json(route) = route_v2(
                CurrentUser::for_test_roles([role]),
                State(flight.clone()),
                query,
            )
            .await
            .unwrap();
            let json = serde_json::to_value(route).unwrap();
            let segments = json.as_array().unwrap();
            assert_eq!(segments.first().unwrap()["from"]["identifier"], "ZBAA");
            assert_eq!(segments.last().unwrap()["to"]["identifier"], "ZSPD");
            assert!(segments.iter().any(|leg| leg["to"]["identifier"] == "PANKI" && leg["leg_identifier"] == "W40"));
            assert!(
                segments
                    .windows(2)
                    .all(|pair| pair[0]["to"] == pair[1]["from"])
            );
        }
    }

    #[tokio::test]
    async fn route_v2_reports_invalid_and_incomplete_routes_as_bad_requests() {
        let flight = flight_service(navdata().await);
        for route in [
            "",
            "ZBAA",
            "ZBAA IFR ZSPD",
            "ZBAA N0450F350 N0460F360 ZSPD",
            "ZBAA DCT DCT ZSPD",
        ] {
            let error = route_v2(
                CurrentUser::for_test_roles([UserRole::SoftwareEngineer]),
                State(flight.clone()),
                query(route),
            )
            .await
            .err()
            .unwrap();
            assert_eq!(error.status_code(), StatusCode::BAD_REQUEST, "{route:?}");
            assert!(matches!(error, ApiError::BadRequest { field, .. } if field == "route"));
        }
    }

    #[tokio::test]
    async fn route_v2_propagates_navdata_errors() {
        let navdata = navdata().await;
        navdata.db.close().await;
        let result = route_v2(
            CurrentUser::for_test_roles([UserRole::SoftwareEngineer]),
            State(flight_service(navdata)),
            query("ZBAA ZSPD"),
        )
        .await;
        assert!(matches!(result, Err(ApiError::RouteParser { .. })));
    }

    #[test]
    fn route_v2_is_documented_with_query_authentication_and_response_schema() {
        let doc = serde_json::to_value(crate::openapi::openapi()).unwrap();
        let operation = doc.pointer("/paths/~1api~1flights~1route~1v2/get").unwrap();
        assert_eq!(operation["security"][0]["oauth2"], serde_json::json!([]));
        assert_eq!(operation["parameters"][0]["name"], "route");
        assert_eq!(operation["parameters"][0]["required"], true);
        for status in ["200", "400", "401", "403", "500"] {
            assert!(operation["responses"].get(status).is_some());
        }
        assert_eq!(
            operation["responses"]["200"]["content"]["application/json"]["schema"]["items"]["$ref"],
            "#/components/schemas/FlightLeg"
        );
    }
}
