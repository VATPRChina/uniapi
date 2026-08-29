use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::{AuditLog, AuditLogEntity};
use super::repository::{AuditLogEntityKind, AuditLogRepository};

#[derive(Debug, Clone)]
pub struct AuditLogService {
    db: PgPool,
}

impl AuditLogService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn record<T: Serialize>(
        &self,
        entity: AuditLogEntity,
        operated_by: Uuid,
        before: Option<&T>,
        after: Option<&T>,
    ) -> Result<(), AuditLogServiceError> {
        let before = serialize_snapshot(before)?;
        let after = serialize_snapshot(after)?;

        self.db
            .create_audit_log(AuditLog {
                entity,
                before,
                after,
                operated_by,
                created_at: Utc::now(),
            })
            .await?;

        Ok(())
    }

    pub async fn list_events(&self) -> Result<Vec<AuditLog>, AuditLogServiceError> {
        Ok(self
            .db
            .list_audit_log_by_entity_kind(AuditLogEntityKind::Event)
            .await?)
    }

    pub async fn list_events_by_id(&self, id: Uuid) -> Result<Vec<AuditLog>, AuditLogServiceError> {
        Ok(self
            .db
            .list_audit_log_by_entity_kind_and_id(AuditLogEntityKind::Event, id)
            .await?)
    }

    pub async fn list_atc_applications(&self) -> Result<Vec<AuditLog>, AuditLogServiceError> {
        Ok(self
            .db
            .list_audit_log_by_entity_kind(AuditLogEntityKind::AtcApplication)
            .await?)
    }

    pub async fn list_atc_positions(&self) -> Result<Vec<AuditLog>, AuditLogServiceError> {
        Ok(self
            .db
            .list_audit_log_by_entity_kind(AuditLogEntityKind::AtcPosition)
            .await?)
    }

    pub async fn list_atc_positions_by_callsign(
        &self,
        callsign: &str,
    ) -> Result<Vec<AuditLog>, AuditLogServiceError> {
        Ok(self
            .db
            .list_audit_log_by_entity_kind_and_key(AuditLogEntityKind::AtcPosition, callsign)
            .await?)
    }

    pub async fn list_atc_applications_by_id(
        &self,
        id: Uuid,
    ) -> Result<Vec<AuditLog>, AuditLogServiceError> {
        Ok(self
            .db
            .list_audit_log_by_entity_kind_and_id(AuditLogEntityKind::AtcApplication, id)
            .await?)
    }

    pub async fn list_users(&self) -> Result<Vec<AuditLog>, AuditLogServiceError> {
        Ok(self
            .db
            .list_audit_log_by_entity_kind(AuditLogEntityKind::User)
            .await?)
    }

    pub async fn list_users_by_id(&self, id: Uuid) -> Result<Vec<AuditLog>, AuditLogServiceError> {
        Ok(self
            .db
            .list_audit_log_by_entity_kind_and_id(AuditLogEntityKind::User, id)
            .await?)
    }

    pub async fn list_user_atc_status(
        &self,
        id: Uuid,
    ) -> Result<Vec<AuditLog>, AuditLogServiceError> {
        Ok(self
            .list_users_by_id(id)
            .await?
            .into_iter()
            .filter(|audit_log| matches!(audit_log.entity, AuditLogEntity::UserAtcPermission(_, _)))
            .collect())
    }
}

fn serialize_snapshot<T: Serialize>(
    snapshot: Option<&T>,
) -> Result<serde_json::Value, serde_json::Error> {
    snapshot
        .map(serde_json::to_value)
        .transpose()
        .map(|value| value.unwrap_or_default())
}

#[derive(Debug, thiserror::Error)]
pub enum AuditLogServiceError {
    #[error("failed to serialize audit log snapshot: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("failed to persist audit log: {0}")]
    Database(#[from] sqlx::Error),
}

#[cfg(test)]
mod tests {
    use super::serialize_snapshot;
    use serde::Serialize;
    use serde::ser::{Error, Serializer};
    use serde_json::json;

    #[derive(Debug)]
    struct FailingSnapshot;

    impl Serialize for FailingSnapshot {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(S::Error::custom("snapshot failed"))
        }
    }

    #[test]
    fn serializes_present_snapshot() {
        assert_eq!(serialize_snapshot(Some(&[1, 2])).unwrap(), json!([1, 2]));
    }

    #[test]
    fn serializes_absent_snapshot_as_null() {
        assert_eq!(serialize_snapshot::<[i32; 0]>(None).unwrap(), json!(null));
    }

    #[test]
    fn returns_snapshot_serialization_errors() {
        assert!(serialize_snapshot(Some(&FailingSnapshot)).is_err());
    }
}
