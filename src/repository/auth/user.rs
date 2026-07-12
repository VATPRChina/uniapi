use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserDetailRecord {
    pub id: Uuid,
    pub cid: String,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserMoodleProvisionRecord {
    pub id: Uuid,
    pub cid: String,
    pub full_name: String,
    pub email: Option<String>,
}

#[derive(FromRow)]
pub struct UserLoginRow {
    pub id: Uuid,
    pub updated_at: DateTime<Utc>,
}

pub trait UserRepositoryExt<'executor> {
    async fn find_user_by_id(self, id: Uuid) -> Result<Option<UserRecord>, sqlx::Error>;

    async fn find_user_detail_by_id(
        self,
        id: Uuid,
    ) -> Result<Option<UserDetailRecord>, sqlx::Error>;

    async fn find_user_moodle_provision_by_id(
        self,
        id: Uuid,
    ) -> Result<Option<UserMoodleProvisionRecord>, sqlx::Error>;

    async fn find_user_detail_by_id_for_update(
        self,
        id: Uuid,
    ) -> Result<Option<UserDetailRecord>, sqlx::Error>;

    async fn list_user_details_ordered_by_cid(self) -> Result<Vec<UserDetailRecord>, sqlx::Error>;

    async fn upsert_user_assumed_user(
        self,
        id: Uuid,
        cid: &str,
        full_name: &str,
        email: Option<&str>,
        roles: Vec<String>,
    ) -> Result<UserLoginRow, sqlx::Error>;

    async fn set_user_roles(
        self,
        id: Uuid,
        roles: Vec<String>,
    ) -> Result<Option<UserDetailRecord>, sqlx::Error>;

    async fn upsert_user_login(
        self,
        cid: &str,
        full_name: &str,
        email: &str,
    ) -> Result<UserLoginRow, sqlx::Error>;
}

impl<'executor, E> UserRepositoryExt<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn find_user_by_id(self, id: Uuid) -> Result<Option<UserRecord>, sqlx::Error> {
        sqlx::query_as::<_, UserRecord>(
            r#"
        SELECT id, roles
        FROM public."user"
        WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_optional(self)
        .await
    }
    async fn find_user_detail_by_id(
        self,
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
        .fetch_optional(self)
        .await
    }
    async fn find_user_moodle_provision_by_id(
        self,
        id: Uuid,
    ) -> Result<Option<UserMoodleProvisionRecord>, sqlx::Error> {
        sqlx::query_as::<_, UserMoodleProvisionRecord>(
            r#"
        SELECT id, cid, full_name, email
        FROM public."user"
        WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_optional(self)
        .await
    }
    async fn find_user_detail_by_id_for_update(
        self,
        id: Uuid,
    ) -> Result<Option<UserDetailRecord>, sqlx::Error> {
        sqlx::query_as::<_, UserDetailRecord>(
            r#"
        SELECT id, cid, full_name, created_at, updated_at, roles
        FROM public."user"
        WHERE id = $1
        FOR UPDATE
        "#,
        )
        .bind(id)
        .fetch_optional(self)
        .await
    }
    async fn list_user_details_ordered_by_cid(self) -> Result<Vec<UserDetailRecord>, sqlx::Error> {
        sqlx::query_as::<_, UserDetailRecord>(
            r#"
        SELECT id, cid, full_name, created_at, updated_at, roles
        FROM public."user"
        ORDER BY cid
        "#,
        )
        .fetch_all(self)
        .await
    }
    async fn upsert_user_assumed_user(
        self,
        id: Uuid,
        cid: &str,
        full_name: &str,
        email: Option<&str>,
        roles: Vec<String>,
    ) -> Result<UserLoginRow, sqlx::Error> {
        sqlx::query_as::<_, UserLoginRow>(
            r#"
        INSERT INTO public."user" (id, cid, full_name, email, roles)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (cid) DO UPDATE
        SET full_name = EXCLUDED.full_name,
            email = EXCLUDED.email,
            roles = EXCLUDED.roles,
            updated_at = CURRENT_TIMESTAMP
        RETURNING id, updated_at
        "#,
        )
        .bind(id)
        .bind(cid)
        .bind(full_name)
        .bind(email)
        .bind(roles)
        .fetch_one(self)
        .await
    }
    async fn set_user_roles(
        self,
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
        .fetch_optional(self)
        .await
    }
    async fn upsert_user_login(
        self,
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
        .fetch_one(self)
        .await
    }
}
