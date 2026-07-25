use chrono::{Duration, Utc};
use rand::RngExt;
use sqlx::PgPool;
use ulid::Ulid;
use uuid::Uuid;

use super::super::models::{
    DeviceAuthorizationConfirmation, DeviceAuthorizationGrant, IssuedDeviceAuthorization,
};
use super::super::repository::device_authorization::{
    DeviceAuthorizationRepository, NewDeviceAuthorization,
};

const USER_CODE_ALPHABET: &[u8] = b"BCDFGHJKLMNPQRSTVWXZ";

#[derive(Clone)]
pub struct DeviceAuthorizationService {
    db: PgPool,
    expires_seconds: i64,
}

impl DeviceAuthorizationService {
    pub fn new(db: PgPool, expires_seconds: i64) -> Self {
        Self {
            db,
            expires_seconds,
        }
    }

    pub async fn create(&self, client_id: &str) -> Result<IssuedDeviceAuthorization, sqlx::Error> {
        let device_code = Ulid::new();
        let user_code = random_user_code();
        let expires_at = Utc::now() + Duration::seconds(self.expires_seconds);

        self.db
            .create_device_authorization(NewDeviceAuthorization {
                device_code,
                user_code: &user_code,
                expires_at,
                client_id,
            })
            .await?;

        Ok(IssuedDeviceAuthorization {
            device_code,
            user_code,
            expires_at,
        })
    }

    pub async fn find_by_user_code(
        &self,
        user_code: &str,
    ) -> Result<Option<DeviceAuthorizationConfirmation>, sqlx::Error> {
        self.db
            .find_device_authorization_by_user_code(user_code)
            .await
    }

    pub async fn find_for_grant(
        &self,
        device_code: Ulid,
    ) -> Result<Option<DeviceAuthorizationGrant>, sqlx::Error> {
        self.db
            .find_device_authorization_for_grant(device_code)
            .await
    }

    pub async fn associate_user(&self, user_code: &str, user_id: Uuid) -> Result<(), sqlx::Error> {
        self.db
            .associate_device_authorization_user(user_code, user_id)
            .await
    }

    pub async fn delete(&self, device_code: Ulid) -> Result<(), sqlx::Error> {
        self.db.delete_device_authorization(device_code).await
    }
}

fn random_user_code() -> String {
    let mut rng = rand::rng();
    (0..8)
        .map(|_| {
            let index = rng.random_range(0..USER_CODE_ALPHABET.len());
            USER_CODE_ALPHABET[index] as char
        })
        .collect()
}

pub fn normalize_user_code(user_code: Option<&str>) -> String {
    user_code
        .unwrap_or_default()
        .to_uppercase()
        .chars()
        .filter(|char| USER_CODE_ALPHABET.contains(&(*char as u8)))
        .collect()
}
