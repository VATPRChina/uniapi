use sqlx::PgPool;
use uuid::Uuid;

use crate::adapter::moodle::{MoodleClient, MoodleError};
use crate::model::audit_log::AuditLogEntity;
use crate::model::user::{MoodleUser, User, UserSummary};
use crate::model::user_role::UserRole;
use crate::repository::auth::user::{UserRecord, UserRepositoryExt};
use crate::services::audit_log::{AuditLogService, AuditLogServiceError};

#[derive(Clone)]
pub struct UserService {
    db: PgPool,
    moodle: MoodleClient,
    audit_log: AuditLogService,
}

impl UserService {
    pub fn new(db: PgPool, moodle: MoodleClient, audit_log: AuditLogService) -> Self {
        Self {
            db,
            moodle,
            audit_log,
        }
    }

    pub async fn list(&self) -> Result<Vec<UserSummary>, UserServiceError> {
        Ok(self
            .db
            .list_user_details_ordered_by_cid()
            .await?
            .into_iter()
            .map(user_summary)
            .collect())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, UserServiceError> {
        let Some(user) = self.db.find_user_detail_by_id(id).await? else {
            return Ok(None);
        };
        let moodle_user = self.moodle_user(&user.cid).await?;

        Ok(Some(user_model(user, moodle_user)))
    }

    pub async fn set_roles(
        &self,
        id: Uuid,
        roles: Vec<UserRole>,
        operated_by: Uuid,
        can_remove_staff: bool,
    ) -> Result<Option<UserSummary>, UserServiceError> {
        let mut transaction = self.db.begin().await?;
        let Some(before) = (&mut *transaction)
            .find_user_detail_by_id_for_update(id)
            .await?
        else {
            return Ok(None);
        };

        let currently_staff = before
            .roles
            .iter()
            .any(|role| role.parse() == Ok(UserRole::Staff));
        if currently_staff && !roles.contains(&UserRole::Staff) && !can_remove_staff {
            return Err(UserServiceError::RemoveStaffForbidden);
        }

        let roles = roles.into_iter().map(|role| role.to_string()).collect();
        let Some(after) = (&mut *transaction).set_user_roles(id, roles).await? else {
            return Ok(None);
        };
        transaction.commit().await?;

        self.audit_log
            .record(
                AuditLogEntity::UserRole(id, id),
                operated_by,
                Some(&before),
                Some(&after),
            )
            .await?;

        Ok(Some(user_summary(after)))
    }

    pub async fn ensure_moodle_account(&self, id: Uuid) -> Result<Option<User>, UserServiceError> {
        let Some(user) = self.db.find_user_detail_by_id(id).await? else {
            return Ok(None);
        };

        if let Some(moodle_user) = self.moodle.get_user_by_cid(&user.cid).await? {
            tracing::info!(
                user_id = %user.id,
                moodle_user_id = moodle_user.id,
                cid = %user.cid,
                "Moodle user found for CID, skipping user creation"
            );
            return Ok(Some(user_model(
                user,
                Some(MoodleUser { id: moodle_user.id }),
            )));
        }

        tracing::info!(
            user_id = %user.id,
            cid = %user.cid,
            "No Moodle user found for CID, creating user"
        );
        let created_user = self
            .moodle
            .create_user(&user.cid, &user.full_name, user.email.as_deref())
            .await?
            .into_iter()
            .next();

        if let Some(created_user) = created_user {
            tracing::info!(
                user_id = %user.id,
                moodle_user_id = created_user.id,
                moodle_username = %created_user.username,
                cid = %user.cid,
                "Created Moodle user"
            );
            return Ok(Some(user_model(
                user,
                Some(MoodleUser {
                    id: created_user.id,
                }),
            )));
        }

        let moodle_user = self.moodle_user(&user.cid).await?;
        Ok(Some(user_model(user, moodle_user)))
    }

    async fn moodle_user(&self, cid: &str) -> Result<Option<MoodleUser>, UserServiceError> {
        Ok(self
            .moodle
            .get_user_by_cid(cid)
            .await?
            .map(|user| MoodleUser { id: user.id }))
    }
}

fn user_summary(user: UserRecord) -> UserSummary {
    let direct_roles = user
        .roles
        .iter()
        .filter_map(|role| role.parse().ok())
        .collect();

    UserSummary {
        id: user.id,
        cid: user.cid,
        full_name: user.full_name,
        email: user.email,
        direct_roles,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }
}

fn user_model(user: UserRecord, moodle_user: Option<MoodleUser>) -> User {
    let summary = user_summary(user);
    User {
        id: summary.id,
        cid: summary.cid,
        full_name: summary.full_name,
        email: summary.email,
        direct_roles: summary.direct_roles,
        created_at: summary.created_at,
        updated_at: summary.updated_at,
        moodle_user,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UserServiceError {
    #[error("only division director can remove staff role")]
    RemoveStaffForbidden,
    #[error("failed to query user repository: {0}")]
    Database(#[from] sqlx::Error),
    #[error("failed to access Moodle: {0}")]
    Moodle(#[from] MoodleError),
    #[error("failed to record user audit log: {0}")]
    AuditLog(#[from] AuditLogServiceError),
}
