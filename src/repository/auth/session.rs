use chrono::{DateTime, Utc};
use sqlx::FromRow;
use ulid::Ulid;
use uuid::Uuid;

#[derive(FromRow)]
pub struct RefreshSessionRow {
    #[sqlx(try_from = "Uuid")]
    pub token: Ulid,
    pub user_id: Uuid,
    pub user_updated_at: DateTime<Utc>,
    pub expires_in: DateTime<Utc>,
    #[allow(dead_code)]
    pub code: Option<Uuid>,
    pub client_id: String,
    pub updated_at: DateTime<Utc>,
}

pub struct RefreshSessionIssue {
    pub token: Ulid,
    pub code: Option<Ulid>,
}

pub trait SessionTransactionExt {
    async fn issue_session_refresh_token(
        &mut self,
        user_id: Uuid,
        user_updated_at: DateTime<Utc>,
        expires_in: DateTime<Utc>,
        client_id: &str,
        old_token: Option<Ulid>,
        create_code: bool,
    ) -> Result<RefreshSessionIssue, sqlx::Error>;
}

impl SessionTransactionExt for sqlx::Transaction<'_, sqlx::Postgres> {
    async fn issue_session_refresh_token(
        &mut self,
        user_id: Uuid,
        user_updated_at: DateTime<Utc>,
        expires_in: DateTime<Utc>,
        client_id: &str,
        old_token: Option<Ulid>,
        create_code: bool,
    ) -> Result<RefreshSessionIssue, sqlx::Error> {
        tracing::info!(
            operation = "issue_refresh_token",
            repository = "src/repository/auth/session.rs",
            "modifying data"
        );

        let token = Ulid::new();
        let code = create_code.then(Ulid::new);

        sqlx::query(
            r#"
        INSERT INTO session (token, user_id, user_updated_at, expires_in, code, client_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        )
        .bind(Uuid::from(token))
        .bind(user_id)
        .bind(user_updated_at)
        .bind(expires_in)
        .bind(code.map(Uuid::from))
        .bind(client_id)
        .execute(&mut **self)
        .await?;

        if let Some(old_token) = old_token {
            sqlx::query("DELETE FROM session WHERE token = $1")
                .bind(Uuid::from(old_token))
                .execute(&mut **self)
                .await?;
        }

        sqlx::query("DELETE FROM session WHERE user_id = $1 AND expires_in < now()")
            .bind(user_id)
            .execute(&mut **self)
            .await?;
        sqlx::query(
            r#"
        DELETE FROM session
        WHERE user_id = $1
          AND user_updated_at <> (SELECT updated_at FROM "user" WHERE id = $1)
        "#,
        )
        .bind(user_id)
        .execute(&mut **self)
        .await?;

        Ok(RefreshSessionIssue { token, code })
    }
}

pub trait SessionRepositoryExt<'executor> {
    async fn find_session(self, token: Ulid) -> Result<Option<RefreshSessionRow>, sqlx::Error>;

    async fn find_session_by_code(
        self,
        code: Ulid,
    ) -> Result<Option<RefreshSessionRow>, sqlx::Error>;

    async fn clear_session_code(self, code: Ulid) -> Result<(), sqlx::Error>;

    async fn delete_session(self, token: Ulid) -> Result<bool, sqlx::Error>;
}

impl<'executor, E> SessionRepositoryExt<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn find_session(self, token: Ulid) -> Result<Option<RefreshSessionRow>, sqlx::Error> {
        sqlx::query_as::<_, RefreshSessionRow>(
            r#"
        SELECT session.token, session.user_id, session.user_updated_at, session.expires_in,
               session.code, session.client_id, "user".updated_at
        FROM session
        JOIN "user" ON "user".id = session.user_id
        WHERE session.token = $1
        "#,
        )
        .bind(Uuid::from(token))
        .fetch_optional(self)
        .await
    }
    async fn find_session_by_code(
        self,
        code: Ulid,
    ) -> Result<Option<RefreshSessionRow>, sqlx::Error> {
        sqlx::query_as::<_, RefreshSessionRow>(
            r#"
        SELECT session.token, session.user_id, session.user_updated_at, session.expires_in,
               session.code, session.client_id, "user".updated_at
        FROM session
        JOIN "user" ON "user".id = session.user_id
        WHERE session.code = $1
        "#,
        )
        .bind(Uuid::from(code))
        .fetch_optional(self)
        .await
    }
    async fn clear_session_code(self, code: Ulid) -> Result<(), sqlx::Error> {
        tracing::info!(
            operation = "clear_code",
            repository = "src/repository/auth/session.rs",
            "modifying data"
        );

        sqlx::query("UPDATE session SET code = NULL WHERE code = $1")
            .bind(Uuid::from(code))
            .execute(self)
            .await?;

        Ok(())
    }
    async fn delete_session(self, token: Ulid) -> Result<bool, sqlx::Error> {
        tracing::info!(
            operation = "delete",
            repository = "src/repository/auth/session.rs",
            "modifying data"
        );

        let result = sqlx::query("DELETE FROM session WHERE token = $1")
            .bind(Uuid::from(token))
            .execute(self)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
