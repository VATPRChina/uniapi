use serde::Serialize;

use super::models::SectorPermission;

#[derive(Serialize, utoipa::ToSchema)]
pub struct SectorPermissionResponse {
    pub has_permission: bool,
    pub sector_type: &'static str,
}

impl From<SectorPermission> for SectorPermissionResponse {
    fn from(permission: SectorPermission) -> Self {
        Self {
            has_permission: permission.has_permission,
            sector_type: "controller",
        }
    }
}
