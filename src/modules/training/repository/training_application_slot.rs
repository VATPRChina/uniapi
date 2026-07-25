use ulid::Ulid;
use uuid::Uuid;

use crate::modules::training::models::{TrainingApplicationSlot, TrainingApplicationSlotSave};

pub(crate) trait TrainingApplicationSlotRepository<'executor> {
    async fn list_training_application_slot(
        self,
        application_id: Uuid,
    ) -> Result<Vec<TrainingApplicationSlot>, sqlx::Error>;

    async fn find_training_application_slot(
        self,
        application_id: Uuid,
        slot_id: Uuid,
    ) -> Result<Option<TrainingApplicationSlot>, sqlx::Error>;
}

impl<'executor, E> TrainingApplicationSlotRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_training_application_slot(
        self,
        application_id: Uuid,
    ) -> Result<Vec<TrainingApplicationSlot>, sqlx::Error> {
        sqlx::query_as::<_, TrainingApplicationSlot>(
            r#"
        SELECT id, application_id, start_at, end_at
        FROM public.training_application_slot
        WHERE application_id = $1
        ORDER BY start_at
        "#,
        )
        .bind(application_id)
        .fetch_all(self)
        .await
    }
    async fn find_training_application_slot(
        self,
        application_id: Uuid,
        slot_id: Uuid,
    ) -> Result<Option<TrainingApplicationSlot>, sqlx::Error> {
        sqlx::query_as::<_, TrainingApplicationSlot>(
            r#"
        SELECT id, application_id, start_at, end_at
        FROM public.training_application_slot
        WHERE application_id = $1 AND id = $2
        "#,
        )
        .bind(application_id)
        .bind(slot_id)
        .fetch_optional(self)
        .await
    }
}

pub(crate) trait TrainingApplicationSlotTransactionRepository {
    async fn replace_training_application_slot(
        &mut self,
        application_id: Uuid,
        slots: &[TrainingApplicationSlotSave],
    ) -> Result<(), sqlx::Error>;
}

impl TrainingApplicationSlotTransactionRepository for sqlx::Transaction<'_, sqlx::Postgres> {
    async fn replace_training_application_slot(
        &mut self,
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
        .execute(&mut **self)
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
            .execute(&mut **self)
            .await?;
        }

        Ok(())
    }
}
