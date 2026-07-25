use serde::{Deserialize, Serialize};

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
