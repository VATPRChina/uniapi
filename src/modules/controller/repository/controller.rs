use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AtcControllerPermissionRecord {
    pub user_id: Uuid,
    pub is_visiting: Option<bool>,
    pub is_absent: Option<bool>,
    pub rating: Option<String>,
    pub position_kind_id: String,
    pub state: String,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

pub(crate) trait ControllerRepository<'executor> {
    async fn list_atc_controllers(self) -> Result<Vec<AtcControllerPermissionRecord>, sqlx::Error>;
}

impl<'executor, E> ControllerRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_atc_controllers(self) -> Result<Vec<AtcControllerPermissionRecord>, sqlx::Error> {
        sqlx::query_as::<_, AtcControllerPermissionRecord>(
            r#"
        SELECT user_atc_permission.user_id,
               user_atc_status.is_visiting,
               user_atc_status.is_absent,
               user_atc_status.rating,
               user_atc_permission.position_kind_id,
               user_atc_permission.state,
               user_atc_permission.solo_expires_at
        FROM public.user_atc_permission
        LEFT JOIN public.user_atc_status ON user_atc_status.user_id = user_atc_permission.user_id
        ORDER BY user_atc_permission.user_id, user_atc_permission.position_kind_id
        "#,
        )
        .fetch_all(self)
        .await
    }
}
