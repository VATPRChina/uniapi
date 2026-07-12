use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::IntoDeserializer};
use serde_json::Value;
use sqlx::FromRow;
use thiserror::Error;
use uuid::Uuid;

use crate::model::audit_log::{AuditLog, AuditLogEntity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditLogEntityKind {
    Event,
    AtcApplication,
    User,
    UserRole,
    UserAtcPermission,
    EventAtcPosition,
    EventSlot,
}

impl std::fmt::Display for AuditLogEntityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.serialize(formatter)
    }
}

impl std::str::FromStr for AuditLogEntityKind {
    type Err = serde::de::value::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::deserialize(value.into_deserializer())
    }
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct AuditLogRecord {
    pub entity_kind: String,
    pub entity_id: Uuid,
    pub child_entity_kind: Option<String>,
    pub child_entity_id: Option<Uuid>,
    pub before: Value,
    pub after: Value,
    pub operated_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid audit log entity kind {0}")]
pub struct InvalidAuditLogEntityKind(pub String);

impl From<AuditLog> for AuditLogRecord {
    fn from(audit_log: AuditLog) -> Self {
        let (entity_kind, entity_id, child_entity_kind, child_entity_id) = match audit_log.entity {
            AuditLogEntity::AtcApplication(id) => {
                (AuditLogEntityKind::AtcApplication, id, None, None)
            }
            AuditLogEntity::Event(id) => (AuditLogEntityKind::Event, id, None, None),
            AuditLogEntity::EventAtcPosition(pid, id) => (
                AuditLogEntityKind::Event,
                pid,
                Some(AuditLogEntityKind::EventAtcPosition),
                Some(id),
            ),
            AuditLogEntity::EventSlot(pid, id) => (
                AuditLogEntityKind::Event,
                pid,
                Some(AuditLogEntityKind::EventSlot),
                Some(id),
            ),
            AuditLogEntity::User(id) => (AuditLogEntityKind::User, id, None, None),
            AuditLogEntity::UserAtcPermission(pid, id) => (
                AuditLogEntityKind::User,
                pid,
                Some(AuditLogEntityKind::UserAtcPermission),
                Some(id),
            ),
            AuditLogEntity::UserRole(pid, id) => (
                AuditLogEntityKind::User,
                pid,
                Some(AuditLogEntityKind::UserRole),
                Some(id),
            ),
        };

        Self {
            entity_kind: entity_kind.to_string(),
            entity_id,
            child_entity_kind: child_entity_kind.as_ref().map(ToString::to_string),
            child_entity_id,
            before: audit_log.before,
            after: audit_log.after,
            operated_by: audit_log.operated_by,
            created_at: audit_log.created_at,
        }
    }
}

impl TryFrom<AuditLogRecord> for AuditLog {
    type Error = InvalidAuditLogEntityKind;

    fn try_from(record: AuditLogRecord) -> Result<Self, Self::Error> {
        let entity_kind: AuditLogEntityKind = record
            .child_entity_kind
            .as_ref()
            .unwrap_or(&record.entity_kind)
            .parse()
            .map_err(|_| InvalidAuditLogEntityKind(record.entity_kind.to_owned()))?;

        let incomplete_error = || InvalidAuditLogEntityKind(format!("incomplete {entity_kind}"));

        let entity = match entity_kind {
            AuditLogEntityKind::Event => AuditLogEntity::Event(record.entity_id),
            AuditLogEntityKind::AtcApplication => AuditLogEntity::AtcApplication(record.entity_id),
            AuditLogEntityKind::User => AuditLogEntity::User(record.entity_id),
            AuditLogEntityKind::UserRole => AuditLogEntity::UserRole(
                record.entity_id,
                record.child_entity_id.ok_or_else(incomplete_error)?,
            ),
            AuditLogEntityKind::UserAtcPermission => AuditLogEntity::UserAtcPermission(
                record.entity_id,
                record.child_entity_id.ok_or_else(incomplete_error)?,
            ),
            AuditLogEntityKind::EventAtcPosition => AuditLogEntity::EventAtcPosition(
                record.entity_id,
                record.child_entity_id.ok_or_else(incomplete_error)?,
            ),
            AuditLogEntityKind::EventSlot => AuditLogEntity::EventSlot(
                record.entity_id,
                record.child_entity_id.ok_or_else(incomplete_error)?,
            ),
        };

        Ok(Self {
            entity,
            before: record.before,
            after: record.after,
            operated_by: record.operated_by,
            created_at: record.created_at,
        })
    }
}

fn list_records(records: Vec<AuditLogRecord>) -> Result<Vec<AuditLog>, sqlx::Error> {
    records
        .into_iter()
        .map(|record| {
            AuditLog::try_from(record).map_err(|error| sqlx::Error::Decode(Box::new(error)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn record(entity_kind: &str, entity_id: Uuid) -> AuditLogRecord {
        AuditLogRecord {
            entity_kind: entity_kind.to_owned(),
            entity_id,
            child_entity_kind: None,
            child_entity_id: None,
            before: Value::Null,
            after: Value::Null,
            operated_by: Uuid::nil(),
            created_at: Utc.with_ymd_and_hms(2026, 6, 13, 4, 0, 0).unwrap(),
        }
    }

    #[test]
    fn converts_business_entity_to_database_columns() {
        let event_id = Uuid::from_u128(1);
        let position_id = Uuid::from_u128(2);
        let audit_log = AuditLog {
            entity: AuditLogEntity::EventAtcPosition(event_id, position_id),
            before: Value::Null,
            after: Value::Null,
            operated_by: Uuid::nil(),
            created_at: Utc.with_ymd_and_hms(2026, 6, 13, 4, 0, 0).unwrap(),
        };

        let record = AuditLogRecord::from(audit_log);

        assert_eq!(record.entity_kind, "event");
        assert_eq!(record.entity_id, event_id);
        assert_eq!(
            record.child_entity_kind.as_deref(),
            Some("event-atc-position")
        );
        assert_eq!(record.child_entity_id, Some(position_id));
    }

    #[test]
    fn converts_database_columns_to_business_entity() {
        let entity_id = Uuid::from_u128(2);
        let mut record = record("user", entity_id);
        record.child_entity_kind = Some("user-atc-permission".to_owned());
        record.child_entity_id = Some(entity_id);

        assert_eq!(
            AuditLog::try_from(record).unwrap(),
            AuditLog {
                entity: AuditLogEntity::UserAtcPermission(entity_id, entity_id),
                before: Value::Null,
                after: Value::Null,
                operated_by: Uuid::nil(),
                created_at: Utc.with_ymd_and_hms(2026, 6, 13, 4, 0, 0).unwrap(),
            }
        );
    }

    #[test]
    fn rejects_unknown_database_entity_kind() {
        assert_eq!(
            AuditLog::try_from(record("unknown", Uuid::nil())).unwrap_err(),
            InvalidAuditLogEntityKind("unknown".to_owned())
        );
    }
}

pub trait AuditLogRepositoryExt<'executor> {
    async fn create_audit_log(self, audit_log: AuditLog) -> Result<AuditLogRecord, sqlx::Error>;

    async fn list_audit_log_by_entity_kind(
        self,
        entity_kind: AuditLogEntityKind,
    ) -> Result<Vec<AuditLog>, sqlx::Error>;

    async fn list_audit_log_by_entity_kind_and_id(
        self,
        entity_kind: AuditLogEntityKind,
        entity_id: Uuid,
    ) -> Result<Vec<AuditLog>, sqlx::Error>;
}

impl<'executor, E> AuditLogRepositoryExt<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn create_audit_log(self, audit_log: AuditLog) -> Result<AuditLogRecord, sqlx::Error> {
        let record = AuditLogRecord::from(audit_log);

        sqlx::query_as::<_, AuditLogRecord>(
            r#"
        INSERT INTO public.audit_log (
            entity_kind, entity_id, child_entity_kind, child_entity_id,
            before, after, operated_by, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING entity_kind, entity_id, child_entity_kind, child_entity_id,
                  before, after, operated_by, created_at
        "#,
        )
        .bind(record.entity_kind)
        .bind(record.entity_id)
        .bind(record.child_entity_kind)
        .bind(record.child_entity_id)
        .bind(record.before)
        .bind(record.after)
        .bind(record.operated_by)
        .bind(record.created_at)
        .fetch_one(self)
        .await
    }
    async fn list_audit_log_by_entity_kind(
        self,
        entity_kind: AuditLogEntityKind,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        list_records(
            sqlx::query_as::<_, AuditLogRecord>(
                r#"
            SELECT entity_kind, entity_id, child_entity_kind, child_entity_id,
                   before, after, operated_by, created_at
            FROM public.audit_log
            WHERE entity_kind = $1
            ORDER BY created_at DESC
            "#,
            )
            .bind(entity_kind.to_string())
            .fetch_all(self)
            .await?,
        )
    }
    async fn list_audit_log_by_entity_kind_and_id(
        self,
        entity_kind: AuditLogEntityKind,
        entity_id: Uuid,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        list_records(
            sqlx::query_as::<_, AuditLogRecord>(
                r#"
            SELECT entity_kind, entity_id, child_entity_kind, child_entity_id,
                   before, after, operated_by, created_at
            FROM public.audit_log
            WHERE entity_kind = $1 AND entity_id = $2
            ORDER BY created_at DESC
            "#,
            )
            .bind(entity_kind.to_string())
            .bind(entity_id)
            .fetch_all(self)
            .await?,
        )
    }
}
