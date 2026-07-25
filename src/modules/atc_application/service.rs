use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::modules::audit_log::models::AuditLogEntity;
use crate::modules::audit_log::service::{AuditLogService, AuditLogServiceError};
use crate::modules::sheet::models::{SheetAnswer, SheetAnswerSave};
use crate::modules::sheet::repository::sheet_filing::SheetFilingTransactionRepository;
use crate::modules::sheet::repository::sheet_filing_answer::SheetFilingAnswerRepository;
use crate::modules::sheet::service::{SheetAnswerView, SheetService, SheetServiceError};
use crate::modules::user::models::UserSummary;
use crate::modules::user::service::user::{UserService, UserServiceError};

use super::models::{AtcApplication, AtcApplicationStatus};
use super::repository::AtcApplicationRepository;

const APPLICATION_SHEET_ID: &str = "atc-application";
const REVIEW_SHEET_ID: &str = "atc-application-review";

#[derive(Clone)]
pub struct AtcApplicationService {
    db: PgPool,
    audit_log: AuditLogService,
    user: UserService,
    sheet: SheetService,
}

impl AtcApplicationService {
    pub fn new(
        db: PgPool,
        audit_log: AuditLogService,
        user: UserService,
        sheet: SheetService,
    ) -> Self {
        Self {
            db,
            audit_log,
            user,
            sheet,
        }
    }

