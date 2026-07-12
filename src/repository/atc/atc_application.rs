use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AtcApplicationRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_cid: String,
    pub user_full_name: String,
    pub user_email: Option<String>,
    pub user_created_at: DateTime<Utc>,
    pub user_updated_at: DateTime<Utc>,
    pub user_roles: Vec<String>,
    pub application_filing_id: Uuid,
    pub review_filing_id: Option<Uuid>,
    pub applied_at: DateTime<Utc>,
    pub status: String,
}

fn application_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT atc_application.id,
               atc_application.user_id,
               "user".cid AS user_cid,
               "user".full_name AS user_full_name,
               "user".email AS user_email,
               "user".created_at AS user_created_at,
               "user".updated_at AS user_updated_at,
               "user".roles AS user_roles,
               atc_application.application_filing_id,
               atc_application.review_filing_id,
               atc_application.applied_at,
               atc_application.status
        FROM public.atc_application
        INNER JOIN public."user" ON "user".id = atc_application.user_id
        {where_clause}
        "#
    )
}

pub trait AtcApplicationRepositoryExt<'executor> {
    async fn list_atc_application(self) -> Result<Vec<AtcApplicationRecord>, sqlx::Error>;

    async fn find_atc_application_by_id(
        self,
        id: Uuid,
    ) -> Result<Option<AtcApplicationRecord>, sqlx::Error>;

    async fn find_atc_application_by_id_for_update(
        self,
        id: Uuid,
    ) -> Result<Option<AtcApplicationRecord>, sqlx::Error>;

    async fn count_atc_application_active_by_user(self, user_id: Uuid) -> Result<i64, sqlx::Error>;

    async fn create_atc_application(
        self,
        user_id: Uuid,
        application_filing_id: Uuid,
    ) -> Result<AtcApplicationRecord, sqlx::Error>;

    async fn set_atc_application_review(
        self,
        id: Uuid,
        status: &str,
        review_filing_id: Uuid,
    ) -> Result<Option<AtcApplicationRecord>, sqlx::Error>;
}

impl<'executor, E> AtcApplicationRepositoryExt<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_atc_application(self) -> Result<Vec<AtcApplicationRecord>, sqlx::Error> {
        sqlx::query_as::<_, AtcApplicationRecord>(&application_select_sql(
            r#"
        ORDER BY atc_application.applied_at DESC
        "#,
        ))
        .fetch_all(self)
        .await
    }
    async fn find_atc_application_by_id(
        self,
        id: Uuid,
    ) -> Result<Option<AtcApplicationRecord>, sqlx::Error> {
        sqlx::query_as::<_, AtcApplicationRecord>(&application_select_sql(
            r#"
        WHERE atc_application.id = $1
        "#,
        ))
        .bind(id)
        .fetch_optional(self)
        .await
    }
    async fn find_atc_application_by_id_for_update(
        self,
        id: Uuid,
    ) -> Result<Option<AtcApplicationRecord>, sqlx::Error> {
        sqlx::query_as::<_, AtcApplicationRecord>(&application_select_sql(
            r#"
        WHERE atc_application.id = $1
        FOR UPDATE OF atc_application
        "#,
        ))
        .bind(id)
        .fetch_optional(self)
        .await
    }
    async fn count_atc_application_active_by_user(self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
        SELECT COUNT(*)
        FROM public.atc_application
        WHERE user_id = $1 AND status != 'Rejected'
        "#,
        )
        .bind(user_id)
        .fetch_one(self)
        .await
    }
    async fn create_atc_application(
        self,
        user_id: Uuid,
        application_filing_id: Uuid,
    ) -> Result<AtcApplicationRecord, sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "src/repository/atc/atc_application.rs",
            "modifying data"
        );

        let id = Uuid::from(Ulid::new());
        sqlx::query_as::<_, AtcApplicationRecord>(
            r#"
        WITH inserted_application AS (
            INSERT INTO public.atc_application (
                id, user_id, application_filing_id, applied_at, status
            )
            VALUES ($1, $2, $3, $4, 'Submitted')
            RETURNING id, user_id, application_filing_id, review_filing_id,
                      applied_at, status
        )
        SELECT inserted_application.id,
               inserted_application.user_id,
               "user".cid AS user_cid,
               "user".full_name AS user_full_name,
               "user".email AS user_email,
               "user".created_at AS user_created_at,
               "user".updated_at AS user_updated_at,
               "user".roles AS user_roles,
               inserted_application.application_filing_id,
               inserted_application.review_filing_id,
               inserted_application.applied_at,
               inserted_application.status
        FROM inserted_application
        INNER JOIN public."user" ON "user".id = inserted_application.user_id
        "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(application_filing_id)
        .bind(Utc::now())
        .fetch_one(self)
        .await
    }
    async fn set_atc_application_review(
        self,
        id: Uuid,
        status: &str,
        review_filing_id: Uuid,
    ) -> Result<Option<AtcApplicationRecord>, sqlx::Error> {
        tracing::info!(
            operation = "set_review",
            repository = "src/repository/atc/atc_application.rs",
            "modifying data"
        );

        sqlx::query_as::<_, AtcApplicationRecord>(
            r#"
        WITH updated_application AS (
            UPDATE public.atc_application
            SET status = $2, review_filing_id = $3
            WHERE id = $1
            RETURNING id, user_id, application_filing_id, review_filing_id,
                      applied_at, status
        )
        SELECT updated_application.id,
               updated_application.user_id,
               "user".cid AS user_cid,
               "user".full_name AS user_full_name,
               "user".email AS user_email,
               "user".created_at AS user_created_at,
               "user".updated_at AS user_updated_at,
               "user".roles AS user_roles,
               updated_application.application_filing_id,
               updated_application.review_filing_id,
               updated_application.applied_at,
               updated_application.status
        FROM updated_application
        INNER JOIN public."user" ON "user".id = updated_application.user_id
        "#,
        )
        .bind(id)
        .bind(status)
        .bind(review_filing_id)
        .fetch_optional(self)
        .await
    }
}
