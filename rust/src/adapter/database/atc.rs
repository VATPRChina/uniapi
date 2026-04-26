use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AtcControllerPermissionRecord {
    pub user_id: Uuid,
    pub user_cid: String,
    pub user_full_name: String,
    pub user_created_at: DateTime<Utc>,
    pub user_updated_at: DateTime<Utc>,
    pub user_roles: Vec<String>,
    pub is_visiting: Option<bool>,
    pub is_absent: Option<bool>,
    pub rating: Option<String>,
    pub position_kind_id: String,
    pub state: String,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

pub async fn list_controllers(
    db: &PgPool,
) -> Result<Vec<AtcControllerPermissionRecord>, sqlx::Error> {
    sqlx::query_as::<_, AtcControllerPermissionRecord>(
        r#"
        SELECT user_atc_permission.user_id,
               "user".cid AS user_cid,
               "user".full_name AS user_full_name,
               "user".created_at AS user_created_at,
               "user".updated_at AS user_updated_at,
               "user".roles AS user_roles,
               user_atc_status.is_visiting,
               user_atc_status.is_absent,
               user_atc_status.rating,
               user_atc_permission.position_kind_id,
               user_atc_permission.state,
               user_atc_permission.solo_expires_at
        FROM public.user_atc_permission
        INNER JOIN public."user" ON "user".id = user_atc_permission.user_id
        LEFT JOIN public.user_atc_status ON user_atc_status.user_id = user_atc_permission.user_id
        ORDER BY "user".cid, user_atc_permission.position_kind_id
        "#,
    )
    .fetch_all(db)
    .await
}
