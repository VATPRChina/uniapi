use serde::Serialize;

use super::models::UploadedImage;

#[derive(Serialize, utoipa::ToSchema)]
pub struct UploadImageResponse {
    pub url: String,
}

impl From<UploadedImage> for UploadImageResponse {
    fn from(image: UploadedImage) -> Self {
        Self { url: image.url }
    }
}
