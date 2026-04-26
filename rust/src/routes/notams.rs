use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::{adapter::discourse::DiscourseError, services::Services};

pub fn build_notam_routes() -> Router<Services> {
    Router::new().route("/", get(list_notams))
}

async fn list_notams(
    State(services): State<Services>,
) -> Result<Json<Vec<NotamDto>>, NotamRouteError> {
    let endpoint = services.discourse().endpoint().trim_end_matches('/');
    let notams = services
        .discourse()
        .get_notam_topics()
        .await
        .map_err(NotamRouteError::Discourse)?
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

#[derive(Serialize)]
struct NotamDto {
    title: String,
    language_code: &'static str,
    link: String,
}

#[derive(Debug)]
enum NotamRouteError {
    Discourse(DiscourseError),
}

impl IntoResponse for NotamRouteError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            NotamRouteError::Discourse(error) => (StatusCode::BAD_GATEWAY, error.to_string()),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}
