use std::collections::BTreeMap;

use futures::future::try_join_all;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::modules::audit_log::models::AuditLogEntity;
use crate::modules::audit_log::service::{AuditLogService, AuditLogServiceError};
use crate::modules::user::models::UserSummary;
use crate::modules::user::service::user::{UserService, UserServiceError};

use super::models::{
    Controller, ControllerPermission, ControllerPositionKind, ControllerRating, ControllerSave,
    UserControllerState,
};
use super::repository::controller::{AtcControllerPermissionRecord, ControllerRepository};
use super::repository::user_atc_permission::{AtcPermissionRecord, UserAtcPermissionRepository};
use super::repository::user_atc_status::{
    AtcStatusRecord, UserAtcStatusRepository, UserAtcStatusTransactionRepository,
};

#[derive(Clone)]
pub struct ControllerService {
    db: PgPool,
    audit_log: AuditLogService,
    user: UserService,
}

impl ControllerService {
    pub fn new(db: PgPool, audit_log: AuditLogService, user: UserService) -> Self {
        Self {
            db,
            audit_log,
            user,
        }
    }

    pub async fn list(&self) -> Result<Vec<ControllerView>, ControllerServiceError> {
        let rows = self.db.list_atc_controllers().await?;
        let mut controllers = BTreeMap::<Uuid, Controller>::new();

        for row in rows {
            let permission = controller_permission(&row)?;
            let controller = controller(&row)?;
            controllers
                .entry(row.user_id)
                .or_insert(controller)
                .permissions
                .push(permission);
        }

        let mut views = try_join_all(
            controllers
                .into_values()
                .map(|controller| self.with_user(controller)),
        )
        .await?;
        views.sort_by(|left, right| left.user.cid.cmp(&right.user.cid));
        Ok(views)
    }

    pub async fn find(&self, user_id: Uuid) -> Result<ControllerView, ControllerServiceError> {
        let status = self
            .db
            .find_user_atc_status_by_user_id(user_id)
            .await?
            .ok_or(ControllerServiceError::UserNotFound(user_id))?;
        let permissions = self.db.list_user_atc_permission_by_user_id(user_id).await?;
        self.with_user(controller_from_records(status, permissions)?)
            .await
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        status: ControllerSave,
        operated_by: Uuid,
    ) -> Result<ControllerView, ControllerServiceError> {
        let mut transaction = self.db.begin().await?;
        let before = controller_audit_snapshot(&mut transaction, user_id)
            .await?
            .ok_or(ControllerServiceError::UserNotFound(user_id))?;
        transaction.upsert_user_atc_status(user_id, &status).await?;
        let after = controller_audit_snapshot(&mut transaction, user_id)
            .await?
            .ok_or(ControllerServiceError::UserNotFound(user_id))?;
        transaction.commit().await?;

        self.audit_log
            .record(
                AuditLogEntity::UserAtcPermission(user_id, user_id),
                operated_by,
                Some(&before),
                Some(&after),
            )
            .await?;

        self.find(user_id).await
    }

    pub async fn has_any_permission(&self, user_id: Uuid) -> Result<bool, ControllerServiceError> {
        Ok(self
            .db
            .has_user_atc_permission_any_by_user_id(user_id)
            .await?)
    }

    pub async fn has_mentor_permission(
        &self,
        user_id: Uuid,
    ) -> Result<bool, ControllerServiceError> {
        Ok(self
            .db
            .has_user_atc_permission_mentor_by_user_id(user_id)
            .await?)
    }

    async fn with_user(
        &self,
        controller: Controller,
    ) -> Result<ControllerView, ControllerServiceError> {
        let user = self
            .user
            .find_summary_by_id(controller.user_id)
            .await?
            .ok_or(ControllerServiceError::UserNotFound(controller.user_id))?;
        Ok(ControllerView { controller, user })
    }
}

