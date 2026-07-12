use chrono::{DateTime, Utc};
use sqlx::FromRow;
use ulid::Ulid;
use uuid::Uuid;

use crate::repository::atc_training::training_application_slot::{
    TrainingApplicationSlotSave, TrainingApplicationSlotTransactionExt,
};

#[derive(Debug, Clone, FromRow)]
pub struct TrainingApplicationRecord {
    pub id: Uuid,
    pub trainee_id: Uuid,
    pub trainee_cid: String,
    pub trainee_full_name: String,
    pub trainee_email: Option<String>,
    pub trainee_created_at: DateTime<Utc>,
    pub trainee_updated_at: DateTime<Utc>,
    pub trainee_roles: Vec<String>,
    pub name: String,
    pub train_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

fn application_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT training_application.id,
               training_application.trainee_id,
               trainee.cid AS trainee_cid,
               trainee.full_name AS trainee_full_name,
               trainee.email AS trainee_email,
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

pub trait TrainingApplicationRepositoryExt<'executor> {
    async fn list_training_application(
        self,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<Vec<TrainingApplicationRecord>, sqlx::Error>;

    async fn find_training_application_visible_by_id(
        self,
        id: Uuid,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<Option<TrainingApplicationRecord>, sqlx::Error>;

    async fn find_training_application_by_id(
        self,
        id: Uuid,
    ) -> Result<Option<TrainingApplicationRecord>, sqlx::Error>;

    async fn mark_training_application_deleted(self, id: Uuid) -> Result<bool, sqlx::Error>;
}

impl<'executor, E> TrainingApplicationRepositoryExt<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_training_application(
        self,
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
        .fetch_all(self)
        .await
    }
    async fn find_training_application_visible_by_id(
        self,
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
        .fetch_optional(self)
        .await
    }
    async fn find_training_application_by_id(
        self,
        id: Uuid,
    ) -> Result<Option<TrainingApplicationRecord>, sqlx::Error> {
        sqlx::query_as::<_, TrainingApplicationRecord>(&application_select_sql(
            r#"
        WHERE training_application.id = $1
        "#,
        ))
        .bind(id)
        .fetch_optional(self)
        .await
    }
    async fn mark_training_application_deleted(self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE public.training_application
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

pub trait TrainingApplicationTransactionExt {
    async fn create_training_application(
        &mut self,
        trainee_id: Uuid,
        name: &str,
        slots: &[TrainingApplicationSlotSave],
    ) -> Result<Uuid, sqlx::Error>;

    async fn update_training_application(
        &mut self,
        id: Uuid,
        name: &str,
        slots: &[TrainingApplicationSlotSave],
    ) -> Result<bool, sqlx::Error>;
}

impl TrainingApplicationTransactionExt for sqlx::Transaction<'_, sqlx::Postgres> {
    async fn create_training_application(
        &mut self,
        trainee_id: Uuid,
        name: &str,
        slots: &[TrainingApplicationSlotSave],
    ) -> Result<Uuid, sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "src/repository/atc_training/training_application.rs",
            "modifying data"
        );

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
        .execute(&mut **self)
        .await?;

        self.replace_training_application_slot(id, slots).await?;
        Ok(id)
    }

    async fn update_training_application(
        &mut self,
        id: Uuid,
        name: &str,
        slots: &[TrainingApplicationSlotSave],
    ) -> Result<bool, sqlx::Error> {
        tracing::info!(
            operation = "update",
            repository = "src/repository/atc_training/training_application.rs",
            "modifying data"
        );

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
        .execute(&mut **self)
        .await?;

        self.replace_training_application_slot(id, slots).await?;
        Ok(result.rows_affected() > 0)
    }
}
