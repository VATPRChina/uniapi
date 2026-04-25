use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct UserRecord {
    pub id: Uuid,
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
