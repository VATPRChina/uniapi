use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::dto::{SheetFieldAnswerDto, SheetRequestField};
use crate::modules::user::dto::{UserDto, UserMoodleInfoDto};
use crate::modules::user::models::UserSummary;

use super::models::{AtcApplication, AtcApplicationStatus as ApplicationStatus};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AtcApplicationRequest {
    pub request_answers: Vec<SheetRequestField>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AtcApplicationReviewRequest {
    pub status: AtcApplicationStatus,
    pub review_answers: Vec<SheetRequestField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AtcApplicationStatus {
    Submitted,
    InWaitlist,
    Approved,
    Rejected,
    Aborted,
}

impl From<ApplicationStatus> for AtcApplicationStatus {
    fn from(status: ApplicationStatus) -> Self {
        match status {
            ApplicationStatus::Submitted => Self::Submitted,
            ApplicationStatus::InWaitlist => Self::InWaitlist,
            ApplicationStatus::Approved => Self::Approved,
            ApplicationStatus::Rejected => Self::Rejected,
            ApplicationStatus::Aborted => Self::Aborted,
        }
    }
}

impl From<AtcApplicationStatus> for ApplicationStatus {
    fn from(status: AtcApplicationStatus) -> Self {
        match status {
            AtcApplicationStatus::Submitted => Self::Submitted,
            AtcApplicationStatus::InWaitlist => Self::InWaitlist,
            AtcApplicationStatus::Approved => Self::Approved,
            AtcApplicationStatus::Rejected => Self::Rejected,
            AtcApplicationStatus::Aborted => Self::Aborted,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AtcApplicationSummaryDto {
    pub id: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_email: Option<String>,
    pub user: UserDto,
    pub applied_at: DateTime<Utc>,
    pub status: AtcApplicationStatus,
}

impl AtcApplicationSummaryDto {
    pub fn from_entity(
        application: AtcApplication,
        user: UserSummary,
        is_admin: bool,
        current_user_id: Uuid,
    ) -> Self {
        let user_email = is_admin.then_some(user.email.clone()).flatten();
        let user = application_user_to_dto(
            user,
            is_admin || application.user_id == current_user_id,
            None,
        );

        Self {
            id: Ulid::from(application.id).to_string(),
            user_id: Ulid::from(application.user_id).to_string(),
            user_email,
            user,
            applied_at: application.applied_at,
            status: application.status.into(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AtcApplicationDto {
    pub id: String,
    pub user_id: String,
    pub user: UserDto,
    pub applied_at: DateTime<Utc>,
    pub status: AtcApplicationStatus,
    pub application_filing_answers: Vec<SheetFieldAnswerDto>,
    pub review_filing_answers: Option<Vec<SheetFieldAnswerDto>>,
}

impl AtcApplicationDto {
    pub fn from_entity(
        application: AtcApplication,
        user: UserSummary,
        is_admin: bool,
        current_user_id: Uuid,
        application_filing_answers: Vec<SheetFieldAnswerDto>,
        review_filing_answers: Option<Vec<SheetFieldAnswerDto>>,
        moodle_account: Option<UserMoodleInfoDto>,
    ) -> Self {
        let user = application_user_to_dto(
            user,
            is_admin || application.user_id == current_user_id,
            moodle_account,
        );

        Self {
            id: Ulid::from(application.id).to_string(),
            user_id: Ulid::from(application.user_id).to_string(),
            user,
            applied_at: application.applied_at,
            status: application.status.into(),
            application_filing_answers,
            review_filing_answers,
        }
    }
}

fn application_user_to_dto(
    user: UserSummary,
    show_full_name: bool,
    moodle_account: Option<UserMoodleInfoDto>,
) -> UserDto {
    let mut dto = UserDto::from_user_summary(user, show_full_name);
    dto.moodle_account = moodle_account;
    dto
}
