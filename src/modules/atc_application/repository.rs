use std::future::Future;

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use ulid::Ulid;
use uuid::Uuid;

use super::models::{AtcApplication, AtcApplicationStatus, InvalidAtcApplicationStatus};

#[derive(Debug, Clone, FromRow)]
struct AtcApplicationRecord {
    pub id: Uuid,
    pub user_id: Uuid,
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
               atc_application.application_filing_id,
               atc_application.review_filing_id,
               atc_application.applied_at,
               atc_application.status
        FROM public.atc_application
        {where_clause}
        "#
    )
}

pub trait AtcApplicationRepository<'executor> {
    fn list_atc_application(self)
    -> impl Future<Output = Result<Vec<AtcApplication>, sqlx::Error>>;

    fn find_atc_application_by_id(
        self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<AtcApplication>, sqlx::Error>>;

    fn find_atc_application_by_id_for_update(
        self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<AtcApplication>, sqlx::Error>>;

    fn count_atc_application_active_by_user(
        self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<i64, sqlx::Error>>;

    fn create_atc_application(
        self,
        user_id: Uuid,
        application_filing_id: Uuid,
    ) -> impl Future<Output = Result<AtcApplication, sqlx::Error>>;

    fn set_atc_application_review(
        self,
        id: Uuid,
        status: &str,
        review_filing_id: Uuid,
    ) -> impl Future<Output = Result<Option<AtcApplication>, sqlx::Error>>;
}

impl<'executor, E> AtcApplicationRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_atc_application(self) -> Result<Vec<AtcApplication>, sqlx::Error> {
        sqlx::query_as::<_, AtcApplicationRecord>(&application_select_sql(
            r#"
        ORDER BY atc_application.applied_at DESC
        "#,
        ))
        .fetch_all(self)
        .await?
        .into_iter()
        .map(|record| {
            AtcApplication::try_from(record).map_err(|error| sqlx::Error::Decode(Box::new(error)))
        })
        .collect()
    }
    async fn find_atc_application_by_id(
        self,
        id: Uuid,
    ) -> Result<Option<AtcApplication>, sqlx::Error> {
        sqlx::query_as::<_, AtcApplicationRecord>(&application_select_sql(
            r#"
        WHERE atc_application.id = $1
        "#,
        ))
        .bind(id)
        .fetch_optional(self)
        .await?
        .map(AtcApplication::try_from)
        .transpose()
        .map_err(|error| sqlx::Error::Decode(Box::new(error)))
    }
    async fn find_atc_application_by_id_for_update(
        self,
        id: Uuid,
    ) -> Result<Option<AtcApplication>, sqlx::Error> {
        sqlx::query_as::<_, AtcApplicationRecord>(&application_select_sql(
            r#"
        WHERE atc_application.id = $1
        FOR UPDATE OF atc_application
        "#,
        ))
        .bind(id)
        .fetch_optional(self)
        .await?
        .map(AtcApplication::try_from)
        .transpose()
        .map_err(|error| sqlx::Error::Decode(Box::new(error)))
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
    ) -> Result<AtcApplication, sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "src/modules/atc_application/repository.rs",
            "modifying data"
        );

        let id = Uuid::from(Ulid::new());
        AtcApplication::try_from(
            sqlx::query_as::<_, AtcApplicationRecord>(
                r#"
        INSERT INTO public.atc_application (
            id, user_id, application_filing_id, applied_at, status
        )
        VALUES ($1, $2, $3, $4, 'Submitted')
        RETURNING id, user_id, application_filing_id, review_filing_id,
                  applied_at, status
        "#,
            )
            .bind(id)
            .bind(user_id)
            .bind(application_filing_id)
            .bind(Utc::now())
            .fetch_one(self)
            .await?,
        )
        .map_err(|error| sqlx::Error::Decode(Box::new(error)))
    }
    async fn set_atc_application_review(
        self,
        id: Uuid,
        status: &str,
        review_filing_id: Uuid,
    ) -> Result<Option<AtcApplication>, sqlx::Error> {
        tracing::info!(
            operation = "set_review",
            repository = "src/modules/atc_application/repository.rs",
            "modifying data"
        );

        sqlx::query_as::<_, AtcApplicationRecord>(
            r#"
        UPDATE public.atc_application
        SET status = $2, review_filing_id = $3
        WHERE id = $1
        RETURNING id, user_id, application_filing_id, review_filing_id,
                  applied_at, status
        "#,
        )
        .bind(id)
        .bind(status)
        .bind(review_filing_id)
        .fetch_optional(self)
        .await?
        .map(AtcApplication::try_from)
        .transpose()
        .map_err(|error| sqlx::Error::Decode(Box::new(error)))
    }
}

impl TryFrom<AtcApplicationRecord> for AtcApplication {
    type Error = InvalidAtcApplicationStatus;

    fn try_from(record: AtcApplicationRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: record.id,
            user_id: record.user_id,
            application_filing_id: record.application_filing_id,
            review_filing_id: record.review_filing_id,
            applied_at: record.applied_at,
            status: AtcApplicationStatus::try_from(record.status.as_str())?,
        })
    }
}
