use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(FromRow)]
pub struct FutureControllerRow {
    pub callsign: String,
    pub name: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

pub trait CompatRepositoryExt<'executor> {
    async fn future_compat_controllers(self) -> Result<Vec<FutureControllerRow>, sqlx::Error>;
}

impl<'executor, E> CompatRepositoryExt<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn future_compat_controllers(self) -> Result<Vec<FutureControllerRow>, sqlx::Error> {
        sqlx::query_as::<_, FutureControllerRow>(
        r#"
        SELECT atc_booking.callsign, "user".full_name AS name, atc_booking.start_at, atc_booking.end_at
        FROM atc_booking
        JOIN "user" ON "user".id = atc_booking.user_id
        WHERE atc_booking.start_at >= now()
        "#,
    )
    .fetch_all(self)
    .await
    }
}
