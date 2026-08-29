use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::audit_log::models::AuditLogEntity;
use crate::modules::audit_log::service::{AuditLogService, AuditLogServiceError};

use super::models::{AtcPosition, AtcPositionSave};
use super::repository::AtcPositionRepository;

#[derive(Debug, Clone)]
pub struct AtcPositionService {
    db: PgPool,
    audit_log: AuditLogService,
}

impl AtcPositionService {
    pub fn new(db: PgPool, audit_log: AuditLogService) -> Self {
        Self { db, audit_log }
    }

    pub async fn list(&self) -> Result<Vec<AtcPosition>, AtcPositionServiceError> {
        Ok(self.db.list_atc_positions().await?)
    }

    pub async fn find(&self, callsign: &str) -> Result<AtcPosition, AtcPositionServiceError> {
        self.db
            .find_atc_position(callsign)
            .await?
            .ok_or_else(|| AtcPositionServiceError::NotFound(callsign.to_owned()))
    }

    pub async fn create(
        &self,
        position: AtcPositionSave,
        operated_by: Uuid,
    ) -> Result<AtcPosition, AtcPositionServiceError> {
        let position = self.db.create_atc_position(position).await?;
        self.audit_log
            .record(
                AuditLogEntity::AtcPosition(position.callsign.clone()),
                operated_by,
                None,
                Some(&position),
            )
            .await?;
        Ok(position)
    }

    pub async fn update(
        &self,
        callsign: &str,
        position: AtcPositionSave,
        operated_by: Uuid,
    ) -> Result<AtcPosition, AtcPositionServiceError> {
        let mut transaction = self.db.begin().await?;
        let before = (&mut *transaction)
            .find_atc_position_for_update(callsign)
            .await?
            .ok_or_else(|| AtcPositionServiceError::NotFound(callsign.to_owned()))?;
        let position = (&mut *transaction)
            .update_atc_position(callsign, position)
            .await?
            .ok_or_else(|| AtcPositionServiceError::NotFound(callsign.to_owned()))?;
        transaction.commit().await?;
        self.audit_log
            .record(
                AuditLogEntity::AtcPosition(callsign.to_owned()),
                operated_by,
                Some(&before),
                Some(&position),
            )
            .await?;
        Ok(position)
    }

    pub async fn delete(
        &self,
        callsign: &str,
        operated_by: Uuid,
    ) -> Result<(), AtcPositionServiceError> {
        let mut transaction = self.db.begin().await?;
        let before = (&mut *transaction)
            .find_atc_position_for_update(callsign)
            .await?
            .ok_or_else(|| AtcPositionServiceError::NotFound(callsign.to_owned()))?;
        if !(&mut *transaction).delete_atc_position(callsign).await? {
            return Err(AtcPositionServiceError::NotFound(callsign.to_owned()));
        }
        transaction.commit().await?;
        self.audit_log
            .record(
                AuditLogEntity::AtcPosition(callsign.to_owned()),
                operated_by,
                Some(&before),
                None,
            )
            .await?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AtcPositionServiceError {
    #[error("ATC position {0} not found")]
    NotFound(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("audit log error: {0}")]
    AuditLog(#[from] AuditLogServiceError),
}
