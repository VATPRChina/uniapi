use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    type_: &'static str,
    title: String,
    status: u16,
    detail: String,
}

pub fn problem_response(status: StatusCode, detail: impl Into<String>) -> Response {
    let title = status
        .canonical_reason()
        .unwrap_or("HTTP error")
        .to_string();
    let mut response = (
        status,
        Json(ProblemDetails {
            type_: "about:blank",
            title,
            status: status.as_u16(),
            detail: detail.into(),
        }),
    )
        .into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        "application/problem+json"
            .parse()
            .expect("valid media type"),
    );
    response
}
