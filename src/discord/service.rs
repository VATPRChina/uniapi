use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::DiscordSyncStatus;
use super::{DiscordEventPublisher, DiscordPublishError};
use crate::modules::audit_log::models::AuditLogEntity;
use crate::modules::audit_log::service::{AuditLogService, AuditLogServiceError};
use crate::modules::event::models::Event;
use crate::modules::event::repository::event::EventRepository;
use crate::settings::Discord;

#[derive(Clone)]
pub struct DiscordService {
    db: PgPool,
    audit_log: AuditLogService,
    publisher: Option<DiscordEventPublisher>,
}

impl DiscordService {
    pub fn new(db: PgPool, audit_log: AuditLogService, settings: &Discord) -> Self {
        Self {
            db,
            audit_log,
            publisher: DiscordEventPublisher::from_settings(settings),
        }
    }

    pub async fn publish_event(
        &self,
        id: Uuid,
        operated_by: Uuid,
    ) -> Result<Event, DiscordServiceError> {
        let mut event = self
            .db
            .find_event_by_id(id)
            .await?
            .ok_or(DiscordServiceError::EventNotFound(id))?;
        let before = event.clone();
        let result = self.sync_event(&mut event).await;
        self.audit_log
            .record(
                AuditLogEntity::Event(id),
                operated_by,
                Some(&before),
                Some(&event),
            )
            .await?;
        result?;
        Ok(event)
    }

    /// Website changes are already saved; no transaction is held during Discord requests.
    pub(crate) async fn sync_event(&self, event: &mut Event) -> Result<(), DiscordServiceError> {
        let result = match &self.publisher {
            Some(publisher) => publisher.sync(event).await,
            None => Err(DiscordPublishError::NotConfigured),
        };
        if let Err(error) = &result {
            tracing::error!(%error, id = %event.id, "failed to synchronize Discord event");
        }
        if let Some(message) = &mut event.discord_message {
            message.status = if result.is_ok() {
                DiscordSyncStatus::Sync
            } else {
                DiscordSyncStatus::OutOfSync
            };
            if result.is_ok() {
                message.synced_at = Utc::now();
            }
            sqlx::query("INSERT INTO public.event_discord_message (event_id, guild_id, message_id, status, synced_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (event_id) DO UPDATE SET guild_id = EXCLUDED.guild_id,
                    message_id = EXCLUDED.message_id, status = EXCLUDED.status, synced_at = EXCLUDED.synced_at")
                .bind(event.id).bind(&message.guild_id).bind(&message.message_id)
                .bind(message.status).bind(message.synced_at).execute(&self.db).await?;
        }
        result.map_err(DiscordServiceError::Publish)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DiscordServiceError {
    #[error("event {0} not found")]
    EventNotFound(Uuid),
    #[error(transparent)]
    Publish(#[from] DiscordPublishError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    AuditLog(#[from] AuditLogServiceError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unconfigured_publisher_returns_typed_error_without_creating_a_message() {
        let db = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://localhost/unused")
            .unwrap();
        let service = DiscordService {
            audit_log: AuditLogService::new(db.clone()),
            db,
            publisher: None,
        };
        let mut event = crate::discord::event_post_tests::example_event();
        assert!(matches!(
            service.sync_event(&mut event).await,
            Err(DiscordServiceError::Publish(
                DiscordPublishError::NotConfigured
            ))
        ));
        assert!(event.discord_message.is_none());
    }
}
