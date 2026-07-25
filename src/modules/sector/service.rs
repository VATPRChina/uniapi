use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::user::service::user::{UserService, UserServiceError};

use super::models::SectorPermission;
use super::repository::SectorRepository;

#[derive(Clone)]
pub struct SectorService {
    db: PgPool,
    user: UserService,
}

impl SectorService {
    pub fn new(db: PgPool, user: UserService) -> Self {
        Self { db, user }
    }

    pub async fn current_permission(
        &self,
        user_id: Uuid,
    ) -> Result<SectorPermission, SectorServiceError> {
        let user = self
            .user
            .find_summary_by_id(user_id)
            .await?
            .ok_or(SectorServiceError::UserNotFound(user_id))?;
        Ok(SectorPermission {
            has_permission: self.db.user_sector_can_online(user.id, &user.cid).await?,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SectorServiceError {
    #[error("user {0} not found")]
    UserNotFound(Uuid),
    #[error("failed to query sector data: {0}")]
    Database(#[from] sqlx::Error),
    #[error("failed to access sector user: {0}")]
    User(#[from] UserServiceError),
}
