use chrono::{DateTime, Utc};
use sea_query::{Expr, Iden, OnConflict, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::Serialize;
use sqlx::FromRow;
use ulid::Ulid;
use uuid::Uuid;

#[derive(Clone, Copy, Iden)]
enum User {
    Table,
    Id,
    Cid,
    FullName,
    Email,
    CreatedAt,
    UpdatedAt,
    Roles,
}

const USER_COLUMNS: [User; 7] = [
    User::Id,
    User::Cid,
    User::FullName,
    User::Email,
    User::CreatedAt,
    User::UpdatedAt,
    User::Roles,
];

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserRecord {
    pub id: Uuid,
    pub cid: String,
    pub full_name: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub roles: Vec<String>,
}

pub trait UserRepositoryExt<'executor> {
    async fn find_user_by_id(self, id: Uuid) -> Result<Option<UserRecord>, sqlx::Error>;

    async fn find_user_detail_by_id(self, id: Uuid) -> Result<Option<UserRecord>, sqlx::Error>;

    async fn find_user_moodle_provision_by_id(
        self,
        id: Uuid,
    ) -> Result<Option<UserRecord>, sqlx::Error>;

    async fn find_user_detail_by_id_for_update(
        self,
        id: Uuid,
    ) -> Result<Option<UserRecord>, sqlx::Error>;

    async fn list_user_details_ordered_by_cid(self) -> Result<Vec<UserRecord>, sqlx::Error>;

    async fn upsert_user_assumed_user(
        self,
        id: Uuid,
        cid: &str,
        full_name: &str,
        email: Option<&str>,
        roles: Vec<String>,
    ) -> Result<UserRecord, sqlx::Error>;

    async fn set_user_roles(
        self,
        id: Uuid,
        roles: Vec<String>,
    ) -> Result<Option<UserRecord>, sqlx::Error>;

    async fn upsert_user_login(
        self,
        cid: &str,
        full_name: &str,
        email: &str,
    ) -> Result<UserRecord, sqlx::Error>;
}

impl<'executor, E> UserRepositoryExt<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn find_user_by_id(self, id: Uuid) -> Result<Option<UserRecord>, sqlx::Error> {
        let query = Query::select()
            .columns(USER_COLUMNS)
            .from(User::Table)
            .and_where(Expr::col(User::Id).eq(id))
            .to_owned();
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, UserRecord, _>(&sql, values)
            .fetch_optional(self)
            .await
    }

    async fn find_user_detail_by_id(self, id: Uuid) -> Result<Option<UserRecord>, sqlx::Error> {
        let query = Query::select()
            .columns(USER_COLUMNS)
            .from(User::Table)
            .and_where(Expr::col(User::Id).eq(id))
            .to_owned();
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, UserRecord, _>(&sql, values)
            .fetch_optional(self)
            .await
    }

    async fn find_user_moodle_provision_by_id(
        self,
        id: Uuid,
    ) -> Result<Option<UserRecord>, sqlx::Error> {
        let query = Query::select()
            .columns(USER_COLUMNS)
            .from(User::Table)
            .and_where(Expr::col(User::Id).eq(id))
            .to_owned();
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, UserRecord, _>(&sql, values)
            .fetch_optional(self)
            .await
    }

    async fn find_user_detail_by_id_for_update(
        self,
        id: Uuid,
    ) -> Result<Option<UserRecord>, sqlx::Error> {
        let query = Query::select()
            .columns(USER_COLUMNS)
            .from(User::Table)
            .and_where(Expr::col(User::Id).eq(id))
            .lock_exclusive()
            .to_owned();
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, UserRecord, _>(&sql, values)
            .fetch_optional(self)
            .await
    }

    async fn list_user_details_ordered_by_cid(self) -> Result<Vec<UserRecord>, sqlx::Error> {
        let query = Query::select()
            .columns(USER_COLUMNS)
            .from(User::Table)
            .order_by(User::Cid, Order::Asc)
            .to_owned();
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, UserRecord, _>(&sql, values)
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
    ) -> Result<UserRecord, sqlx::Error> {
        let query = Query::insert()
            .into_table(User::Table)
            .columns([
                User::Id,
                User::Cid,
                User::FullName,
                User::Email,
                User::Roles,
            ])
            .values_panic([
                id.into(),
                cid.into(),
                full_name.into(),
                email.into(),
                roles.into(),
            ])
            .on_conflict(
                OnConflict::column(User::Cid)
                    .update_columns([User::FullName, User::Email, User::Roles])
                    .value(User::UpdatedAt, Expr::current_timestamp())
                    .to_owned(),
            )
            .returning(Query::returning().columns(USER_COLUMNS))
            .to_owned();
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, UserRecord, _>(&sql, values)
            .fetch_one(self)
            .await
    }

    async fn set_user_roles(
        self,
        id: Uuid,
        roles: Vec<String>,
    ) -> Result<Option<UserRecord>, sqlx::Error> {
        let query = Query::update()
            .table(User::Table)
            .value(User::Roles, roles)
            .value(User::UpdatedAt, Expr::current_timestamp())
            .and_where(Expr::col(User::Id).eq(id))
            .returning(Query::returning().columns(USER_COLUMNS))
            .to_owned();
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, UserRecord, _>(&sql, values)
            .fetch_optional(self)
            .await
    }

    async fn upsert_user_login(
        self,
        cid: &str,
        full_name: &str,
        email: &str,
    ) -> Result<UserRecord, sqlx::Error> {
        let query = Query::insert()
            .into_table(User::Table)
            .columns([
                User::Id,
                User::Cid,
                User::FullName,
                User::Email,
                User::Roles,
            ])
            .values_panic([
                Uuid::from(Ulid::new()).into(),
                cid.into(),
                full_name.into(),
                email.into(),
                Vec::<String>::new().into(),
            ])
            .on_conflict(
                OnConflict::column(User::Cid)
                    .update_columns([User::FullName, User::Email])
                    .value(User::UpdatedAt, Expr::current_timestamp())
                    .to_owned(),
            )
            .returning(Query::returning().columns(USER_COLUMNS))
            .to_owned();
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, UserRecord, _>(&sql, values)
            .fetch_one(self)
            .await
    }
}
