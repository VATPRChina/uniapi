use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use ulid::Ulid;
use uuid::Uuid;

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

#[derive(Debug, Clone, FromRow)]
pub struct TrainingApplicationSlotRecord {
    pub id: Uuid,
    pub application_id: Uuid,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

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

#[derive(Debug, Clone)]
pub struct TrainingApplicationSlotSave {
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
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

pub async fn list_slots(
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

pub async fn find_slot(
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

    replace_slots(transaction, id, slots).await?;
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

    replace_slots(transaction, id, slots).await?;
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

pub async fn list_responses(
    db: &PgPool,
    application_id: Uuid,
) -> Result<Vec<TrainingApplicationResponseRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingApplicationResponseRecord>(&response_select_sql(
        r#"
        WHERE training_application_response.application_id = $1
        ORDER BY training_application_response.created_at DESC
        "#,
    ))
    .bind(application_id)
    .fetch_all(db)
    .await
}

pub async fn create_response(
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

pub async fn find_response_by_id(
    db: &PgPool,
    id: Uuid,
) -> Result<Option<TrainingApplicationResponseRecord>, sqlx::Error> {
    sqlx::query_as::<_, TrainingApplicationResponseRecord>(&response_select_sql(
        r#"
        WHERE training_application_response.id = $1
        "#,
    ))
    .bind(id)
    .fetch_optional(db)
    .await
}

async fn replace_slots(
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

fn response_select_sql(where_clause: &str) -> String {
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
