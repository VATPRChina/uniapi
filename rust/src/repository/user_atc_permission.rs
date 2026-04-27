use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AtcPermissionRecord {
    pub position_kind_id: String,
    pub state: String,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct AtcPermissionSave {
    pub position_kind_id: String,
    pub state: String,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

pub async fn has_any_by_user_id(db: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM public.user_atc_permission
            WHERE user_id = $1
        )
        "#,
    )
    .bind(user_id)
    .fetch_one(db)
    .await
}

pub async fn has_mentor_by_user_id(db: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM public.user_atc_permission
            WHERE user_id = $1 AND state = 'Mentor'
        )
        "#,
    )
    .bind(user_id)
    .fetch_one(db)
    .await
}

pub async fn list_by_user_id(
    db: &PgPool,
    user_id: Uuid,
) -> Result<Vec<AtcPermissionRecord>, sqlx::Error> {
    sqlx::query_as::<_, AtcPermissionRecord>(
        r#"
        SELECT position_kind_id, state, solo_expires_at
        FROM public.user_atc_permission
        WHERE user_id = $1
        ORDER BY position_kind_id
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn replace(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    permissions: &[AtcPermissionSave],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM public.user_atc_permission
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(&mut **transaction)
    .await?;

    for permission in permissions {
        sqlx::query(
            r#"
            INSERT INTO public.user_atc_permission (
                user_id, position_kind_id, state, solo_expires_at
            )
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user_id)
        .bind(&permission.position_kind_id)
        .bind(&permission.state)
        .bind(permission.solo_expires_at)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}
