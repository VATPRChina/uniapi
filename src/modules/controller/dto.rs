use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::error::ApiError;
use crate::modules::controller::models::Controller;
use crate::modules::user::dto::UserDto;
use crate::modules::user::models::UserSummary;

use super::models::{
    ControllerPermission, ControllerPermissionSave, ControllerRating, ControllerSave,
    UserControllerState,
};

#[derive(Serialize, utoipa::ToSchema)]
pub struct SectorPermissionResponse {
    pub has_permission: bool,
    pub sector_type: &'static str,
}

impl SectorPermissionResponse {
    pub fn new(has_permission: bool) -> Self {
        Self {
            has_permission,
            sector_type: "controller",
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AtcStatusRequest {
    pub is_visiting: bool,
    pub is_absent: bool,
    pub rating: String,
    pub permissions: Vec<AtcPermissionRequest>,
}

impl TryFrom<AtcStatusRequest> for ControllerSave {
    type Error = ApiError;

    fn try_from(request: AtcStatusRequest) -> Result<Self, Self::Error> {
        if request.rating.parse::<ControllerRating>().is_err() {
            return Err(ApiError::bad_request("rating", "invalid ATC rating"));
        }
        if request.permissions.iter().any(|permission| {
            permission.state == UserControllerState::Solo && permission.solo_expires_at.is_none()
        }) {
            return Err(ApiError::SoloExpirationNotProvided);
        }
        if request.is_absent
            && request.permissions.iter().any(|permission| {
                permission.state.to_db_value() > UserControllerState::UnderMentor.to_db_value()
            })
        {
            return Err(ApiError::bad_request(
                "permissions",
                "absent users cannot have ATC permission higher than under mentor",
            ));
        }

        Ok(Self {
            is_visiting: request.is_visiting,
            is_absent: request.is_absent,
            rating: request.rating,
            permissions: request
                .permissions
                .into_iter()
                .map(ControllerPermissionSave::from)
                .collect(),
        })
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AtcPermissionRequest {
    pub position_kind_id: String,
    pub state: UserControllerState,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

impl From<AtcPermissionRequest> for ControllerPermissionSave {
    fn from(permission: AtcPermissionRequest) -> Self {
        Self {
            position_kind_id: permission.position_kind_id,
            state: permission.state.as_db_str().to_owned(),
            solo_expires_at: permission.solo_expires_at,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AtcStatusDto {
    pub user_id: String,
    pub user: UserDto,
    pub is_visiting: bool,
    pub is_absent: bool,
    pub rating: String,
    pub permissions: Vec<AtcPermissionDto>,
}

impl From<(Controller, UserSummary)> for AtcStatusDto {
    fn from((controller, user): (Controller, UserSummary)) -> Self {
        Self {
            user_id: Ulid::from(controller.user_id).to_string(),
            user: UserDto::from_user_summary(user, true),
            is_visiting: controller.is_visiting,
            is_absent: controller.is_absent,
            rating: controller.rating.to_string(),
            permissions: controller.permissions.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AtcPermissionDto {
    pub position_kind_id: String,
    pub state: UserControllerState,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

impl From<ControllerPermission> for AtcPermissionDto {
    fn from(permission: ControllerPermission) -> Self {
        Self {
            position_kind_id: permission.position_kind.to_string(),
            state: permission.state,
            solo_expires_at: permission.solo_expires_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atc_status_request(is_absent: bool, state: UserControllerState) -> AtcStatusRequest {
        AtcStatusRequest {
            is_visiting: false,
            is_absent,
            rating: "S2".to_owned(),
            permissions: vec![AtcPermissionRequest {
                position_kind_id: "TWR".to_owned(),
                state,
                solo_expires_at: Some(Utc::now()),
            }],
        }
    }

    #[test]
    fn absent_user_cannot_have_permission_higher_than_under_mentor() {
        for state in [
            UserControllerState::Solo,
            UserControllerState::Certified,
            UserControllerState::Mentor,
        ] {
            assert!(matches!(
                ControllerSave::try_from(atc_status_request(true, state)),
                Err(ApiError::BadRequest { field, .. }) if field == "permissions"
            ));
        }
    }

    #[test]
    fn absent_user_can_have_permission_up_to_under_mentor() {
        for state in [
            UserControllerState::Student,
            UserControllerState::UnderMentor,
        ] {
            assert!(ControllerSave::try_from(atc_status_request(true, state)).is_ok());
        }
    }

    #[test]
    fn non_absent_user_can_have_permission_higher_than_under_mentor() {
        assert!(
            ControllerSave::try_from(atc_status_request(false, UserControllerState::Mentor))
                .is_ok()
        );
    }
}
