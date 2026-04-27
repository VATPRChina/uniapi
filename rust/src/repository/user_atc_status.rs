use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::repository::user_atc_permission::{self, AtcPermissionSave};

#[derive(Debug, Clone, FromRow)]
pub struct AtcStatusRecord {
    pub user_id: Uuid,
    pub user_cid: String,
    pub user_full_name: String,
    pub user_created_at: DateTime<Utc>,
    pub user_updated_at: DateTime<Utc>,
    pub user_roles: Vec<String>,
    pub is_visiting: Option<bool>,
    pub is_absent: Option<bool>,
    pub rating: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AtcStatusSave {
    pub is_visiting: bool,
    pub is_absent: bool,
    pub rating: String,
    pub permissions: Vec<AtcPermissionSave>,
}

pub async fn find_by_user_id(
    db: &PgPool,
    user_id: Uuid,
) -> Result<Option<AtcStatusRecord>, sqlx::Error> {
    sqlx::query_as::<_, AtcStatusRecord>(
        r#"
        SELECT "user".id AS user_id,
               "user".cid AS user_cid,
               "user".full_name AS user_full_name,
               "user".created_at AS user_created_at,
               "user".updated_at AS user_updated_at,
               "user".roles AS user_roles,
               user_atc_status.is_visiting,
               user_atc_status.is_absent,
               user_atc_status.rating
        FROM public."user"
        LEFT JOIN public.user_atc_status ON user_atc_status.user_id = "user".id
        WHERE "user".id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
}

pub async fn upsert(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    status: &AtcStatusSave,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO public.user_atc_status (user_id, is_visiting, is_absent, rating)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id)
        DO UPDATE SET is_visiting = EXCLUDED.is_visiting,
                      is_absent = EXCLUDED.is_absent,
                      rating = EXCLUDED.rating
        "#,
    )
    .bind(user_id)
    .bind(status.is_visiting)
    .bind(status.is_absent)
    .bind(&status.rating)
    .execute(&mut **transaction)
    .await?;

    user_atc_permission::replace(transaction, user_id, &status.permissions).await
}

pub async fn delete_with_permissions(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM public.user_atc_status
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(&mut **transaction)
    .await?;

    user_atc_permission::replace(transaction, user_id, &[]).await?;

    Ok(result.rows_affected() > 0)
}
