use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Serialize;

use crate::{adapter::smms::SmmsError, services::Services};

pub fn build_storage_routes() -> Router<Services> {
    Router::new().route("/api/storage/images", post(upload_image))
}

#[derive(Serialize)]
pub struct UploadImageResponse {
    url: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

pub async fn upload_image(
    State(services): State<Services>,
    mut multipart: Multipart,
) -> Result<Json<UploadImageResponse>, StorageError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(StorageError::Multipart)?
    {
        if field.name() != Some("image") {
            continue;
        }

        let file_name = field.file_name().map(ToOwned::to_owned);
        let content_type = field.content_type().map(ToOwned::to_owned);
        let image = field
            .bytes()
            .await
            .map_err(StorageError::Multipart)?
            .to_vec();

        if image.is_empty() {
            return Err(StorageError::BadRequest("No image file provided.".into()));
        }

        let url = services
            .smms()
            .upload_image(image, file_name, content_type)
            .await
            .map_err(StorageError::Smms)?;

        return Ok(Json(UploadImageResponse { url }));
    }

    Err(StorageError::BadRequest("No image file provided.".into()))
}

#[derive(Debug)]
pub enum StorageError {
    BadRequest(String),
    Multipart(axum::extract::multipart::MultipartError),
    Smms(SmmsError),
}

impl IntoResponse for StorageError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            StorageError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            StorageError::Multipart(error) => (StatusCode::BAD_REQUEST, error.to_string()),
            StorageError::Smms(SmmsError::MissingSecretToken) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SM.MS secret token is not configured.".into(),
            ),
            StorageError::Smms(SmmsError::Request(error)) => (
                StatusCode::BAD_GATEWAY,
                format!("Image upload to SM.MS failed: {error}"),
            ),
            StorageError::Smms(SmmsError::Rejected(message)) => (
                StatusCode::BAD_GATEWAY,
                format!("Image upload to SM.MS failed: {message}"),
            ),
            StorageError::Smms(SmmsError::MissingUrl) => (
                StatusCode::BAD_GATEWAY,
                "Image upload to SM.MS failed: No URL returned".into(),
            ),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}
