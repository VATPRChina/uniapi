use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use ulid::Ulid;
use uuid::Uuid;

use super::super::models::{IssuedRefreshToken, RefreshToken};
use super::super::repository::refresh_token::{RefreshTokenRepository, RefreshTokenTransaction};

#[derive(Clone)]
pub struct RefreshTokenService {
    db: PgPool,
    expires_days: i64,
}

impl RefreshTokenService {
    pub fn new(db: PgPool, expires_days: i64) -> Self {
        Self { db, expires_days }
    }

    pub async fn issue(
        &self,
        user_id: Uuid,
        user_updated_at: DateTime<Utc>,
        client_id: &str,
        old_token: Option<Ulid>,
        create_code: bool,
    ) -> Result<IssuedRefreshToken, sqlx::Error> {
        let expires_at = Utc::now() + Duration::days(self.expires_days);
        let mut transaction = self.db.begin().await?;
        let issue = transaction
            .issue_session_refresh_token(
                user_id,
                user_updated_at,
                expires_at,
                client_id,
                old_token,
                create_code,
            )
            .await?;
        transaction.commit().await?;

        Ok(issue)
    }

    pub async fn find(&self, token: Ulid) -> Result<Option<RefreshToken>, sqlx::Error> {
        self.db.find_session(token).await
    }

    pub async fn find_by_code(&self, code: Ulid) -> Result<Option<RefreshToken>, sqlx::Error> {
        self.db.find_session_by_code(code).await
    }

    pub async fn clear_code(&self, code: Ulid) -> Result<(), sqlx::Error> {
        self.db.clear_session_code(code).await
    }

    pub async fn delete(&self, token: Ulid) -> Result<bool, sqlx::Error> {
        self.db.delete_session(token).await
    }
}
