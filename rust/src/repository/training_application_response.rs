use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use ulid::Ulid;
use uuid::Uuid;

use crate::repository::{
    training_application::TrainingApplicationRecord,
    training_application_slot::TrainingApplicationSlotRecord,
};

#[derive(Debug, Clone, FromRow)]
pub struct TrainingApplicationResponseRecord {
    pub id: Uuid,
    pub application_id: Uuid,
    pub trainer_id: Uuid,
    pub trainer_cid: String,
    pub trainer_full_name: String,
    pub trainer_created_at: DateTime<Utc>,
    pub trainer_updated_at: DateTime<Utc>,
    pub trainer_roles: Vec<String>,
    pub slot_id: Option<Uuid>,
    pub comment: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn list(
    db: &PgPool,
    application_id: Uuid,
) -> Result<Vec<TrainingApplicationResponseRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingApplicationResponseRecord>(&select_sql(
        r#"
        WHERE training_application_response.application_id = $1
        ORDER BY training_application_response.created_at DESC
        "#,
    ))
    .bind(application_id)
    .fetch_all(db)
    .await
}

pub async fn create(
    transaction: &mut Transaction<'_, Postgres>,
    application: &TrainingApplicationRecord,
    trainer_id: Uuid,
    slot: Option<&TrainingApplicationSlotRecord>,
    comment: &str,
) -> Result<Uuid, sqlx::Error> {
    let response_id = Uuid::from(Ulid::new());
    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO public.training_application_response (
            id, application_id, trainer_id, slot_id, comment, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        "#,
    )
    .bind(response_id)
    .bind(application.id)
    .bind(trainer_id)
    .bind(slot.map(|slot| slot.id))
    .bind(comment)
    .bind(now)
    .execute(&mut **transaction)
    .await?;

    if let Some(slot) = slot {
        let training_id = Uuid::from(Ulid::new());
        sqlx::query(
            r#"
            INSERT INTO public.training (
                id, name, trainer_id, trainee_id, start_at, end_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
            "#,
        )
        .bind(training_id)
        .bind(&application.name)
        .bind(trainer_id)
        .bind(application.trainee_id)
        .bind(slot.start_at)
        .bind(slot.end_at)
        .bind(now)
        .execute(&mut **transaction)
        .await?;

        sqlx::query(
            r#"
            UPDATE public.training_application
            SET train_id = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(application.id)
        .bind(training_id)
        .bind(now)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(response_id)
}

pub async fn find(
    db: &PgPool,
    id: Uuid,
) -> Result<Option<TrainingApplicationResponseRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingApplicationResponseRecord>(&select_sql(
        r#"
        WHERE training_application_response.id = $1
        "#,
    ))
    .bind(id)
    .fetch_optional(db)
    .await
}

fn select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT training_application_response.id,
               training_application_response.application_id,
               training_application_response.trainer_id,
               trainer.cid AS trainer_cid,
               trainer.full_name AS trainer_full_name,
               trainer.created_at AS trainer_created_at,
               trainer.updated_at AS trainer_updated_at,
               trainer.roles AS trainer_roles,
               training_application_response.slot_id,
               training_application_response.comment,
               training_application_response.created_at,
               training_application_response.updated_at
        FROM public.training_application_response
        INNER JOIN public."user" AS trainer ON trainer.id = training_application_response.trainer_id
        {where_clause}
        "#
    )
}