#[derive(Debug)]
pub struct ControllerView {
    pub controller: Controller,
    pub user: UserSummary,
}

fn controller(row: &AtcControllerPermissionRecord) -> Result<Controller, ControllerServiceError> {
    Ok(Controller {
        user_id: row.user_id,
        is_visiting: row.is_visiting.unwrap_or(false),
        is_absent: row.is_absent.unwrap_or(false),
        rating: parse_rating(row.rating.as_deref())?,
        permissions: Vec::new(),
    })
}

fn controller_permission(
    row: &AtcControllerPermissionRecord,
) -> Result<ControllerPermission, ControllerServiceError> {
    Ok(ControllerPermission {
        position_kind: parse_position_kind(&row.position_kind_id)?,
        state: parse_controller_state(&row.state)?,
        solo_expires_at: row.solo_expires_at,
    })
}

fn controller_from_records(
    status: AtcStatusRecord,
    permissions: Vec<AtcPermissionRecord>,
) -> Result<Controller, ControllerServiceError> {
    Ok(Controller {
        user_id: status.user_id,
        is_visiting: status.is_visiting.unwrap_or(false),
        is_absent: status.is_absent.unwrap_or(false),
        rating: parse_rating(status.rating.as_deref())?,
        permissions: permissions
            .into_iter()
            .map(|permission| {
                Ok(ControllerPermission {
                    position_kind: parse_position_kind(&permission.position_kind_id)?,
                    state: parse_controller_state(&permission.state)?,
                    solo_expires_at: permission.solo_expires_at,
                })
            })
            .collect::<Result<_, ControllerServiceError>>()?,
    })
}

fn parse_rating(rating: Option<&str>) -> Result<ControllerRating, ControllerServiceError> {
    let rating = rating.unwrap_or("OBS");
    rating
        .parse()
        .map_err(|_| ControllerServiceError::InvalidControllerRating(rating.to_owned()))
}

fn parse_position_kind(
    position_kind: &str,
) -> Result<ControllerPositionKind, ControllerServiceError> {
    position_kind.parse().map_err(|_| {
        ControllerServiceError::InvalidControllerPositionKind(position_kind.to_owned())
    })
}

fn parse_controller_state(state: &str) -> Result<UserControllerState, ControllerServiceError> {
    state
        .parse()
        .map_err(|_| ControllerServiceError::InvalidControllerState(state.to_owned()))
}

#[derive(serde::Serialize)]
struct ControllerAuditSnapshot {
    status: AtcStatusRecord,
    permissions: Vec<AtcPermissionRecord>,
}

async fn controller_audit_snapshot(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<Option<ControllerAuditSnapshot>, sqlx::Error> {
    let Some(status) = (&mut **transaction)
        .find_user_atc_status_by_user_id_for_update(user_id)
        .await?
    else {
        return Ok(None);
    };
    let permissions = (&mut **transaction)
        .list_user_atc_permission_by_user_id(user_id)
        .await?;

    Ok(Some(ControllerAuditSnapshot {
        status,
        permissions,
    }))
}

#[derive(Debug, thiserror::Error)]
pub enum ControllerServiceError {
    #[error("user {0} not found")]
    UserNotFound(Uuid),
    #[error("failed to query controller information: {0}")]
    Database(#[from] sqlx::Error),
    #[error("invalid controller state {0}")]
    InvalidControllerState(String),
    #[error("invalid controller rating {0}")]
    InvalidControllerRating(String),
    #[error("invalid controller position kind {0}")]
    InvalidControllerPositionKind(String),
    #[error("failed to access controller user: {0}")]
    User(#[from] UserServiceError),
    #[error("failed to record controller audit log: {0}")]
    AuditLog(#[from] AuditLogServiceError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_controller_state() {
        assert!(matches!(
            parse_controller_state("student"),
            Err(ControllerServiceError::InvalidControllerState(value)) if value == "student"
        ));
    }
}
