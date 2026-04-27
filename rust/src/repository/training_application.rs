use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use ulid::Ulid;
use uuid::Uuid;

use crate::repository::training_application_slot::{self, TrainingApplicationSlotSave};

#[derive(Debug, Clone, FromRow)]
pub struct TrainingApplicationRecord {
    pub id: Uuid,
    pub trainee_id: Uuid,
    pub trainee_cid: String,
    pub trainee_full_name: String,
    pub trainee_created_at: DateTime<Utc>,
    pub trainee_updated_at: DateTime<Utc>,
    pub trainee_roles: Vec<String>,
    pub name: String,
    pub train_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

pub async fn list(
    db: &PgPool,
    current_user_id: Uuid,
    is_admin: bool,
) -> Result<Vec<TrainingApplicationRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingApplicationRecord>(&application_select_sql(
        r#"
        WHERE ($1 OR training_application.trainee_id = $2)
        ORDER BY training_application.created_at DESC
        "#,
    ))
    .bind(is_admin)
    .bind(current_user_id)
    .fetch_all(db)
    .await
}

pub async fn find_visible_by_id(
    db: &PgPool,
    id: Uuid,
    current_user_id: Uuid,
    is_admin: bool,
) -> Result<Option<TrainingApplicationRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingApplicationRecord>(&application_select_sql(
        r#"
        WHERE training_application.id = $1
          AND ($2 OR training_application.trainee_id = $3)
        "#,
    ))
    .bind(id)
    .bind(is_admin)
    .bind(current_user_id)
    .fetch_optional(db)
    .await
}

pub async fn find_by_id(
    db: &PgPool,
    id: Uuid,
) -> Result<Option<TrainingApplicationRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingApplicationRecord>(&application_select_sql(
        r#"
        WHERE training_application.id = $1
        "#,
    ))
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn create(
    transaction: &mut Transaction<'_, Postgres>,
    trainee_id: Uuid,
    name: &str,
    slots: &[TrainingApplicationSlotSave],
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::from(Ulid::new());
    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO public.training_application (
            id, trainee_id, name, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $4)
        "#,
    )
    .bind(id)
    .bind(trainee_id)
    .bind(name)
    .bind(now)
    .execute(&mut **transaction)
    .await?;

    training_application_slot::replace(transaction, id, slots).await?;
    Ok(id)
}

pub async fn update(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    name: &str,
    slots: &[TrainingApplicationSlotSave],
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE public.training_application
        SET name = $2, updated_at = $3
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(Utc::now())
    .execute(&mut **transaction)
    .await?;

    training_application_slot::replace(transaction, id, slots).await?;
    Ok(result.rows_affected() > 0)
}

pub async fn mark_deleted(db: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE public.training_application
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

fn application_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT training_application.id,
               training_application.trainee_id,
               trainee.cid AS trainee_cid,
               trainee.full_name AS trainee_full_name,
               trainee.created_at AS trainee_created_at,
               trainee.updated_at AS trainee_updated_at,
               trainee.roles AS trainee_roles,
               training_application.name,
               training_application.train_id,
               training_application.created_at,
               training_application.updated_at,
               training_application.deleted_at
        FROM public.training_application
        INNER JOIN public."user" AS trainee ON trainee.id = training_application.trainee_id
        {where_clause}
        "#
    )
}
