use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_notams))]
pub(crate) struct ApiDoc;

pub fn build_notam_routes() -> Router<Services> {
    Router::new().route("/", get(list_notams))
}

#[utoipa::path(get, path = "api/notams", tag = "NOTAM", responses((status = 200, description = "Successful response", body = Vec<NotamDto>)))]
async fn list_notams(State(services): State<Services>) -> Result<Json<Vec<NotamDto>>, ApiError> {
    let endpoint = services.discourse().endpoint().trim_end_matches('/');
    let notams = services
        .discourse()
        .get_notam_topics()
        .await
        .map_err(ApiError::Discourse)?
        .topic_list
        .topics
        .into_iter()
        .map(|topic| NotamDto {
            title: topic.title,
            language_code: if topic.tags.iter().any(|tag| tag == "english") {
                "en"
            } else {
                "zh"
            },
            link: format!("{endpoint}/t/topic/{}", topic.id),
        })
        .collect();

    Ok(Json(notams))
}

#[derive(Serialize, utoipa::ToSchema)]
struct NotamDto {
    title: String,
    language_code: &'static str,
    link: String,
}
