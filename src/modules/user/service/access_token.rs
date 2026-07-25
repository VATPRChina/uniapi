use chrono::{Duration, Utc};
use jsonwebtoken::errors::Error as JwtCrateError;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;
use uuid::Uuid;

use crate::settings::JwtAuthentication;

use super::super::models::{IssuedAccessToken, ValidatedAccessToken, ValidatedAuthorizationCode};

#[derive(Clone)]
pub struct AccessTokenService {
    decoding_key: DecodingKey,
    encoding_key: EncodingKey,
    audience: String,
    clients: Vec<crate::settings::JwtClient>,
    first_party_expires_seconds: i64,
    issuer: String,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    iat: i64,
    exp: i64,
    sid: Option<String>,
    client_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    sub: String,
    iat: i64,
    jti: String,
    scope: String,
    sid: String,
    client_id: String,
    iss: String,
    aud: String,
    exp: i64,
    nbf: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthCodeClaims {
    iat: i64,
    client_id: String,
    redirect_uri: String,
    authorization_code: String,
    jti: String,
    iss: String,
    aud: String,
    exp: i64,
    nbf: i64,
}

#[derive(Debug, Error)]
pub enum AccessTokenServiceError {
    #[error("invalid token: {0}")]
    InvalidToken(#[from] JwtCrateError),
}

impl AccessTokenService {
    pub fn new(settings: &JwtAuthentication) -> Self {
        Self {
            decoding_key: DecodingKey::from_ec_pem(settings.public_key.as_bytes())
                .expect("invalid JWT public key"),
            encoding_key: EncodingKey::from_ec_pem(settings.private_key.as_bytes())
                .expect("invalid JWT private key"),
            audience: settings.audience_first_party.clone(),
            clients: settings.clients.clone(),
            first_party_expires_seconds: settings.first_party_expires_seconds,
            issuer: settings.issuer.clone(),
        }
    }

    pub fn validate_access_token(&self, token: &str) -> Result<String, AccessTokenServiceError> {
        Ok(self.validate_access_token_claims(token)?.subject)
    }

    pub fn validate_access_token_claims(
        &self,
        token: &str,
    ) -> Result<ValidatedAccessToken, AccessTokenServiceError> {
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_issuer(&[&self.issuer]);
        validation.validate_aud = false;

        let claims = decode::<Claims>(token, &self.decoding_key, &validation)?.claims;
        Ok(ValidatedAccessToken {
            subject: claims.sub,
            issued_at: claims.iat,
            expires_at: claims.exp,
            session_id: claims.sid,
            client_id: claims.client_id,
        })
    }

    pub fn issue_access_token(
        &self,
        user_id: Uuid,
        user_updated_at: chrono::DateTime<Utc>,
        refresh_token: Ulid,
        client_id: &str,
    ) -> Result<IssuedAccessToken, AccessTokenServiceError> {
        self.issue_token(
            Ulid::from(user_id).to_string(),
            refresh_token.to_string(),
            client_id.to_string(),
            Some(user_updated_at.timestamp()),
        )
    }

    pub fn issue_client_access_token(
        &self,
        client_id: &str,
    ) -> Result<IssuedAccessToken, AccessTokenServiceError> {
        self.issue_token(
            client_id.to_string(),
            Ulid::new().to_string(),
            client_id.to_string(),
            None,
        )
    }

    pub fn issue_auth_code(
        &self,
        code: Ulid,
        client_id: &str,
        redirect_uri: &str,
    ) -> Result<String, AccessTokenServiceError> {
        let now = Utc::now();
        let expires = now + Duration::seconds(self.first_party_expires_seconds);
        Ok(encode(
            &Header::new(Algorithm::ES256),
            &AuthCodeClaims {
                iat: now.timestamp(),
                client_id: client_id.to_string(),
                redirect_uri: redirect_uri.to_string(),
                authorization_code: code.to_string(),
                jti: code.to_string(),
                iss: self.issuer.clone(),
                aud: self.audience.clone(),
                exp: expires.timestamp(),
                nbf: now.timestamp(),
            },
            &self.encoding_key,
        )?)
    }

    pub fn validate_auth_code(
        &self,
        code: &str,
        client_id: &str,
    ) -> Result<Option<ValidatedAuthorizationCode>, AccessTokenServiceError> {
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        let claims = decode::<AuthCodeClaims>(code, &self.decoding_key, &validation)?.claims;
        if claims.client_id != client_id {
            return Ok(None);
        }

        Ok(claims
            .jti
            .parse::<Ulid>()
            .ok()
            .map(|code| ValidatedAuthorizationCode {
                code,
                client_id: claims.client_id,
            }))
    }

    pub fn check_client_exists(&self, client_id: &str) -> bool {
        self.clients
            .iter()
            .any(|client| client.client_id == client_id)
    }

    pub fn check_client_redirect(&self, client_id: &str, redirect_uri: &str) -> bool {
        self.clients.iter().any(|client| {
            client.client_id == client_id
                && client.redirect_uri.iter().any(|uri| uri == redirect_uri)
        })
    }

    pub fn check_client_secret(&self, client_id: &str, client_secret: &str) -> bool {
        self.clients.iter().any(|client| {
            client.client_id == client_id
                && client
                    .client_secret
                    .as_ref()
                    .is_some_and(|secret| secret == client_secret)
        })
    }

    pub fn check_client_can_unsafe_assume_user(&self, client_id: &str) -> bool {
        self.clients
            .iter()
            .any(|client| client.client_id == client_id && client.unsafe_assume_user)
    }

    fn issue_token(
        &self,
        subject: String,
        session_id: String,
        client_id: String,
        updated_at: Option<i64>,
    ) -> Result<IssuedAccessToken, AccessTokenServiceError> {
        let now = Utc::now();
        let expires = now + Duration::seconds(self.first_party_expires_seconds);
        let scope = String::new();
        let token = encode(
            &Header::new(Algorithm::ES256),
            &TokenClaims {
                sub: subject,
                iat: now.timestamp(),
                jti: Ulid::new().to_string(),
                scope: scope.clone(),
                sid: session_id,
                client_id,
                iss: self.issuer.clone(),
                aud: self.audience.clone(),
                exp: expires.timestamp(),
                nbf: now.timestamp(),
                updated_at,
            },
            &self.encoding_key,
        )?;

        Ok(IssuedAccessToken {
            token,
            expires_in: (expires.timestamp() - now.timestamp()) as u32,
            scope,
        })
    }
}
