use chrono::Utc;
use futures::future::try_join_all;
use sqlx::PgPool;
use uuid::Uuid;

use crate::adapter::email::{EmailClient, EmailContent, EmailError};
use crate::modules::user::models::UserSummary;
use crate::modules::user::service::user::{UserService, UserServiceError};
use crate::repository::atc::user_atc_permission::UserAtcPermissionRepositoryExt;
use crate::repository::sheet::sheet_filing::SheetFilingTransactionExt;
use crate::repository::sheet::sheet_filing_answer::{
    SheetAnswerRecord, SheetAnswerSave, SheetFilingAnswerRepositoryExt,
};

use super::models::{
    Training, TrainingApplication, TrainingApplicationResponse, TrainingApplicationSlotSave,
    TrainingSave,
};
use super::repository::training::TrainingRepository;
use super::repository::training_application::{
    TrainingApplicationRepository, TrainingApplicationTransactionRepository,
};
use super::repository::training_application_response::{
    TrainingApplicationResponseRepository, TrainingApplicationResponseTransactionRepository,
};
use super::repository::training_application_slot::TrainingApplicationSlotRepository;

const RECORD_SHEET_ID: &str = "training-record";

#[derive(Clone)]
pub struct TrainingService {
    db: PgPool,
    user: UserService,
}

impl TrainingService {
    pub fn new(db: PgPool, user: UserService) -> Self {
        Self { db, user }
    }

