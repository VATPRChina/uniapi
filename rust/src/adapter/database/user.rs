use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserDetailRecord {
    pub id: Uuid,
    pub cid: String,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub roles: Vec<String>,
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserRecord>(
        r#"
        SELECT id, roles
        FROM public."user"
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn find_detail_by_id(
    db: &PgPool,
    id: Uuid,
) -> Result<Option<UserDetailRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserDetailRecord>(
        r#"
        SELECT id, cid, full_name, created_at, updated_at, roles
        FROM public."user"
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}
