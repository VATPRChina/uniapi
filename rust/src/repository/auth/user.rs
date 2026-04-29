use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use ulid::Ulid;
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

#[derive(FromRow)]
pub struct UserLoginRow {
    pub id: Uuid,
    pub updated_at: DateTime<Utc>,
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

pub async fn list_details_ordered_by_cid(
    db: &PgPool,
) -> Result<Vec<UserDetailRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserDetailRecord>(
        r#"
        SELECT id, cid, full_name, created_at, updated_at, roles
        FROM public."user"
        ORDER BY cid
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn find_detail_by_cid(
    db: &PgPool,
    cid: &str,
) -> Result<Option<UserDetailRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserDetailRecord>(
        r#"
        SELECT id, cid, full_name, created_at, updated_at, roles
        FROM public."user"
        WHERE cid = $1
        "#,
    )
    .bind(cid)
    .fetch_optional(db)
    .await
}

pub async fn create_assumed_user(
    db: &PgPool,
    id: Uuid,
    cid: &str,
) -> Result<UserDetailRecord, sqlx::Error> {
    sqlx::query_as::<_, UserDetailRecord>(
        r#"
        INSERT INTO public."user" (id, cid, full_name, email, roles)
        VALUES ($1, $2, $2, NULL, $3)
        RETURNING id, cid, full_name, created_at, updated_at, roles
        "#,
    )
    .bind(id)
    .bind(cid)
    .bind(Vec::<String>::new())
    .fetch_one(db)
    .await
}

pub async fn set_roles(
    db: &PgPool,
    id: Uuid,
    roles: Vec<String>,
) -> Result<Option<UserDetailRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserDetailRecord>(
        r#"
        UPDATE public."user"
        SET roles = $2, updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING id, cid, full_name, created_at, updated_at, roles
        "#,
    )
    .bind(id)
    .bind(roles)
    .fetch_optional(db)
    .await
}

pub async fn upsert_login(
    db: &PgPool,
    cid: &str,
    full_name: &str,
    email: &str,
) -> Result<UserLoginRow, sqlx::Error> {
    sqlx::query_as::<_, UserLoginRow>(
        r#"
        INSERT INTO "user" (id, cid, full_name, email, roles)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (cid) DO UPDATE
        SET full_name = EXCLUDED.full_name,
            email = EXCLUDED.email,
            updated_at = CURRENT_TIMESTAMP
        RETURNING id, updated_at
        "#,
    )
    .bind(Uuid::from(Ulid::new()))
    .bind(cid)
    .bind(full_name)
    .bind(email)
    .bind(Vec::<String>::new())
    .fetch_one(db)
    .await
}
