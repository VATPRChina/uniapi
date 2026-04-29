use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct TrainingApplicationSlotRecord {
    pub id: Uuid,
    pub application_id: Uuid,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TrainingApplicationSlotSave {
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

pub async fn list(
    db: &PgPool,
    application_id: Uuid,
) -> Result<Vec<TrainingApplicationSlotRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingApplicationSlotRecord>(
        r#"
        SELECT id, application_id, start_at, end_at
        FROM public.training_application_slot
        WHERE application_id = $1
        ORDER BY start_at
        "#,
    )
    .bind(application_id)
    .fetch_all(db)
    .await
}

pub async fn find(
    db: &PgPool,
    application_id: Uuid,
    slot_id: Uuid,
) -> Result<Option<TrainingApplicationSlotRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingApplicationSlotRecord>(
        r#"
        SELECT id, application_id, start_at, end_at
        FROM public.training_application_slot
        WHERE application_id = $1 AND id = $2
        "#,
    )
    .bind(application_id)
    .bind(slot_id)
    .fetch_optional(db)
    .await
}

pub async fn replace(
    transaction: &mut Transaction<'_, Postgres>,
    application_id: Uuid,
    slots: &[TrainingApplicationSlotSave],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM public.training_application_slot
        WHERE application_id = $1
        "#,
    )
    .bind(application_id)
    .execute(&mut **transaction)
    .await?;

    for slot in slots {
        sqlx::query(
            r#"
            INSERT INTO public.training_application_slot (
                id, application_id, start_at, end_at
            )
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::from(Ulid::new()))
        .bind(application_id)
        .bind(slot.start_at)
        .bind(slot.end_at)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}
