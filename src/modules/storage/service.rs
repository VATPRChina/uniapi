use crate::adapter::smms::{SmmsClient, SmmsError};

use super::models::UploadedImage;
use super::repository::ImageRepository;

#[derive(Clone)]
pub struct StorageService {
    images: SmmsClient,
}

impl StorageService {
    pub fn new(images: SmmsClient) -> Self {
        Self { images }
    }

    pub async fn upload_image(
        &self,
        image: Vec<u8>,
        file_name: Option<String>,
        content_type: Option<String>,
    ) -> Result<UploadedImage, StorageServiceError> {
        Ok(UploadedImage {
            url: ImageRepository::upload_image(&self.images, image, file_name, content_type)
                .await?,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageServiceError {
    #[error("failed to upload image: {0}")]
    Images(#[from] SmmsError),
}
