use chrono::{DateTime, Utc};
use serde::Serialize;
use ulid::Ulid;

use super::models::{AuditLog, AuditLogEntity};

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AuditLogEntityKindDto {
    Event,
    AtcApplication,
    User,
    UserRole,
    UserAtcPermission,
    EventAtcPosition,
    EventSlot,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AuditLogEntityDto {
    pub kind: AuditLogEntityKindDto,
    pub id: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AuditLogDto {
    pub entity: AuditLogEntityDto,
    pub child_entity: Option<AuditLogEntityDto>,
    pub before: serde_json::Value,
    pub after: serde_json::Value,
    pub operated_by: String,
    pub created_at: DateTime<Utc>,
}

impl From<AuditLogEntity> for (AuditLogEntityDto, Option<AuditLogEntityDto>) {
    fn from(entity: AuditLogEntity) -> Self {
        match entity {
            AuditLogEntity::AtcApplication(id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::AtcApplication,
                    id: Ulid::from(id).to_string(),
                },
                None,
            ),
            AuditLogEntity::Event(id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::Event,
                    id: Ulid::from(id).to_string(),
                },
                None,
            ),
            AuditLogEntity::EventAtcPosition(pid, id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::Event,
                    id: Ulid::from(pid).to_string(),
                },
                Some(AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::EventAtcPosition,
                    id: Ulid::from(id).to_string(),
                }),
            ),
            AuditLogEntity::EventSlot(pid, id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::Event,
                    id: Ulid::from(pid).to_string(),
                },
                Some(AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::EventSlot,
                    id: Ulid::from(id).to_string(),
                }),
            ),
            AuditLogEntity::User(id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::User,
                    id: Ulid::from(id).to_string(),
                },
                None,
            ),
            AuditLogEntity::UserAtcPermission(pid, id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::User,
                    id: Ulid::from(pid).to_string(),
                },
                Some(AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::UserAtcPermission,
                    id: Ulid::from(id).to_string(),
                }),
            ),
            AuditLogEntity::UserRole(pid, id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::User,
                    id: Ulid::from(pid).to_string(),
                },
                Some(AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::UserRole,
                    id: Ulid::from(id).to_string(),
                }),
            ),
        }
    }
}

impl From<AuditLog> for AuditLogDto {
    fn from(audit_log: AuditLog) -> Self {
        let (entity, child_entity) = audit_log.entity.into();

        Self {
            entity,
            child_entity,
            before: audit_log.before,
            after: audit_log.after,
            operated_by: Ulid::from(audit_log.operated_by).to_string(),
            created_at: audit_log.created_at,
        }
    }
}
