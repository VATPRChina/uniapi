use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Serialize;

use crate::{
    adapter::smms::SmmsError, auth::CurrentUser, models::user_role::UserRole, services::Services,
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(upload_image))]
pub(crate) struct ApiDoc;

pub fn build_storage_routes() -> Router<Services> {
    Router::new().route("/images", post(upload_image))
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UploadImageResponse {
    url: String,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ErrorResponse {
    message: String,
}

#[utoipa::path(post, path = "api/storage/images", tag = "Storage", security(("bearerAuth" = [])), request_body(content = String, content_type = "multipart/form-data"), responses((status = 200, description = "Successful response", body = UploadImageResponse)))]
async fn upload_image(
    State(services): State<Services>,
    current_user: CurrentUser,
    mut multipart: Multipart,
) -> Result<Json<UploadImageResponse>, StorageError> {
    if !current_user.has_role(UserRole::Volunteer) {
        return Err(StorageError::Forbidden);
    }
    tracing::debug!(
        subject = %current_user.subject,
        user_id = ?current_user.user_id,
        roles = ?current_user.roles().collect::<Vec<_>>(),
        "authenticated storage image upload"
    );

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
    Forbidden,
    Multipart(axum::extract::multipart::MultipartError),
    Smms(SmmsError),
}

impl IntoResponse for StorageError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            StorageError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            StorageError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            StorageError::Multipart(error) => (StatusCode::BAD_REQUEST, error.to_string()),
            StorageError::Smms(SmmsError::MissingSecretToken) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                SmmsError::MissingSecretToken.to_string(),
            ),
            StorageError::Smms(error) => (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Image upload to SM.MS failed: {error}"),
            ),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}
