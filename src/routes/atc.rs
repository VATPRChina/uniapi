use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::dto::*;
use crate::repository::atc::atc::AtcRepositoryExt;
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_controllers))]
pub(crate) struct ApiDoc;

pub fn build_atc_routes() -> Router<Services> {
    Router::new().route("/", get(list_controllers))
}

#[utoipa::path(get, path = "api/atc/controllers", tag = "ATC", responses((status = 200, description = "Successful response", body = Vec<AtcStatusDto>)))]
async fn list_controllers(
    State(services): State<Services>,
) -> Result<Json<Vec<AtcStatusDto>>, ApiError> {
    let rows = services.db().list_atc_controllers().await?;
    Ok(Json(AtcStatusDto::from_controller_rows(rows)?))
}
