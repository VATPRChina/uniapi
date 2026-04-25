use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use reqwest::Url;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::settings::VatsimAuthentication;

#[derive(Clone)]
pub struct VatsimAuthClient {
    http: reqwest::Client,
    settings: VatsimAuthentication,
}

#[derive(Debug, Error)]
pub enum VatsimAuthError {
    #[error("invalid VATSIM auth URL: {0}")]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

impl VatsimAuthClient {
    pub fn new(settings: VatsimAuthentication) -> Self {
        Self {
            http: reqwest::Client::new(),
            settings,
        }
    }

    pub fn authorization_url(
        &self,
        state: &str,
        code_challenge: &str,
    ) -> Result<String, VatsimAuthError> {
        let mut url = Url::parse(&self.settings.endpoint)?;
        url.path_segments_mut()
            .expect("VATSIM endpoint should be a base URL")
            .extend(["oauth", "authorize"]);
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.settings.client_id)
            .append_pair("redirect_uri", &self.settings.redirect_uri)
            .append_pair("state", state)
            .append_pair("code_challenge", code_challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("scope", "full_name email");
        Ok(url.to_string())
    }

    pub async fn get_token(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<VatsimTokenResponse, VatsimAuthError> {
        let mut url = Url::parse(&self.settings.endpoint)?;
        url.path_segments_mut()
            .expect("VATSIM endpoint should be a base URL")
            .extend(["oauth", "token"]);

        Ok(self
            .http
            .post(url)
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", &self.settings.client_id),
                ("client_secret", &self.settings.client_secret),
                ("redirect_uri", &self.settings.redirect_uri),
                ("code", code),
                ("code_verifier", code_verifier),
                ("scope", "full_name email vatsim_details"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<VatsimTokenResponse>()
            .await?)
    }

    pub async fn get_user(
        &self,
        access_token: &str,
    ) -> Result<VatsimUserResponse, VatsimAuthError> {
        let mut url = Url::parse(&self.settings.endpoint)?;
        url.path_segments_mut()
            .expect("VATSIM endpoint should be a base URL")
            .extend(["api", "user"]);

        Ok(self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await?
            .error_for_status()?
            .json::<VatsimUserResponse>()
            .await?)
    }
}

pub fn generate_pkce() -> (String, String) {
    let mut random = [0_u8; 32];
    rand::rng().fill_bytes(&mut random);
    let verifier = URL_SAFE_NO_PAD.encode(random);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    (challenge, verifier)
}

#[derive(Debug, Deserialize)]
pub struct VatsimTokenResponse {
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct VatsimUserResponse {
    pub data: VatsimUserData,
}

#[derive(Debug, Deserialize)]
pub struct VatsimUserData {
    pub cid: String,
    pub personal: VatsimUserPersonal,
}

#[derive(Debug, Deserialize)]
pub struct VatsimUserPersonal {
    #[serde(rename = "name_full")]
    pub full_name: String,
    pub email: String,
}
