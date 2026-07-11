use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditLogEntity {
    AtcApplication(Uuid),
    Event(Uuid),
    EventAtcPosition(Uuid, Uuid),
    EventSlot(Uuid, Uuid),
    User(Uuid),
    UserAtcPermission(Uuid, Uuid),
    UserRole(Uuid, Uuid),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditLog {
    pub entity: AuditLogEntity,
    pub before: Value,
    pub after: Value,
    pub operated_by: Uuid,
    pub created_at: DateTime<Utc>,
}
