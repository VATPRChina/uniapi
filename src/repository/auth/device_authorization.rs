use chrono::{DateTime, Utc};
use sqlx::FromRow;
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NewDeviceAuthorization<'a> {
    pub device_code: Ulid,
    pub user_code: &'a str,
    pub expires_at: DateTime<Utc>,
    pub client_id: &'a str,
}

#[derive(FromRow)]
pub struct DeviceAuthorizationConfirmRow {
    pub device_code: Uuid,
    pub user_code: String,
    pub expires_at: DateTime<Utc>,
    pub user_id: Option<Uuid>,
}

#[derive(FromRow)]
pub struct DeviceAuthorizationGrantRow {
    #[allow(dead_code)]
    #[sqlx(try_from = "Uuid")]
    pub device_code: Ulid,
    #[allow(dead_code)]
    pub user_code: String,
    pub expires_at: DateTime<Utc>,
    pub client_id: String,
    pub user_id: Option<Uuid>,
    pub user_updated_at: Option<DateTime<Utc>>,
}

pub trait DeviceAuthorizationRepositoryExt<'executor> {
    async fn create_device_authorization(
        self,
        device_authorization: NewDeviceAuthorization<'_>,
    ) -> Result<(), sqlx::Error>;

    async fn find_device_authorization_by_user_code(
        self,
        user_code: &str,
    ) -> Result<Option<DeviceAuthorizationConfirmRow>, sqlx::Error>;

    async fn find_device_authorization_for_grant(
        self,
        device_code: Ulid,
    ) -> Result<Option<DeviceAuthorizationGrantRow>, sqlx::Error>;

    async fn associate_device_authorization_user(
        self,
        user_code: &str,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error>;

    async fn delete_device_authorization(self, device_code: Ulid) -> Result<(), sqlx::Error>;
}

impl<'executor, E> DeviceAuthorizationRepositoryExt<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn create_device_authorization(
        self,
        device_authorization: NewDeviceAuthorization<'_>,
    ) -> Result<(), sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "src/repository/auth/device_authorization.rs",
            "modifying data"
        );

        sqlx::query(
            r#"
        INSERT INTO device_authorization (device_code, user_code, expires_at, client_id)
        VALUES ($1, $2, $3, $4)
        "#,
        )
        .bind(Uuid::from(device_authorization.device_code))
        .bind(device_authorization.user_code)
        .bind(device_authorization.expires_at)
        .bind(device_authorization.client_id)
        .execute(self)
        .await?;

        Ok(())
    }
    async fn find_device_authorization_by_user_code(
        self,
        user_code: &str,
    ) -> Result<Option<DeviceAuthorizationConfirmRow>, sqlx::Error> {
        sqlx::query_as::<_, DeviceAuthorizationConfirmRow>(
            r#"
        SELECT device_code, user_code, expires_at, user_id
        FROM device_authorization
        WHERE user_code = $1
        "#,
        )
        .bind(user_code)
        .fetch_optional(self)
        .await
    }
    async fn find_device_authorization_for_grant(
        self,
        device_code: Ulid,
    ) -> Result<Option<DeviceAuthorizationGrantRow>, sqlx::Error> {
        sqlx::query_as::<_, DeviceAuthorizationGrantRow>(
            r#"
        SELECT device_authorization.device_code, device_authorization.user_code,
               device_authorization.expires_at, device_authorization.client_id,
               device_authorization.user_id, "user".updated_at AS user_updated_at
        FROM device_authorization
        LEFT JOIN "user" ON "user".id = device_authorization.user_id
        WHERE device_authorization.device_code = $1
        "#,
        )
        .bind(Uuid::from(device_code))
        .fetch_optional(self)
        .await
    }
    async fn associate_device_authorization_user(
        self,
        user_code: &str,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        tracing::info!(
            operation = "associate_user",
            repository = "src/repository/auth/device_authorization.rs",
            "modifying data"
        );

        sqlx::query("UPDATE device_authorization SET user_id = $1 WHERE user_code = $2")
            .bind(user_id)
            .bind(user_code)
            .execute(self)
            .await?;

        Ok(())
    }
    async fn delete_device_authorization(self, device_code: Ulid) -> Result<(), sqlx::Error> {
        tracing::info!(
            operation = "delete",
            repository = "src/repository/auth/device_authorization.rs",
            "modifying data"
        );

        sqlx::query("DELETE FROM device_authorization WHERE device_code = $1")
            .bind(Uuid::from(device_code))
            .execute(self)
            .await?;

        Ok(())
    }
}
