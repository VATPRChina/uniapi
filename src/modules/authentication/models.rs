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