    pub async fn list_active(
        &self,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<Vec<TrainingView>, TrainingServiceError> {
        self.with_filings(
            self.db
                .list_training_active(current_user_id, is_admin)
                .await?,
        )
        .await
    }

    pub async fn list_finished(
        &self,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<Vec<TrainingView>, TrainingServiceError> {
        self.with_filings(
            self.db
                .list_training_finished(current_user_id, is_admin)
                .await?,
        )
        .await
    }

    pub async fn list_by_user(
        &self,
        user_id: Uuid,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<Vec<TrainingView>, TrainingServiceError> {
        if user_id != current_user_id && !is_admin {
            return Err(TrainingServiceError::NotOwned {
                entity: "user",
                id: user_id,
            });
        }
        self.with_filings(self.db.list_training_by_user(user_id).await?)
            .await
    }

    pub async fn find_visible(
        &self,
        id: Uuid,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<TrainingView, TrainingServiceError> {
        let training = self.find(id).await?;
        if training.trainer_id != current_user_id
            && training.trainee_id != current_user_id
            && !is_admin
        {
            return Err(TrainingServiceError::NotOwned {
                entity: "training",
                id,
            });
        }
        self.with_filing(training).await
    }

    pub async fn create(
        &self,
        training: TrainingSave,
        current_user_id: Uuid,
        can_create_for_other_trainer: bool,
    ) -> Result<TrainingView, TrainingServiceError> {
        if training.trainer_id != current_user_id && !can_create_for_other_trainer {
            return Err(TrainingServiceError::CannotCreateForOtherTrainer);
        }
        let training = self.db.create_training(training).await?;
        self.with_filing(training).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        save: TrainingSave,
        current_user_id: Uuid,
        can_manage_all: bool,
    ) -> Result<TrainingView, TrainingServiceError> {
        let training = self.find(id).await?;
        ensure_trainer_access(&training, current_user_id, can_manage_all)?;
        if save.trainer_id != training.trainer_id || save.trainee_id != training.trainee_id {
            return Err(TrainingServiceError::CannotUpdateTrainerTrainee);
        }
        let training = self
            .db
            .update_training(id, save)
            .await?
            .ok_or(TrainingServiceError::NotFound(id))?;
        self.with_filing(training).await
    }

    pub async fn set_record(
        &self,
        id: Uuid,
        answers: &[SheetAnswerSave],
        current_user_id: Uuid,
        can_manage_all: bool,
    ) -> Result<TrainingView, TrainingServiceError> {
        let training = self.find(id).await?;
        ensure_trainer_access(&training, current_user_id, can_manage_all)?;
        let mut transaction = self.db.begin().await?;
        let filing_id = transaction
            .set_sheet_filing(
                RECORD_SHEET_ID,
                training.record_sheet_filing_id,
                training.trainer_id,
                answers,
            )
            .await?;
        transaction.commit().await?;
        let training = self
            .db
            .set_training_record_filing(id, filing_id)
            .await?
            .ok_or(TrainingServiceError::NotFound(id))?;
        self.with_filing(training).await
    }

    pub async fn delete(
        &self,
        id: Uuid,
        current_user_id: Uuid,
        can_manage_all: bool,
    ) -> Result<(), TrainingServiceError> {
        let training = self.find(id).await?;
        ensure_trainer_access(&training, current_user_id, can_manage_all)?;
        if training.start_at <= Utc::now() {
            return Err(TrainingServiceError::CannotDeleteStarted);
        }
        self.db.mark_training_deleted(id).await?;
        Ok(())
    }

    async fn find(&self, id: Uuid) -> Result<Training, TrainingServiceError> {
        self.db
            .find_training_by_id(id)
            .await?
            .ok_or(TrainingServiceError::NotFound(id))
    }

    async fn with_filings(
        &self,
        trainings: Vec<Training>,
    ) -> Result<Vec<TrainingView>, TrainingServiceError> {
        let mut views = Vec::with_capacity(trainings.len());
        for training in trainings {
            views.push(self.with_filing(training).await?);
        }
        Ok(views)
    }

    async fn with_filing(&self, training: Training) -> Result<TrainingView, TrainingServiceError> {
        let record_sheet_filing = match training.record_sheet_filing_id {
            Some(filing_id) => Some(
                self.db
                    .list_sheet_filing_answer_by_filing(filing_id)
                    .await?,
            ),
            None => None,
        };
        let trainer = self
            .user
            .find_summary_by_id(training.trainer_id)
            .await?
            .ok_or(TrainingServiceError::UserNotFound(training.trainer_id))?;
        let trainee = self
            .user
            .find_summary_by_id(training.trainee_id)
            .await?
            .ok_or(TrainingServiceError::UserNotFound(training.trainee_id))?;
        Ok(TrainingView {
            training,
            trainer,
            trainee,
            record_sheet_filing,
        })
    }
}

#[derive(Debug)]
pub struct TrainingView {
    pub training: Training,
    pub trainer: UserSummary,
    pub trainee: UserSummary,
    pub record_sheet_filing: Option<Vec<SheetAnswerRecord>>,
}

fn ensure_trainer_access(
    training: &Training,
    current_user_id: Uuid,
    can_manage_all: bool,
) -> Result<(), TrainingServiceError> {
    if training.trainer_id == current_user_id || can_manage_all {
        return Ok(());
    }
    Err(TrainingServiceError::NotOwned {
        entity: "training",
        id: training.id,
    })
}

#[derive(Clone)]
pub struct TrainingApplicationService {
    db: PgPool,
    email: EmailClient,
    user: UserService,
}

impl TrainingApplicationService {
    pub fn new(db: PgPool, email: EmailClient, user: UserService) -> Self {
        Self { db, email, user }
    }

    pub async fn list(
        &self,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<Vec<TrainingApplicationView>, TrainingApplicationServiceError> {
        let applications = self
            .db
            .list_training_application(current_user_id, is_admin)
            .await?;
        self.with_slots_many(applications).await
    }

    pub async fn find_visible(
        &self,
        id: Uuid,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<TrainingApplicationView, TrainingApplicationServiceError> {
        let application = self
            .db
            .find_training_application_visible_by_id(id, current_user_id, is_admin)
            .await?
            .ok_or(TrainingApplicationServiceError::NotFound(id))?;
        self.with_slots(application).await
    }

    pub async fn create(
        &self,
        trainee_id: Uuid,
        name: &str,
        slots: &[TrainingApplicationSlotSave],
    ) -> Result<TrainingApplicationView, TrainingApplicationServiceError> {
        if !self
            .db
            .has_user_atc_permission_any_by_user_id(trainee_id)
            .await?
        {
            return Err(TrainingApplicationServiceError::ControllerPermissionRequired);
        }
        let mut transaction = self.db.begin().await?;
        let id = transaction
            .create_training_application(trainee_id, name, slots)
            .await?;
        transaction.commit().await?;
        self.find(id).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        current_user_id: Uuid,
        is_admin: bool,
        name: &str,
        slots: &[TrainingApplicationSlotSave],
    ) -> Result<TrainingApplicationView, TrainingApplicationServiceError> {
        let application = self
            .find_visible_entity(id, current_user_id, is_admin)
            .await?;
        let mut transaction = self.db.begin().await?;
        transaction
            .update_training_application(application.id, name, slots)
            .await?;
        transaction.commit().await?;
        self.find(application.id).await
    }

    pub async fn delete(
        &self,
        id: Uuid,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<TrainingApplicationView, TrainingApplicationServiceError> {
        let application = self
            .find_visible_entity(id, current_user_id, is_admin)
            .await?;
        self.db
            .mark_training_application_deleted(application.id)
            .await?;
        self.find(application.id).await
    }

    pub async fn list_responses(
        &self,
        id: Uuid,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<Vec<TrainingApplicationResponseView>, TrainingApplicationServiceError> {
        let application = self
            .find_visible_entity(id, current_user_id, is_admin)
            .await?;
        let responses = self
            .db
            .list_training_application_response(application.id)
            .await?;
        try_join_all(
            responses
                .into_iter()
                .map(|response| self.with_trainer(response)),
        )
        .await
    }

    pub async fn respond(
        &self,
        application_id: Uuid,
        trainer_id: Uuid,
        slot_id: Option<Uuid>,
        comment: &str,
    ) -> Result<TrainingApplicationResponseView, TrainingApplicationServiceError> {
        let application = self
            .db
            .find_training_application_by_id(application_id)
            .await?
            .ok_or(TrainingApplicationServiceError::NotFound(application_id))?;
        if application.train_id.is_some() {
            return Err(TrainingApplicationServiceError::AlreadyAccepted);
        }
        if application.deleted_at.is_some() {
            return Err(TrainingApplicationServiceError::NotFound(application_id));
        }
        let slot = match slot_id {
            Some(slot_id) => Some(
                self.db
                    .find_training_application_slot(application.id, slot_id)
                    .await?
                    .ok_or(TrainingApplicationServiceError::SlotNotFound(slot_id))?,
            ),
            None => None,
        };
        let mut transaction = self.db.begin().await?;
        let response_id = transaction
            .create_training_application_response(&application, trainer_id, slot.as_ref(), comment)
            .await?;
        transaction.commit().await?;
        let response = self
            .db
            .find_training_application_response(response_id)
            .await?
            .ok_or(TrainingApplicationServiceError::ResponseNotFound(
                response_id,
            ))?;
        let trainee = self
            .user
            .find_summary_by_id(application.trainee_id)
            .await?
            .ok_or(TrainingApplicationServiceError::UserNotFound(
                application.trainee_id,
            ))?;
        if let Some(email) = trainee.email.as_deref() {
            self.email
                .send(
                    email,
                    EmailContent::training_application_response(&application, &trainee, &response),
                )
                .await?;
        }
        self.with_trainer(response).await
    }

    async fn find(
        &self,
        id: Uuid,
    ) -> Result<TrainingApplicationView, TrainingApplicationServiceError> {
        let application = self
            .db
            .find_training_application_by_id(id)
            .await?
            .ok_or(TrainingApplicationServiceError::NotFound(id))?;
        self.with_slots(application).await
    }

    async fn find_visible_entity(
        &self,
        id: Uuid,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<TrainingApplication, TrainingApplicationServiceError> {
        self.db
            .find_training_application_visible_by_id(id, current_user_id, is_admin)
            .await?
            .ok_or(TrainingApplicationServiceError::NotFound(id))
    }

    async fn with_slots_many(
        &self,
        applications: Vec<TrainingApplication>,
    ) -> Result<Vec<TrainingApplicationView>, TrainingApplicationServiceError> {
        let mut views = Vec::with_capacity(applications.len());
        for application in applications {
            views.push(self.with_slots(application).await?);
        }
        Ok(views)
    }

    async fn with_slots(
        &self,
        application: TrainingApplication,
    ) -> Result<TrainingApplicationView, TrainingApplicationServiceError> {
        let slots = self
            .db
            .list_training_application_slot(application.id)
            .await?;
        let trainee = self
            .user
            .find_summary_by_id(application.trainee_id)
            .await?
            .ok_or(TrainingApplicationServiceError::UserNotFound(
                application.trainee_id,
            ))?;
        Ok(TrainingApplicationView {
            application,
            trainee,
            slots,
        })
    }

    async fn with_trainer(
        &self,
        response: TrainingApplicationResponse,
    ) -> Result<TrainingApplicationResponseView, TrainingApplicationServiceError> {
        let trainer = self
            .user
            .find_summary_by_id(response.trainer_id)
            .await?
            .ok_or(TrainingApplicationServiceError::UserNotFound(
                response.trainer_id,
            ))?;
        Ok(TrainingApplicationResponseView { response, trainer })
    }
}

#[derive(Debug)]
pub struct TrainingApplicationView {
    pub application: TrainingApplication,
    pub trainee: UserSummary,
    pub slots: Vec<super::models::TrainingApplicationSlot>,
}

#[derive(Debug)]
pub struct TrainingApplicationResponseView {
    pub response: TrainingApplicationResponse,
    pub trainer: UserSummary,
}

#[derive(Debug, thiserror::Error)]
pub enum TrainingServiceError {
    #[error("training {0} not found")]
    NotFound(Uuid),
    #[error("user {0} referenced by a training was not found")]
    UserNotFound(Uuid),
    #[error("{entity} {id} is not owned by the current user")]
    NotOwned { entity: &'static str, id: Uuid },
    #[error("cannot create training for another trainer")]
    CannotCreateForOtherTrainer,
    #[error("cannot update a training's trainer or trainee")]
    CannotUpdateTrainerTrainee,
    #[error("cannot delete a training that has started")]
    CannotDeleteStarted,
    #[error("failed to access training data: {0}")]
    Database(#[from] sqlx::Error),
    #[error("failed to access training user: {0}")]
    User(#[from] UserServiceError),
}

#[derive(Debug, thiserror::Error)]
pub enum TrainingApplicationServiceError {
    #[error("training application {0} not found")]
    NotFound(Uuid),
    #[error("training application response {0} not found")]
    ResponseNotFound(Uuid),
    #[error("training application slot {0} not found")]
    SlotNotFound(Uuid),
    #[error("user {0} referenced by a training application was not found")]
    UserNotFound(Uuid),
    #[error("controller permission is required")]
    ControllerPermissionRequired,
    #[error("training application has already been accepted")]
    AlreadyAccepted,
    #[error("failed to access training application data: {0}")]
    Database(#[from] sqlx::Error),
    #[error("failed to send training application email: {0}")]
    Email(#[from] EmailError),
    #[error("failed to access training application user: {0}")]
    User(#[from] UserServiceError),
}
