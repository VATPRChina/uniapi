use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Training {
    pub id: Uuid,
    pub name: String,
    pub trainer_id: Uuid,
    pub trainee_id: Uuid,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub record_sheet_filing_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct TrainingSave {
    pub name: String,
    pub trainer_id: Uuid,
    pub trainee_id: Uuid,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TrainingApplication {
    pub id: Uuid,
    pub trainee_id: Uuid,
    pub name: String,
    pub train_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TrainingApplicationSlot {
    pub id: Uuid,
    pub application_id: Uuid,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TrainingApplicationSlotSave {
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TrainingApplicationResponse {
    pub id: Uuid,
    pub application_id: Uuid,
    pub trainer_id: Uuid,
    pub slot_id: Option<Uuid>,
    pub comment: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
