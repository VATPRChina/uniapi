use std::collections::{BTreeMap, HashSet};

use sqlx::PgPool;
use uuid::Uuid;

use crate::model::controller_info::{ControllerInfo, ControllerPermission};
use crate::model::user::UserSummary;
use crate::model::user_role::UserRole;
use crate::repository::atc::atc::{AtcControllerPermissionRecord, AtcRepositoryExt};

#[derive(Clone)]
pub struct ControllerInfoService {
    db: PgPool,
}

impl ControllerInfoService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list(&self) -> Result<Vec<ControllerInfo>, ControllerInfoServiceError> {
        let rows = self.db.list_atc_controllers().await?;
        let mut controllers = BTreeMap::<Uuid, ControllerInfo>::new();

        for row in rows {
            let permission = controller_permission(&row)?;
            let controller = controller_info(&row)?;
            controllers
                .entry(row.user_id)
                .or_insert(controller)
                .permissions
                .push(permission);
        }

        Ok(controllers.into_values().collect())
    }
}

fn controller_info(
    row: &AtcControllerPermissionRecord,
) -> Result<ControllerInfo, ControllerInfoServiceError> {
    let direct_roles = row
        .user_roles
        .iter()
        .filter_map(|role| role.parse::<UserRole>().ok())
        .collect::<HashSet<_>>();

    Ok(ControllerInfo {
        user: UserSummary {
            id: row.user_id,
            cid: row.user_cid.clone(),
            full_name: row.user_full_name.clone(),
            email: row.user_email.clone(),
            direct_roles,
            created_at: row.user_created_at,
            updated_at: row.user_updated_at,
        },
        is_visiting: row.is_visiting.unwrap_or(false),
        is_absent: row.is_absent.unwrap_or(false),
        rating: row
            .rating
            .as_deref()
            .unwrap_or("OBS")
            .parse()
            .map_err(|_| {
                ControllerInfoServiceError::InvalidControllerRating(
                    row.rating.clone().unwrap_or_else(|| "OBS".to_owned()),
                )
            })?,
        permissions: Vec::new(),
    })
}

fn controller_permission(
    row: &AtcControllerPermissionRecord,
) -> Result<ControllerPermission, ControllerInfoServiceError> {
    Ok(ControllerPermission {
        position_kind: row.position_kind_id.parse().map_err(|_| {
            ControllerInfoServiceError::InvalidControllerPositionKind(row.position_kind_id.clone())
        })?,
        state: row
            .state
            .parse()
            .map_err(|_| ControllerInfoServiceError::InvalidControllerState(row.state.clone()))?,
        solo_expires_at: row.solo_expires_at,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum ControllerInfoServiceError {
    #[error("failed to query controller information: {0}")]
    Database(#[from] sqlx::Error),
    #[error("invalid controller state {0}")]
    InvalidControllerState(String),
    #[error("invalid controller rating {0}")]
    InvalidControllerRating(String),
    #[error("invalid controller position kind {0}")]
    InvalidControllerPositionKind(String),
}
