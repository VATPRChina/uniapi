use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct DiscordMessage {
    /// Discord snowflake serialized as a string to preserve precision in JavaScript.
    pub guild_id: String,
    /// The forum starter message and thread share this ID.
    pub message_id: String,
    pub status: DiscordSyncStatus,
    /// Time of the most recent successful synchronization.
    pub synced_at: DateTime<Utc>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "text")]
pub enum DiscordSyncStatus {
    OutOfSync,
    Sync,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_message_decodes_text_status_and_snowflakes() {
        let value = serde_json::json!({
            "event_id": "00000000-0000-0000-0000-000000000000",
            "guild_id": "123456789012345678",
            "message_id": "987654321098765432",
            "status": "OutOfSync",
            "synced_at": "2026-09-08T00:00:00+00:00"
        });
        let message: DiscordMessage = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(message.status, DiscordSyncStatus::OutOfSync);
        assert_eq!(message.message_id, "987654321098765432");
        assert_eq!(
            serde_json::to_value(message).unwrap()["status"],
            "OutOfSync"
        );
        let mut invalid = value;
        invalid["status"] = serde_json::json!(200);
        assert!(serde_json::from_value::<DiscordMessage>(invalid).is_err());
    }
}
