use crate::settings::VatsimAuthentication;

use super::repository::{
    VatsimAuthClient, VatsimAuthError, VatsimTokenResponse, VatsimUserResponse, generate_pkce,
};

#[derive(Clone)]
pub struct AuthenticationService {
    vatsim: VatsimAuthClient,
}

impl AuthenticationService {
    pub fn new(settings: VatsimAuthentication) -> Self {
        Self {
            vatsim: VatsimAuthClient::new(settings),
        }
    }

    pub fn begin_login(&self, state: &str) -> Result<(String, String), VatsimAuthError> {
        let (challenge, verifier) = generate_pkce();
        Ok((self.vatsim.authorization_url(state, &challenge)?, verifier))
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        verifier: &str,
    ) -> Result<VatsimTokenResponse, VatsimAuthError> {
        self.vatsim.get_token(code, verifier).await
    }

    pub async fn user(&self, access_token: &str) -> Result<VatsimUserResponse, VatsimAuthError> {
        self.vatsim.get_user(access_token).await
    }
}
