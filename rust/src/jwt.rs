use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use thiserror::Error;

use crate::settings::JwtAuthentication;

#[derive(Clone)]
pub struct JwtService {
    decoding_key: DecodingKey,
    issuer: String,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    #[serde(rename = "exp")]
    _exp: usize,
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("invalid token: {0}")]
    InvalidToken(#[from] jsonwebtoken::errors::Error),
}

impl JwtService {
    pub fn new(settings: &JwtAuthentication) -> Self {
        Self {
            decoding_key: DecodingKey::from_ec_pem(settings.public_key.as_bytes())
                .expect("invalid JWT public key"),
            issuer: settings.issuer.clone(),
        }
    }

    pub fn validate_access_token(&self, token: &str) -> Result<String, JwtError> {
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_issuer(&[&self.issuer]);
        validation.validate_aud = false;

        Ok(decode::<Claims>(token, &self.decoding_key, &validation)?
            .claims
            .sub)
    }
}
