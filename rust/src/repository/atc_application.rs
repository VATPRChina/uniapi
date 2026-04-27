use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AtcApplicationRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_cid: String,
    pub user_full_name: String,
    pub user_created_at: DateTime<Utc>,
    pub user_updated_at: DateTime<Utc>,
    pub user_roles: Vec<String>,
    pub application_filing_id: Uuid,
    pub review_filing_id: Option<Uuid>,
    pub applied_at: DateTime<Utc>,
    pub status: String,
}

pub async fn list(db: &PgPool) -> Result<Vec<AtcApplicationRecord>, sqlx::Error> {
    sqlx::query_as::<_, AtcApplicationRecord>(&application_select_sql(
        r#"
        ORDER BY atc_application.applied_at DESC
        "#,
    ))
    .fetch_all(db)
    .await
}

pub async fn find_by_id(
    db: &PgPool,
    id: Uuid,
) -> Result<Option<AtcApplicationRecord>, sqlx::Error> {
    sqlx::query_as::<_, AtcApplicationRecord>(&application_select_sql(
        r#"
        WHERE atc_application.id = $1
        "#,
    ))
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn count_active_by_user(db: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM public.atc_application
        WHERE user_id = $1 AND status != 'Rejected'
        "#,
    )
    .bind(user_id)
    .fetch_one(db)
    .await
}

pub async fn create(
    db: &PgPool,
    user_id: Uuid,
    application_filing_id: Uuid,
) -> Result<AtcApplicationRecord, sqlx::Error> {
    let id = Uuid::from(Ulid::new());
    sqlx::query(
        r#"
        INSERT INTO public.atc_application (
            id, user_id, application_filing_id, applied_at, status
        )
        VALUES ($1, $2, $3, $4, 'Submitted')
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(application_filing_id)
    .bind(Utc::now())
    .execute(db)
    .await?;

    find_by_id(db, id).await?.ok_or(sqlx::Error::RowNotFound)
}

pub async fn set_review(
    db: &PgPool,
    id: Uuid,
    status: &str,
    review_filing_id: Uuid,
) -> Result<Option<AtcApplicationRecord>, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE public.atc_application
        SET status = $2, review_filing_id = $3
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(review_filing_id)
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    find_by_id(db, id).await
}

fn application_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT atc_application.id,
               atc_application.user_id,
               "user".cid AS user_cid,
               "user".full_name AS user_full_name,
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
