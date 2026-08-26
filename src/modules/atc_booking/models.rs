use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AtcBooking {
    pub id: Uuid,
    pub user_id: Uuid,
    pub callsign: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub remarks: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub event_position_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct AtcBookingSave {
    pub callsign: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub remarks: Option<String>,
}
