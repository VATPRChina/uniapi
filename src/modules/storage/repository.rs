use crate::adapter::smms::{SmmsClient, SmmsError};

pub(crate) trait ImageRepository {
    async fn upload_image(
        &self,
        image: Vec<u8>,
        file_name: Option<String>,
        content_type: Option<String>,
    ) -> Result<String, SmmsError>;
}

impl ImageRepository for SmmsClient {
    async fn upload_image(
        &self,
        image: Vec<u8>,
        file_name: Option<String>,
        content_type: Option<String>,
    ) -> Result<String, SmmsError> {
        SmmsClient::upload_image(self, image, file_name, content_type).await
    }
}
