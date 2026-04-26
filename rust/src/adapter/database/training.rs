use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct TrainingRecord {
    pub id: Uuid,
    pub name: String,
    pub trainer_id: Uuid,
    pub trainer_cid: String,
    pub trainer_full_name: String,
    pub trainer_created_at: DateTime<Utc>,
    pub trainer_updated_at: DateTime<Utc>,
    pub trainer_roles: Vec<String>,
    pub trainee_id: Uuid,
    pub trainee_cid: String,
    pub trainee_full_name: String,
    pub trainee_created_at: DateTime<Utc>,
    pub trainee_updated_at: DateTime<Utc>,
    pub trainee_roles: Vec<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub record_sheet_filing_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct TrainingSave {
    pub name: String,
    pub trainer_id: Uuid,
    pub trainee_id: Uuid,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

pub async fn list_active(
    db: &PgPool,
    current_user_id: Uuid,
    is_admin: bool,
) -> Result<Vec<TrainingRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingRecord>(&training_select_sql(
        r#"
        WHERE training.deleted_at IS NULL
          AND training.record_sheet_filing_id IS NULL
          AND ($1 OR training.trainer_id = $2 OR training.trainee_id = $2)
        ORDER BY training.created_at DESC
        "#,
    ))
    .bind(is_admin)
    .bind(current_user_id)
    .fetch_all(db)
    .await
}

pub async fn list_finished(
    db: &PgPool,
    current_user_id: Uuid,
    is_admin: bool,
) -> Result<Vec<TrainingRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingRecord>(&training_select_sql(
        r#"
        WHERE (training.record_sheet_filing_id IS NOT NULL OR training.deleted_at IS NOT NULL)
          AND ($1 OR training.trainer_id = $2 OR training.trainee_id = $2)
        ORDER BY training.created_at DESC
        "#,
    ))
    .bind(is_admin)
    .bind(current_user_id)
    .fetch_all(db)
    .await
}

pub async fn list_by_trainee(
    db: &PgPool,
    trainee_id: Uuid,
) -> Result<Vec<TrainingRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingRecord>(&training_select_sql(
        r#"
        WHERE training.trainee_id = $1
        ORDER BY training.created_at DESC
        "#,
    ))
    .bind(trainee_id)
    .fetch_all(db)
    .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<TrainingRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingRecord>(&training_select_sql(
        r#"
        WHERE training.id = $1
        "#,
    ))
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn create(db: &PgPool, training: TrainingSave) -> Result<TrainingRecord, sqlx::Error> {
    let id = Uuid::from(Ulid::new());
    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO public.training (
            id, name, trainer_id, trainee_id, start_at, end_at, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
        "#,
    )
    .bind(id)
    .bind(training.name)
    .bind(training.trainer_id)
    .bind(training.trainee_id)
    .bind(training.start_at)
    .bind(training.end_at)
    .bind(now)
    .execute(db)
    .await?;

    find_by_id(db, id).await?.ok_or(sqlx::Error::RowNotFound)
}

pub async fn update(
    db: &PgPool,
    id: Uuid,
    training: TrainingSave,
) -> Result<Option<TrainingRecord>, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE public.training
        SET name = $2, start_at = $3, end_at = $4, updated_at = $5
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(training.name)
    .bind(training.start_at)
    .bind(training.end_at)
    .bind(Utc::now())
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    find_by_id(db, id).await
}

pub async fn set_record_filing(
    db: &PgPool,
    id: Uuid,
    filing_id: Uuid,
) -> Result<Option<TrainingRecord>, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE public.training
        SET record_sheet_filing_id = $2, updated_at = $3
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(filing_id)
    .bind(Utc::now())
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    find_by_id(db, id).await
}

pub async fn mark_deleted(db: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE public.training
        SET deleted_at = $2, updated_at = $2
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(Utc::now())
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

fn training_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT training.id,
               training.name,
               training.trainer_id,
               trainer.cid AS trainer_cid,
               trainer.full_name AS trainer_full_name,
               trainer.created_at AS trainer_created_at,
               trainer.updated_at AS trainer_updated_at,
               trainer.roles AS trainer_roles,
               training.trainee_id,
               trainee.cid AS trainee_cid,
               trainee.full_name AS trainee_full_name,
               trainee.created_at AS trainee_created_at,
               trainee.updated_at AS trainee_updated_at,
               trainee.roles AS trainee_roles,
               training.start_at,
               training.end_at,
               training.created_at,
               training.updated_at,
               training.deleted_at,
               training.record_sheet_filing_id
        FROM public.training
        INNER JOIN public."user" AS trainer ON trainer.id = training.trainer_id
        INNER JOIN public."user" AS trainee ON trainee.id = training.trainee_id
        {where_clause}
        "#
    )
}
