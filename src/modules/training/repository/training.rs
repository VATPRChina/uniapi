use chrono::Utc;
use ulid::Ulid;
use uuid::Uuid;

use crate::modules::training::models::{Training, TrainingSave};

fn training_select_sql(where_clause: &str) -> String {
    training_select_sql_from("public.training", where_clause)
}

fn training_select_sql_from(source: &str, where_clause: &str) -> String {
    format!(
        r#"
        SELECT training.id,
               training.name,
               training.trainer_id,
               training.trainee_id,
               training.start_at,
               training.end_at,
               training.created_at,
               training.updated_at,
               training.deleted_at,
               training.record_sheet_filing_id
        FROM {source}
        {where_clause}
        "#
    )
}

pub(crate) trait TrainingRepository<'executor> {
    async fn list_training_active(
        self,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<Vec<Training>, sqlx::Error>;

    async fn list_training_finished(
        self,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<Vec<Training>, sqlx::Error>;

    async fn list_training_by_user(self, user_id: Uuid) -> Result<Vec<Training>, sqlx::Error>;

    async fn find_training_by_id(self, id: Uuid) -> Result<Option<Training>, sqlx::Error>;

    async fn create_training(self, training: TrainingSave) -> Result<Training, sqlx::Error>;

    async fn update_training(
        self,
        id: Uuid,
        training: TrainingSave,
    ) -> Result<Option<Training>, sqlx::Error>;

    async fn set_training_record_filing(
        self,
        id: Uuid,
        filing_id: Uuid,
    ) -> Result<Option<Training>, sqlx::Error>;

    async fn mark_training_deleted(self, id: Uuid) -> Result<bool, sqlx::Error>;
}

impl<'executor, E> TrainingRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_training_active(
        self,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<Vec<Training>, sqlx::Error> {
        sqlx::query_as::<_, Training>(&training_select_sql(
            r#"
        WHERE training.deleted_at IS NULL
          AND training.record_sheet_filing_id IS NULL
          AND ($1 OR training.trainer_id = $2 OR training.trainee_id = $2)
        ORDER BY training.created_at DESC
        "#,
        ))
        .bind(is_admin)
        .bind(current_user_id)
        .fetch_all(self)
        .await
    }
    async fn list_training_finished(
        self,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<Vec<Training>, sqlx::Error> {
        sqlx::query_as::<_, Training>(&training_select_sql(
            r#"
        WHERE (training.record_sheet_filing_id IS NOT NULL OR training.deleted_at IS NOT NULL)
          AND ($1 OR training.trainer_id = $2 OR training.trainee_id = $2)
        ORDER BY training.created_at DESC
        "#,
        ))
        .bind(is_admin)
        .bind(current_user_id)
        .fetch_all(self)
        .await
    }
    async fn list_training_by_user(self, user_id: Uuid) -> Result<Vec<Training>, sqlx::Error> {
        sqlx::query_as::<_, Training>(&training_select_sql(
            r#"
        WHERE training.trainer_id = $1 OR training.trainee_id = $1
        ORDER BY training.created_at DESC
        "#,
        ))
        .bind(user_id)
        .fetch_all(self)
        .await
    }
    async fn find_training_by_id(self, id: Uuid) -> Result<Option<Training>, sqlx::Error> {
        sqlx::query_as::<_, Training>(&training_select_sql(
            r#"
        WHERE training.id = $1
        "#,
        ))
        .bind(id)
        .fetch_optional(self)
        .await
    }
    async fn create_training(self, training: TrainingSave) -> Result<Training, sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "src/modules/training/repository/training.rs",
            "modifying data"
        );

        let id = Uuid::from(Ulid::new());
        let now = Utc::now();
        let query = format!(
            r#"
        WITH inserted AS (
          INSERT INTO public.training (
            id, name, trainer_id, trainee_id, start_at, end_at, created_at, updated_at
          )
          VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
          RETURNING *
        )
        {}
        "#,
            training_select_sql_from("inserted AS training", "WHERE training.id = $1"),
        );
        sqlx::query_as::<_, Training>(&query)
            .bind(id)
            .bind(training.name)
            .bind(training.trainer_id)
            .bind(training.trainee_id)
            .bind(training.start_at)
            .bind(training.end_at)
            .bind(now)
            .fetch_one(self)
            .await
    }
    async fn update_training(
        self,
        id: Uuid,
        training: TrainingSave,
    ) -> Result<Option<Training>, sqlx::Error> {
        tracing::info!(
            operation = "update",
            repository = "src/modules/training/repository/training.rs",
            "modifying data"
        );

        let query = format!(
            r#"
        WITH updated AS (
          UPDATE public.training
          SET name = $2, start_at = $3, end_at = $4, updated_at = $5
          WHERE id = $1
          RETURNING *
        )
        {}
        "#,
            training_select_sql_from("updated AS training", "WHERE training.id = $1"),
        );
        sqlx::query_as::<_, Training>(&query)
            .bind(id)
            .bind(training.name)
            .bind(training.start_at)
            .bind(training.end_at)
            .bind(Utc::now())
            .fetch_optional(self)
            .await
    }
    async fn set_training_record_filing(
        self,
        id: Uuid,
        filing_id: Uuid,
    ) -> Result<Option<Training>, sqlx::Error> {
        tracing::info!(
            operation = "set_record_filing",
            repository = "src/modules/training/repository/training.rs",
            "modifying data"
        );

        let query = format!(
            r#"
        WITH updated AS (
          UPDATE public.training
          SET record_sheet_filing_id = $2, updated_at = $3
          WHERE id = $1
          RETURNING *
        )
        {}
        "#,
            training_select_sql_from("updated AS training", "WHERE training.id = $1"),
        );
        sqlx::query_as::<_, Training>(&query)
            .bind(id)
            .bind(filing_id)
            .bind(Utc::now())
            .fetch_optional(self)
            .await
    }
    async fn mark_training_deleted(self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE public.training
        SET deleted_at = $2, updated_at = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(Utc::now())
        .execute(self)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
