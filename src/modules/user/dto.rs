use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::modules::user::models::{UserRole, role_closure, role_closure_from_strings};

use super::models::{User, UserSummary};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AuthorizeQuery {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DeviceConfirmQuery {
    pub user_code: Option<String>,
    pub confirm: Option<bool>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct LoginQuery {
    pub state: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct VatsimCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DeviceAuthorizationRequest {
    pub client_id: String,
    #[allow(dead_code)]
    pub scope: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct DeviceAuthorizationResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_uri_complete: Option<String>,
    pub expires_in: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AccessTokenRequest {
    #[serde(default)]
    pub grant_type: String,
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub device_code: String,
    #[serde(default)]
    pub refresh_token: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub code_verifier: String,
    #[serde(default)]
    pub client_secret: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UnsafeAssumeUserRequest {
    pub id: Option<String>,
    pub cid: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub scope: String,
}

fn direct_roles_to_dto(roles: &[String]) -> Vec<UserRole> {
    roles
        .iter()
        .filter_map(|role| role.parse::<UserRole>().ok())
        .collect()
}

fn roles_to_dto(roles: &[String]) -> Vec<UserRole> {
    let mut roles = role_closure_from_strings(roles.iter().map(String::as_str))
        .into_iter()
        .collect::<Vec<_>>();
    roles.sort();
    roles
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserMoodleInfoDto {
    pub id: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserDto {
    pub id: String,
    pub cid: String,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub roles: Vec<UserRole>,
    pub direct_roles: Vec<UserRole>,
    pub moodle_account: Option<UserMoodleInfoDto>,
}

impl UserDto {
    pub fn from_role_strings(
        id: Uuid,
        cid: String,
        full_name: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        roles: Vec<String>,
    ) -> Self {
        Self {
            id: Ulid::from(id).to_string(),
            cid,
            full_name,
            created_at,
            updated_at,
            roles: roles_to_dto(&roles),
            direct_roles: direct_roles_to_dto(&roles),
            moodle_account: None,
        }
    }

    pub fn from_user_summary(user: UserSummary, show_full_name: bool) -> Self {
        let direct_roles = user.direct_roles;
        let roles = role_closure(direct_roles.iter().copied())
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        Self {
            id: Ulid::from(user.id).to_string(),
            cid: user.cid,
            full_name: if show_full_name {
                user.full_name
            } else {
                String::new()
            },
            created_at: user.created_at,
            updated_at: user.updated_at,
            roles,
            direct_roles: direct_roles.into_iter().sorted().collect(),
            moodle_account: None,
        }
    }

    pub fn from_user(user: User, show_full_name: bool) -> Self {
        let moodle_account = user.moodle_user.map(|user| UserMoodleInfoDto {
            id: user.id.to_string(),
        });
        let mut dto = Self::from_user_summary(
            UserSummary {
                id: user.id,
                cid: user.cid,
                full_name: user.full_name,
                email: user.email,
                direct_roles: user.direct_roles,
                created_at: user.created_at,
                updated_at: user.updated_at,
            },
            show_full_name,
        );
        dto.moodle_account = moodle_account;
        dto
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TokenDto {
    pub user: UserDto,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
