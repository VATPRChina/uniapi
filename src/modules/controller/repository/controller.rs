use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AtcControllerPermissionRecord {
    pub user_id: Uuid,
    pub is_visiting: bool,
    pub is_absent: bool,
    pub rating: String,
    pub position_kind_id: Option<String>,
    pub state: Option<String>,
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
        SELECT user_atc_status.user_id,
               user_atc_status.is_visiting,
               user_atc_status.is_absent,
               user_atc_status.rating,
               user_atc_permission.position_kind_id,
               user_atc_permission.state,
               user_atc_permission.solo_expires_at
        FROM public.user_atc_status
        LEFT JOIN public.user_atc_permission ON user_atc_permission.user_id = user_atc_status.user_id
        ORDER BY user_atc_status.user_id, user_atc_permission.position_kind_id
        "#,
        )
        .fetch_all(self)
        .await
    }
}
