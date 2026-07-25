use serde::Serialize;
use ulid::Ulid;
use uuid::Uuid;

use crate::routes::ApiError;

pub fn parse_ulid_uuid(field: &'static str, id: &str) -> Result<Uuid, ApiError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| ApiError::bad_request(field, "invalid ULID"))
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UploadImageResponse {
    pub url: String,
}
