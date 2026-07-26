use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use ulid::Ulid;

use crate::modules::controller::dto::AtcStatusDto;
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
    let controllers = services.controller().list().await?;
    let user_ids = controllers
        .iter()
        .map(|controller| controller.user_id)
        .collect::<Vec<_>>();
    let mut users = services.user().get_users_bulk(&user_ids).await?;
    let controllers = controllers
        .into_iter()
        .map(|controller| {
            let user = users.remove(&controller.user_id).ok_or_else(|| {
                ApiError::not_found("user", Ulid::from(controller.user_id).to_string())
            })?;
            Ok((controller, user).into())
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    Ok(Json(controllers))
}
