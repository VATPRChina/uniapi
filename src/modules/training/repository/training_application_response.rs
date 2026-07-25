use chrono::Utc;
use ulid::Ulid;
use uuid::Uuid;

use crate::modules::training::models::{
    TrainingApplication, TrainingApplicationResponse, TrainingApplicationSlot,
};

fn select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT training_application_response.id,
               training_application_response.application_id,
               training_application_response.trainer_id,
               training_application_response.slot_id,
               training_application_response.comment,
               training_application_response.created_at,
               training_application_response.updated_at
        FROM public.training_application_response
        {where_clause}
        "#
    )
}

pub(crate) trait TrainingApplicationResponseRepository<'executor> {
    async fn list_training_application_response(
        self,
        application_id: Uuid,
    ) -> Result<Vec<TrainingApplicationResponse>, sqlx::Error>;

    async fn find_training_application_response(
        self,
        id: Uuid,
    ) -> Result<Option<TrainingApplicationResponse>, sqlx::Error>;
}

impl<'executor, E> TrainingApplicationResponseRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_training_application_response(
        self,
        application_id: Uuid,
    ) -> Result<Vec<TrainingApplicationResponse>, sqlx::Error> {
        sqlx::query_as::<_, TrainingApplicationResponse>(&select_sql(
            r#"
        WHERE training_application_response.application_id = $1
        ORDER BY training_application_response.created_at DESC
        "#,
        ))
        .bind(application_id)
        .fetch_all(self)
        .await
    }
    async fn find_training_application_response(
        self,
        id: Uuid,
    ) -> Result<Option<TrainingApplicationResponse>, sqlx::Error> {
        sqlx::query_as::<_, TrainingApplicationResponse>(&select_sql(
            r#"
        WHERE training_application_response.id = $1
        "#,
        ))
        .bind(id)
        .fetch_optional(self)
        .await
    }
}

pub(crate) trait TrainingApplicationResponseTransactionRepository {
    async fn create_training_application_response(
        &mut self,
        application: &TrainingApplication,
        trainer_id: Uuid,
        slot: Option<&TrainingApplicationSlot>,
        comment: &str,
    ) -> Result<Uuid, sqlx::Error>;
}

impl TrainingApplicationResponseTransactionRepository for sqlx::Transaction<'_, sqlx::Postgres> {
    async fn create_training_application_response(
        &mut self,
        application: &TrainingApplication,
        trainer_id: Uuid,
        slot: Option<&TrainingApplicationSlot>,
        comment: &str,
    ) -> Result<Uuid, sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "src/modules/training/repository/training_application_response.rs",
            "modifying data"
        );

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
        .execute(&mut **self)
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
            .execute(&mut **self)
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
            .execute(&mut **self)
            .await?;
        }

        Ok(response_id)
    }
}
