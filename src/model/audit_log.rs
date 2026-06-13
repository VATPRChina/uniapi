use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum AuditLogEntity {
    Event(Uuid),
    AtcApplication(Uuid),
    UserRole(Uuid),
    UserAtcPermission(Uuid),
    EventAtcPosition(Uuid),
    EventSlot(Uuid),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AuditLog {
    pub entity: AuditLogEntity,
    pub before: Value,
    pub after: Value,
    pub operated_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use serde_json::json;

    use super::*;

    #[test]
    fn serializes_audit_log() {
        let entity_id = Uuid::nil();
        let operated_by = Uuid::from_u128(1);
        let created_at = Utc.with_ymd_and_hms(2026, 6, 13, 4, 0, 0).unwrap();
        let audit_log = AuditLog {
            entity: AuditLogEntity::Event(entity_id),
            before: Value::Null,
            after: json!({"title": "VATPRC Event"}),
            operated_by,
            created_at,
        };

        assert_eq!(
            serde_json::to_value(audit_log).unwrap(),
            json!({
                "entity": {"Event": entity_id},
                "before": null,
                "after": {"title": "VATPRC Event"},
                "operated_by": operated_by,
                "created_at": "2026-06-13T04:00:00Z"
            })
        );
    }

    #[test]
    fn deserializes_audit_log_entity() {
        let entity_id = Uuid::from_u128(2);

        assert_eq!(
            serde_json::from_value::<AuditLogEntity>(json!({
                "UserAtcPermission": entity_id
            }))
            .unwrap(),
            AuditLogEntity::UserAtcPermission(entity_id)
        );
    }
}
