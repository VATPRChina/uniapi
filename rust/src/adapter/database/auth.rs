use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
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

#[derive(FromRow)]
pub struct RefreshSessionRow {
    #[sqlx(try_from = "Uuid")]
    pub token: Ulid,
    pub user_id: Uuid,
    pub user_updated_at: DateTime<Utc>,
    pub expires_in: DateTime<Utc>,
    #[allow(dead_code)]
    pub code: Option<Uuid>,
    pub client_id: String,
    pub updated_at: DateTime<Utc>,
}

pub struct RefreshSessionIssue {
    pub token: Ulid,
    pub code: Option<Ulid>,
}

#[derive(FromRow)]
pub struct UserLoginRow {
    pub id: Uuid,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_device_authorization(
    db: &PgPool,
    device_authorization: NewDeviceAuthorization<'_>,
) -> Result<(), sqlx::Error> {
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
    .execute(db)
    .await?;

    Ok(())
}

pub async fn find_device_authorization_by_user_code(
    db: &PgPool,
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
    .fetch_optional(db)
    .await
}

pub async fn find_device_authorization_for_grant(
    db: &PgPool,
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
    .fetch_optional(db)
    .await
}

pub async fn associate_device_authorization_user(
    db: &PgPool,
    user_code: &str,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE device_authorization SET user_id = $1 WHERE user_code = $2")
        .bind(user_id)
        .bind(user_code)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn delete_device_authorization(
    db: &PgPool,
    device_code: Ulid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM device_authorization WHERE device_code = $1")
        .bind(Uuid::from(device_code))
        .execute(db)
        .await?;

    Ok(())
}

pub async fn issue_refresh_token(
    db: &PgPool,
    user_id: Uuid,
    user_updated_at: DateTime<Utc>,
    expires_in: DateTime<Utc>,
    client_id: &str,
    old_token: Option<Ulid>,
    create_code: bool,
) -> Result<RefreshSessionIssue, sqlx::Error> {
    let token = Ulid::new();
    let code = create_code.then(Ulid::new);

    let mut transaction = db.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO session (token, user_id, user_updated_at, expires_in, code, client_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(Uuid::from(token))
    .bind(user_id)
    .bind(user_updated_at)
    .bind(expires_in)
    .bind(code.map(Uuid::from))
    .bind(client_id)
    .execute(&mut *transaction)
    .await?;

    if let Some(old_token) = old_token {
        sqlx::query("DELETE FROM session WHERE token = $1")
            .bind(Uuid::from(old_token))
            .execute(&mut *transaction)
            .await?;
    }

    sqlx::query("DELETE FROM session WHERE user_id = $1 AND expires_in < now()")
        .bind(user_id)
        .execute(&mut *transaction)
        .await?;
    sqlx::query(
        r#"
        DELETE FROM session
        WHERE user_id = $1
          AND user_updated_at <> (SELECT updated_at FROM "user" WHERE id = $1)
        "#,
    )
    .bind(user_id)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(RefreshSessionIssue { token, code })
}

pub async fn find_refresh_session(
    db: &PgPool,
    token: Ulid,
) -> Result<Option<RefreshSessionRow>, sqlx::Error> {
    sqlx::query_as::<_, RefreshSessionRow>(
        r#"
        SELECT session.token, session.user_id, session.user_updated_at, session.expires_in,
               session.code, session.client_id, "user".updated_at
        FROM session
        JOIN "user" ON "user".id = session.user_id
        WHERE session.token = $1
        "#,
    )
    .bind(Uuid::from(token))
    .fetch_optional(db)
    .await
}

pub async fn find_refresh_session_by_code(
    db: &PgPool,
    code: Ulid,
) -> Result<Option<RefreshSessionRow>, sqlx::Error> {
    sqlx::query_as::<_, RefreshSessionRow>(
        r#"
        SELECT session.token, session.user_id, session.user_updated_at, session.expires_in,
               session.code, session.client_id, "user".updated_at
        FROM session
        JOIN "user" ON "user".id = session.user_id
        WHERE session.code = $1
        "#,
    )
    .bind(Uuid::from(code))
    .fetch_optional(db)
    .await
}

pub async fn clear_session_code(db: &PgPool, code: Ulid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE session SET code = NULL WHERE code = $1")
        .bind(Uuid::from(code))
        .execute(db)
        .await?;

    Ok(())
}

pub async fn upsert_user(
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
