use std::collections::HashSet;

use chrono::{DateTime, Utc};
use ulid::Ulid;
use uuid::Uuid;

use crate::model::user_role::UserRole;

#[derive(Debug, Clone)]
pub struct UserSummary {
    pub id: Uuid,
    pub cid: String,
    pub full_name: String,
    pub email: Option<String>,
    pub direct_roles: HashSet<UserRole>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub cid: String,
    pub full_name: String,
    pub email: Option<String>,
    pub direct_roles: HashSet<UserRole>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub moodle_user: Option<MoodleUser>,
}

#[derive(Debug, Clone, Copy)]
pub struct MoodleUser {
    pub id: i64,
}

pub struct IssuedAccessToken {
    pub token: String,
    pub expires_in: u32,
    pub scope: String,
}

pub struct ValidatedAuthorizationCode {
    pub code: Ulid,
    pub client_id: String,
}

pub struct ValidatedAccessToken {
    pub subject: String,
    pub issued_at: i64,
    pub expires_at: i64,
    pub session_id: Option<String>,
    pub client_id: String,
}

#[derive(sqlx::FromRow)]
pub struct RefreshToken {
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

pub struct IssuedRefreshToken {
    pub token: Ulid,
    pub code: Option<Ulid>,
}

#[derive(sqlx::FromRow)]
pub struct DeviceAuthorizationConfirmation {
    pub device_code: Uuid,
    pub user_code: String,
    pub expires_at: DateTime<Utc>,
    pub user_id: Option<Uuid>,
}

#[derive(sqlx::FromRow)]
pub struct DeviceAuthorizationGrant {
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

pub struct IssuedDeviceAuthorization {
    pub device_code: Ulid,
    pub user_code: String,
    pub expires_at: DateTime<Utc>,
}
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct AuthenticationState {
    pub auth_type: AuthenticationStateType,
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub user_code: Option<String>,
    pub state: Option<String>,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationStateType {
    Code,
    Device,
}
