use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::routing::post;
use axum::{Json, Router};

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::user_role::UserRole;
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(upload_image))]
pub(crate) struct ApiDoc;

pub fn build_storage_routes() -> Router<Services> {
    Router::new()
        .route("/images", post(upload_image))
        .layer(DefaultBodyLimit::max(10_000_000))
}

#[utoipa::path(post, path = "api/storage/images", tag = "Storage", security(("oauth2" = [])), request_body(content = String, content_type = "multipart/form-data"), responses((status = 200, description = "Successful response", body = UploadImageResponse)))]
async fn upload_image(
    State(services): State<Services>,
    current_user: CurrentUser,
    mut multipart: Multipart,
) -> Result<Json<UploadImageResponse>, ApiError> {
    current_user.require_role(UserRole::Volunteer)?;
    tracing::debug!(
        subject = %current_user.subject,
        user_id = ?current_user.user_id,
        roles = ?current_user.roles().collect::<Vec<_>>(),
        "authenticated storage image upload"
    );

    while let Some(field) = multipart.next_field().await? {
        if field.name() != Some("image") {
            continue;
        }

        let file_name = field.file_name().map(ToOwned::to_owned);
        let content_type = field.content_type().map(ToOwned::to_owned);
        let image = field.bytes().await?.to_vec();

        if image.is_empty() {
            return Err(ApiError::bad_request("image", "file is empty"));
        }

        let url = services
            .smms()
            .upload_image(image, file_name, content_type)
            .await?;

        return Ok(Json(UploadImageResponse { url }));
    }

    Err(ApiError::bad_request("image", "image not included"))
}