    pub async fn list(
        &self,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<Vec<AtcApplicationView>, AtcApplicationServiceError> {
        let applications = self
            .db
            .list_atc_application()
            .await?
            .into_iter()
            .filter(|application| is_admin || application.user_id == current_user_id)
            .collect::<Vec<_>>();
        let mut result = Vec::with_capacity(applications.len());
        for application in applications {
            result.push(self.with_user(application).await?);
        }

        Ok(result)
    }

    pub async fn create(
        &self,
        current_user_id: Uuid,
        answers: &[SheetAnswerSave],
    ) -> Result<AtcApplicationView, AtcApplicationServiceError> {
        if self
            .db
            .count_atc_application_active_by_user(current_user_id)
            .await?
            > 0
        {
            return Err(AtcApplicationServiceError::AlreadyExists);
        }

        let mut transaction = self.db.begin().await?;
        let filing_id = transaction
            .set_sheet_filing(APPLICATION_SHEET_ID, None, current_user_id, answers)
            .await?;
        let application = (&mut *transaction)
            .create_atc_application(current_user_id, filing_id)
            .await?;
        let after = application_audit_snapshot(&mut transaction, &application).await?;
        transaction.commit().await?;
        self.record_audit_log(None, &after, current_user_id).await?;

        self.with_user(application).await
    }

    pub async fn find_visible(
        &self,
        id: Uuid,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<AtcApplicationView, AtcApplicationServiceError> {
        let application = self
            .db
            .find_atc_application_by_id(id)
            .await?
            .ok_or(AtcApplicationServiceError::NotFound(id))?;
        ensure_visible(&application, current_user_id, is_admin)?;

        self.with_user(application).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        current_user_id: Uuid,
        is_admin: bool,
        answers: &[SheetAnswerSave],
    ) -> Result<AtcApplicationView, AtcApplicationServiceError> {
        let mut transaction = self.db.begin().await?;
        let application = (&mut *transaction)
            .find_atc_application_by_id_for_update(id)
            .await?
            .ok_or(AtcApplicationServiceError::NotFound(id))?;
        ensure_visible(&application, current_user_id, is_admin)?;
        if application.status != AtcApplicationStatus::Submitted {
            return Err(AtcApplicationServiceError::CannotUpdate);
        }
        let before = application_audit_snapshot(&mut transaction, &application).await?;

        transaction
            .set_sheet_filing(
                APPLICATION_SHEET_ID,
                Some(application.application_filing_id),
                current_user_id,
                answers,
            )
            .await?;
        let application = (&mut *transaction)
            .find_atc_application_by_id_for_update(application.id)
            .await?
            .ok_or(AtcApplicationServiceError::NotFound(id))?;
        let after = application_audit_snapshot(&mut transaction, &application).await?;
        transaction.commit().await?;
        self.record_audit_log(Some(&before), &after, current_user_id)
            .await?;

        self.with_user(application).await
    }

    pub async fn review(
        &self,
        id: Uuid,
        current_user_id: Uuid,
        status: AtcApplicationStatus,
        answers: &[SheetAnswerSave],
    ) -> Result<AtcApplicationView, AtcApplicationServiceError> {
        let mut transaction = self.db.begin().await?;
        let application = (&mut *transaction)
            .find_atc_application_by_id_for_update(id)
            .await?
            .ok_or(AtcApplicationServiceError::NotFound(id))?;
        let before = application_audit_snapshot(&mut transaction, &application).await?;
        let filing_id = transaction
            .set_sheet_filing(
                REVIEW_SHEET_ID,
                application.review_filing_id,
                current_user_id,
                answers,
            )
            .await?;
        let application = (&mut *transaction)
            .set_atc_application_review(id, status.as_db_str(), filing_id)
            .await?
            .ok_or(AtcApplicationServiceError::NotFound(id))?;
        let after = application_audit_snapshot(&mut transaction, &application).await?;
        transaction.commit().await?;
        self.record_audit_log(Some(&before), &after, current_user_id)
            .await?;

        self.with_user(application).await
    }

    pub async fn filing_answers(
        &self,
        application: &AtcApplication,
    ) -> Result<(Vec<SheetAnswerView>, Option<Vec<SheetAnswerView>>), AtcApplicationServiceError>
    {
        let application_answers = self
            .sheet
            .filing_answers(application.application_filing_id)
            .await?;
        let review_answers = match application.review_filing_id {
            Some(filing_id) => Some(self.sheet.filing_answers(filing_id).await?),
            None => None,
        };

        Ok((application_answers, review_answers))
    }

    pub async fn ensure_moodle_account(
        &self,
        user_id: Uuid,
    ) -> Result<(), AtcApplicationServiceError> {
        self.user
            .ensure_moodle_account(user_id)
            .await?
            .ok_or(AtcApplicationServiceError::UserNotFound(user_id))?;
        Ok(())
    }

    pub async fn moodle_account(
        &self,
        user_id: Uuid,
    ) -> Result<Option<i64>, AtcApplicationServiceError> {
        Ok(self
            .user
            .find_by_id(user_id)
            .await?
            .ok_or(AtcApplicationServiceError::UserNotFound(user_id))?
            .moodle_user
            .map(|user| user.id))
    }

    async fn with_user(
        &self,
        application: AtcApplication,
    ) -> Result<AtcApplicationView, AtcApplicationServiceError> {
        let user = self
            .user
            .find_summary_by_id(application.user_id)
            .await?
            .ok_or(AtcApplicationServiceError::UserNotFound(
                application.user_id,
            ))?;

        Ok(AtcApplicationView { application, user })
    }

    async fn record_audit_log(
        &self,
        before: Option<&AtcApplicationAuditSnapshot>,
        after: &AtcApplicationAuditSnapshot,
        operated_by: Uuid,
    ) -> Result<(), AtcApplicationServiceError> {
        self.audit_log
            .record(
                AuditLogEntity::AtcApplication(after.application.id),
                operated_by,
                before,
                Some(after),
            )
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AtcApplicationView {
    pub application: AtcApplication,
    pub user: UserSummary,
}

fn ensure_visible(
    application: &AtcApplication,
    current_user_id: Uuid,
    is_admin: bool,
) -> Result<(), AtcApplicationServiceError> {
    if !is_admin && application.user_id != current_user_id {
        return Err(AtcApplicationServiceError::NotFound(application.id));
    }

    Ok(())
}

#[derive(Serialize)]
struct AtcApplicationAuditSnapshot {
    application: AtcApplication,
    application_filing_answers: Vec<SheetAnswer>,
    review_filing_answers: Option<Vec<SheetAnswer>>,
}

async fn application_audit_snapshot(
    transaction: &mut Transaction<'_, Postgres>,
    application: &AtcApplication,
) -> Result<AtcApplicationAuditSnapshot, sqlx::Error> {
    let application_filing_answers = (&mut **transaction)
        .list_sheet_filing_answer_by_filing_in_transaction(application.application_filing_id)
        .await?;
    let review_filing_answers = match application.review_filing_id {
        Some(filing_id) => Some(
            (&mut **transaction)
                .list_sheet_filing_answer_by_filing_in_transaction(filing_id)
                .await?,
        ),
        None => None,
    };

    Ok(AtcApplicationAuditSnapshot {
        application: application.clone(),
        application_filing_answers,
        review_filing_answers,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum AtcApplicationServiceError {
    #[error("ATC application already exists")]
    AlreadyExists,
    #[error("ATC application cannot be updated at its current status")]
    CannotUpdate,
    #[error("ATC application {0} not found")]
    NotFound(Uuid),
    #[error("user {0} referenced by an ATC application was not found")]
    UserNotFound(Uuid),
    #[error("failed to access ATC application data: {0}")]
    Database(#[from] sqlx::Error),
    #[error("failed to record ATC application audit log: {0}")]
    AuditLog(#[from] AuditLogServiceError),
    #[error("failed to access ATC application user: {0}")]
    User(#[from] UserServiceError),
    #[error("failed to access ATC application sheet: {0}")]
    Sheet(#[from] SheetServiceError),
}
