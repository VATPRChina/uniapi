#![allow(async_fn_in_trait)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::IntoDeserializer};
use serde_json::Value;
use sqlx::FromRow;
use thiserror::Error;
use uuid::Uuid;

use super::models::{AuditLog, AuditLogEntity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditLogEntityKind {
    Event,
    AtcApplication,
    AtcPosition,
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

pub trait AuditLogRepository<'executor> {
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

    async fn list_audit_log_by_entity_kind_and_key(
        self,
        entity_kind: AuditLogEntityKind,
        entity_key: &str,
    ) -> Result<Vec<AuditLog>, sqlx::Error>;
}

impl<'executor, E> AuditLogRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn create_audit_log(self, audit_log: AuditLog) -> Result<AuditLogRecord, sqlx::Error> {
        let record = AuditLogRecord::from(audit_log);

        sqlx::query_as::<_, AuditLogRecord>(
            r#"
        INSERT INTO public.audit_log (
            entity_kind, entity_id, entity_key, child_entity_kind, child_entity_id,
            before, after, operated_by, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING entity_kind, entity_id, entity_key, child_entity_kind, child_entity_id,
                  before, after, operated_by, created_at
        "#,
        )
        .bind(record.entity_kind)
        .bind(record.entity_id)
        .bind(record.entity_key)
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
            SELECT entity_kind, entity_id, entity_key, child_entity_kind, child_entity_id,
                   before, after, operated_by, created_at
            FROM public.audit_log
            WHERE entity_kind = $1
            ORDER BY created_at DESC
            LIMIT 100
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
            SELECT entity_kind, entity_id, entity_key, child_entity_kind, child_entity_id,
                   before, after, operated_by, created_at
            FROM public.audit_log
            WHERE entity_kind = $1 AND entity_id = $2
            ORDER BY created_at DESC
            LIMIT 100
            "#,
            )
            .bind(entity_kind.to_string())
            .bind(entity_id)
            .fetch_all(self)
            .await?,
        )
    }

    async fn list_audit_log_by_entity_kind_and_key(
        self,
        entity_kind: AuditLogEntityKind,
        entity_key: &str,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        list_records(
            sqlx::query_as::<_, AuditLogRecord>(
                r#"
            SELECT entity_kind, entity_id, entity_key, child_entity_kind, child_entity_id,
                   before, after, operated_by, created_at
            FROM public.audit_log
            WHERE entity_kind = $1 AND entity_key = $2
            ORDER BY created_at DESC
            LIMIT 100
            "#,
            )
            .bind(entity_kind.to_string())
            .bind(entity_key)
            .fetch_all(self)
            .await?,
        )
    }
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct AuditLogRecord {
    pub entity_kind: String,
    pub entity_id: Option<Uuid>,
    pub entity_key: Option<String>,
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
        let (entity_kind, entity_id, entity_key, child_entity_kind, child_entity_id) =
            match audit_log.entity {
                AuditLogEntity::AtcApplication(id) => (
                    AuditLogEntityKind::AtcApplication,
                    Some(id),
                    None,
                    None,
                    None,
                ),
                AuditLogEntity::AtcPosition(key) => {
                    (AuditLogEntityKind::AtcPosition, None, Some(key), None, None)
                }
                AuditLogEntity::Event(id) => {
                    (AuditLogEntityKind::Event, Some(id), None, None, None)
                }
                AuditLogEntity::EventAtcPosition(pid, id) => (
                    AuditLogEntityKind::Event,
                    Some(pid),
                    None,
                    Some(AuditLogEntityKind::EventAtcPosition),
                    Some(id),
                ),
                AuditLogEntity::EventSlot(pid, id) => (
                    AuditLogEntityKind::Event,
                    Some(pid),
                    None,
                    Some(AuditLogEntityKind::EventSlot),
                    Some(id),
                ),
                AuditLogEntity::User(id) => (AuditLogEntityKind::User, Some(id), None, None, None),
                AuditLogEntity::UserAtcPermission(pid, id) => (
                    AuditLogEntityKind::User,
                    Some(pid),
                    None,
                    Some(AuditLogEntityKind::UserAtcPermission),
                    Some(id),
                ),
                AuditLogEntity::UserRole(pid, id) => (
                    AuditLogEntityKind::User,
                    Some(pid),
                    None,
                    Some(AuditLogEntityKind::UserRole),
                    Some(id),
                ),
            };

        Self {
            entity_kind: entity_kind.to_string(),
            entity_id,
            entity_key,
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

        let entity_id = |id: Option<Uuid>| id.ok_or_else(|| incomplete_error());
        let entity = match entity_kind {
            AuditLogEntityKind::Event => AuditLogEntity::Event(entity_id(record.entity_id)?),
            AuditLogEntityKind::AtcApplication => {
                AuditLogEntity::AtcApplication(entity_id(record.entity_id)?)
            }
            AuditLogEntityKind::AtcPosition => {
                AuditLogEntity::AtcPosition(record.entity_key.clone().ok_or_else(incomplete_error)?)
            }
            AuditLogEntityKind::User => AuditLogEntity::User(entity_id(record.entity_id)?),
            AuditLogEntityKind::UserRole => AuditLogEntity::UserRole(
                entity_id(record.entity_id)?,
                record.child_entity_id.ok_or_else(incomplete_error)?,
            ),
            AuditLogEntityKind::UserAtcPermission => AuditLogEntity::UserAtcPermission(
                entity_id(record.entity_id)?,
                record.child_entity_id.ok_or_else(incomplete_error)?,
            ),
            AuditLogEntityKind::EventAtcPosition => AuditLogEntity::EventAtcPosition(
                entity_id(record.entity_id)?,
                record.child_entity_id.ok_or_else(incomplete_error)?,
            ),
            AuditLogEntityKind::EventSlot => AuditLogEntity::EventSlot(
                entity_id(record.entity_id)?,
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
            entity_id: Some(entity_id),
            entity_key: None,
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
        assert_eq!(record.entity_id, Some(event_id));
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
    fn converts_atc_position_entity_in_both_directions() {
        let audit_log = AuditLog {
            entity: AuditLogEntity::AtcPosition("ZBAA_TWR".to_owned()),
            before: Value::Null,
            after: Value::Null,
            operated_by: Uuid::nil(),
            created_at: Utc.with_ymd_and_hms(2026, 8, 29, 0, 0, 0).unwrap(),
        };

        let record = AuditLogRecord::from(audit_log.clone());
        assert_eq!(record.entity_kind, "atc-position");
        assert_eq!(record.entity_id, None);
        assert_eq!(record.entity_key.as_deref(), Some("ZBAA_TWR"));
        assert_eq!(AuditLog::try_from(record).unwrap(), audit_log);
    }

    #[test]
    fn rejects_unknown_database_entity_kind() {
        assert_eq!(
            AuditLog::try_from(record("unknown", Uuid::nil())).unwrap_err(),
            InvalidAuditLogEntityKind("unknown".to_owned())
        );
    }
}
