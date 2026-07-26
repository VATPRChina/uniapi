use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::error::ApiError;
use crate::modules::sheet::dto::{SheetFieldAnswerDto, SheetRequestField};
use crate::modules::user::dto::UserDto;
use crate::modules::user::models::UserSummary;

use super::models::{
    Training, TrainingApplication, TrainingApplicationResponse, TrainingApplicationSlot,
    TrainingApplicationSlotSave, TrainingSave,
};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TrainingSaveRequest {
    pub name: String,
    pub trainer_id: String,
    pub trainee_id: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

impl TryFrom<TrainingSaveRequest> for TrainingSave {
    type Error = ApiError;

    fn try_from(request: TrainingSaveRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            name: request.name,
            trainer_id: request.trainer_id.parse::<Ulid>()?.into(),
            trainee_id: request.trainee_id.parse::<Ulid>()?.into(),
            start_at: request.start_at,
            end_at: request.end_at,
        })
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TrainingRecordRequest {
    pub request_answers: Vec<SheetRequestField>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TrainingDto {
    pub id: String,
    pub name: String,
    pub trainer_id: String,
    pub trainer: UserDto,
    pub trainee_id: String,
    pub trainee: UserDto,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub record_sheet_filing_id: Option<String>,
    pub record_sheet_filing: Option<Vec<SheetFieldAnswerDto>>,
}

impl TrainingDto {
    pub fn from_entity(
        training: Training,
        trainer: UserSummary,
        trainee: UserSummary,
        record_sheet_filing: Option<Vec<SheetFieldAnswerDto>>,
    ) -> Self {
        Self {
            id: Ulid::from(training.id).to_string(),
            name: training.name,
            trainer_id: Ulid::from(training.trainer_id).to_string(),
            trainer: UserDto::from_user_summary(trainer, true),
            trainee_id: Ulid::from(training.trainee_id).to_string(),
            trainee: UserDto::from_user_summary(trainee, true),
            start_at: training.start_at,
            end_at: training.end_at,
            created_at: training.created_at,
            updated_at: training.updated_at,
            deleted_at: training.deleted_at,
            record_sheet_filing_id: training
                .record_sheet_filing_id
                .map(|id| Ulid::from(id).to_string()),
            record_sheet_filing,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TrainingApplicationCreateRequest {
    pub name: String,
    pub slots: Vec<TrainingApplicationCreateRequestSlot>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TrainingApplicationCreateRequestSlot {
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

impl From<TrainingApplicationCreateRequestSlot> for TrainingApplicationSlotSave {
    fn from(slot: TrainingApplicationCreateRequestSlot) -> Self {
        Self {
            start_at: slot.start_at,
            end_at: slot.end_at,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TrainingApplicationResponseRequest {
    pub slot_id: Option<String>,
    pub comment: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TrainingApplicationDto {
    pub id: String,
    pub trainee_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trainee_email: Option<String>,
    pub trainee: UserDto,
    pub status: TrainingApplicationStatus,
    pub name: String,
    pub train_id: Option<String>,
    pub slots: Vec<TrainingApplicationSlotDto>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TrainingApplicationDto {
    pub fn from_entity(
        application: TrainingApplication,
        trainee: UserSummary,
        slots: Vec<TrainingApplicationSlot>,
        include_trainee_email: bool,
    ) -> Self {
        let status = TrainingApplicationStatus::from_entity(&application, &slots);
        let trainee_email = include_trainee_email
            .then_some(trainee.email.clone())
            .flatten();
        Self {
            id: Ulid::from(application.id).to_string(),
            trainee_id: Ulid::from(application.trainee_id).to_string(),
            trainee_email,
            trainee: UserDto::from_user_summary(trainee, true),
            status,
            name: application.name,
            train_id: application.train_id.map(|id| Ulid::from(id).to_string()),
            slots: slots
                .into_iter()
                .map(TrainingApplicationSlotDto::from)
                .collect(),
            created_at: application.created_at,
            updated_at: application.updated_at,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TrainingApplicationStatus {
    Pending,
    Accepted,
    Rejected,
    Cancelled,
}

impl TrainingApplicationStatus {
    fn from_entity(application: &TrainingApplication, slots: &[TrainingApplicationSlot]) -> Self {
        if application.train_id.is_some() {
            Self::Accepted
        } else if application.deleted_at.is_some() {
            Self::Cancelled
        } else if slots
            .iter()
            .map(|slot| slot.end_at)
            .max()
            .is_some_and(|end_at| end_at < Utc::now())
        {
            Self::Rejected
        } else {
            Self::Pending
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TrainingApplicationSlotDto {
    pub id: String,
    pub application_id: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

impl From<TrainingApplicationSlot> for TrainingApplicationSlotDto {
    fn from(slot: TrainingApplicationSlot) -> Self {
        Self {
            id: Ulid::from(slot.id).to_string(),
            application_id: Ulid::from(slot.application_id).to_string(),
            start_at: slot.start_at,
            end_at: slot.end_at,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TrainingApplicationResponseDto {
    pub id: String,
    pub application_id: String,
    pub trainer_id: String,
    pub trainer: UserDto,
    pub is_accepted: bool,
    pub comment: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TrainingApplicationResponseDto {
    pub fn from_entity(response: TrainingApplicationResponse, trainer: UserSummary) -> Self {
        Self {
            id: Ulid::from(response.id).to_string(),
            application_id: Ulid::from(response.application_id).to_string(),
            trainer_id: Ulid::from(response.trainer_id).to_string(),
            trainer: UserDto::from_user_summary(trainer, true),
            is_accepted: response.slot_id.is_some(),
            comment: response.comment,
            created_at: response.created_at,
            updated_at: response.updated_at,
        }
    }
}
