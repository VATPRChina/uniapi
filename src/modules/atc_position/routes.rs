use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use crate::error::ApiError;
use crate::modules::user::middleware::CurrentUser;
use crate::modules::user::models::UserRole;
use crate::services::Services;

use super::dto::{AtcPositionDto, AtcPositionSaveRequest, normalize_callsign};
use super::models::AtcPositionSave;

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    list_atc_positions,
    get_atc_position,
    create_atc_position,
    update_atc_position,
    delete_atc_position
))]
pub(crate) struct ApiDoc;

pub fn build_atc_position_routes() -> Router<Services> {
    Router::new()
        .route("/", get(list_atc_positions).post(create_atc_position))
        .route(
            "/{callsign}",
            get(get_atc_position)
                .put(update_atc_position)
                .delete(delete_atc_position),
        )
}

#[utoipa::path(get, path = "api/atc/positions", tag = "ATC Positions", responses((status = 200, description = "All published ATC positions and frequencies", body = Vec<AtcPositionDto>)))]
async fn list_atc_positions(
    State(services): State<Services>,
) -> Result<Json<Vec<AtcPositionDto>>, ApiError> {
    services
        .atc_position()
        .list()
        .await?
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>, _>>()
        .map(Json)
}

#[utoipa::path(get, path = "api/atc/positions/{callsign}", tag = "ATC Positions", params(("callsign" = String, Path, description = "ATC callsign")), responses((status = 200, description = "ATC position", body = AtcPositionDto)))]
async fn get_atc_position(
    State(services): State<Services>,
    Path(callsign): Path<String>,
) -> Result<Json<AtcPositionDto>, ApiError> {
    let callsign = normalize_callsign(callsign)?;
    services
        .atc_position()
        .find(&callsign)
        .await?
        .try_into()
        .map(Json)
}

#[utoipa::path(post, path = "api/atc/positions", tag = "ATC Positions", security(("oauth2" = [])), request_body = AtcPositionSaveRequest, responses((status = 200, description = "ATC position created", body = AtcPositionDto)))]
async fn create_atc_position(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<AtcPositionSaveRequest>,
) -> Result<Json<AtcPositionDto>, ApiError> {
    current_user.require_role(UserRole::TechAfvFacilityEngineer)?;
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    services
        .atc_position()
        .create(request.try_into()?, operated_by)
        .await?
        .try_into()
        .map(Json)
}

#[utoipa::path(put, path = "api/atc/positions/{callsign}", tag = "ATC Positions", security(("oauth2" = [])), params(("callsign" = String, Path, description = "ATC callsign")), request_body = AtcPositionSaveRequest, responses((status = 200, description = "ATC position updated", body = AtcPositionDto)))]
async fn update_atc_position(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(callsign): Path<String>,
    Json(request): Json<AtcPositionSaveRequest>,
) -> Result<Json<AtcPositionDto>, ApiError> {
    current_user.require_role(UserRole::TechAfvFacilityEngineer)?;
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let callsign = normalize_callsign(callsign)?;
    let position: AtcPositionSave = request.try_into()?;
    if position.callsign != callsign {
        return Err(ApiError::bad_request(
            "callsign",
            "must match the callsign in the request path",
        ));
    }
    services
        .atc_position()
        .update(&callsign, position, operated_by)
        .await?
        .try_into()
        .map(Json)
}

#[utoipa::path(delete, path = "api/atc/positions/{callsign}", tag = "ATC Positions", security(("oauth2" = [])), params(("callsign" = String, Path, description = "ATC callsign")), responses((status = 204, description = "ATC position deleted")))]
async fn delete_atc_position(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(callsign): Path<String>,
) -> Result<StatusCode, ApiError> {
    current_user.require_role(UserRole::TechAfvFacilityEngineer)?;
    let operated_by = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let callsign = normalize_callsign(callsign)?;
    services
        .atc_position()
        .delete(&callsign, operated_by)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
