use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::user_role::UserRole;
use crate::repository::atc_training::training::TrainingRepositoryExt;
use crate::repository::atc_training::training::{TrainingRecord, TrainingSave};
use crate::repository::sheet::sheet::SheetRepositoryExt;
use crate::repository::sheet::sheet_field::SheetFieldRepositoryExt;
use crate::repository::sheet::sheet_filing::SheetFilingTransactionExt;
use crate::repository::sheet::sheet_filing_answer::SheetAnswerSave;
use crate::repository::sheet::sheet_filing_answer::SheetFilingAnswerRepositoryExt;
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    create_training,
    list_active,
    list_by_user,
    list_finished,
    get_record_sheet,
    get_training,
    update_training,
    delete_training,
    set_record_sheet
))]
pub(crate) struct ApiDoc;

const RECORD_SHEET_ID: &str = "training-record";

pub fn build_training_routes() -> Router<Services> {
    Router::new()
        .route("/", axum::routing::post(create_training))
        .route("/active", get(list_active))
        .route("/by-user/{user_id}", get(list_by_user))
        .route("/finished", get(list_finished))
        .route("/record-sheet", get(get_record_sheet))
        .route(
            "/{id}",
            get(get_training)
                .put(update_training)
                .delete(delete_training),
        )
        .route("/{id}/record", axum::routing::put(set_record_sheet))
}

#[utoipa::path(get, path = "api/atc/trainings/active", tag = "Training", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<TrainingDto>)))]
async fn list_active(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<TrainingDto>>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    trainings_to_dto(
        &services,
        services
            .db()
            .list_training_active(user_id, is_training_history_admin(&current_user))
            .await?,
    )
    .await
    .map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/by-user/{userId}", tag = "Training", security(("oauth2" = [])), params(("userId" = String, Path, description = "User ULID")), responses((status = 200, description = "Successful response", body = Vec<TrainingDto>)))]
async fn list_by_user(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<TrainingDto>>, ApiError> {
    let user_id = parse_ulid_uuid("user_id", &user_id)?;
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    if current_user_id != user_id && !is_training_history_admin(&current_user) {
        return Err(ApiError::NotOwned {
            entity: "user".to_string(),
            id: user_id.to_string(),
        });
    }

    trainings_to_dto(
        &services,
        services.db().list_training_by_user(user_id).await?,
    )
    .await
    .map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/finished", tag = "Training", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<TrainingDto>)))]
async fn list_finished(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<TrainingDto>>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    trainings_to_dto(
        &services,
        services
            .db()
            .list_training_finished(user_id, is_training_history_admin(&current_user))
            .await?,
    )
    .await
    .map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/{id}", tag = "Training", security(("oauth2" = [])), params(("id" = String, Path, description = "Training ULID")), responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn get_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<TrainingDto>, ApiError> {
    let training = find_training(&services, &id).await?;
    validate_view_access(&current_user, &training)?;
    training_to_dto(&services, training).await.map(Json)
}

#[utoipa::path(post, path = "api/atc/trainings", tag = "Training", security(("oauth2" = [])), request_body = TrainingSaveRequest, responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn create_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<TrainingSaveRequest>,
) -> Result<Json<TrainingDto>, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let training = TrainingSave::try_from(request)?;
    if training.trainer_id != current_user_id
        && !current_user.has_role(UserRole::ControllerTrainingDirectorAssistant)
    {
        return Err(ApiError::CannotCreateForOtherTrainer);
    }

    let training = services.db().create_training(training).await?;
    training_to_dto(&services, training).await.map(Json)
}

#[utoipa::path(put, path = "api/atc/trainings/{id}", tag = "Training", security(("oauth2" = [])), params(("id" = String, Path, description = "Training ULID")), request_body = TrainingSaveRequest, responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn update_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<TrainingSaveRequest>,
) -> Result<Json<TrainingDto>, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let id = parse_ulid_uuid("id", &id)?;
    let training = services
        .db()
        .find_training_by_id(id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;
    validate_ownership(&current_user, &training, true)?;
    let save = TrainingSave::try_from(request)?;
    if save.trainer_id != training.trainer_id || save.trainee_id != training.trainee_id {
        return Err(ApiError::CannotUpdateTrainerTrainee);
    }

    let training = services
        .db()
        .update_training(id, save)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;
    training_to_dto(&services, training).await.map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/record-sheet", tag = "Training", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = SheetDto)))]
async fn get_record_sheet(State(services): State<Services>) -> Result<Json<SheetDto>, ApiError> {
    services
        .db()
        .ensure_sheet(RECORD_SHEET_ID, "Training Record Sheet")
        .await?;
    let sheet = services
        .db()
        .find_sheet(RECORD_SHEET_ID)
        .await?
        .ok_or(ApiError::not_found("sheet", "training-record"))?;
    let fields = services.db().list_sheet_field(RECORD_SHEET_ID).await?;

    Ok(Json(SheetDto {
        id: sheet.id,
        name: sheet.name,
        fields: fields
            .into_iter()
            .filter(|field| !field.is_deleted)
            .map(SheetFieldDto::from)
            .collect(),
    }))
}

#[utoipa::path(put, path = "api/atc/trainings/{id}/record", tag = "Training", security(("oauth2" = [])), params(("id" = String, Path, description = "Training ULID")), request_body = TrainingRecordRequest, responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn set_record_sheet(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<TrainingRecordRequest>,
) -> Result<Json<TrainingDto>, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let id = parse_ulid_uuid("id", &id)?;
    let training = services
        .db()
        .find_training_by_id(id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;
    validate_ownership(&current_user, &training, true)?;

    let answers = request
        .request_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    let mut transaction = services.db().begin().await?;
    let filing_id = (&mut transaction)
        .set_sheet_filing(
            RECORD_SHEET_ID,
            training.record_sheet_filing_id,
            training.trainer_id,
            &answers,
        )
        .await?;
    transaction.commit().await?;
    let training = services
        .db()
        .set_training_record_filing(id, filing_id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;

    training_to_dto(&services, training).await.map(Json)
}

#[utoipa::path(delete, path = "api/atc/trainings/{id}", tag = "Training", security(("oauth2" = [])), params(("id" = String, Path, description = "Training ULID")), responses((status = 204, description = "No content")))]
async fn delete_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let id = parse_ulid_uuid("id", &id)?;
    let training = services
        .db()
        .find_training_by_id(id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;
    validate_ownership(&current_user, &training, true)?;
    if training.start_at <= Utc::now() {
        return Err(ApiError::CannotDeleteStartedTraining);
    }

    services.db().mark_training_deleted(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn find_training(services: &Services, id: &str) -> Result<TrainingRecord, ApiError> {
    services
        .db()
        .find_training_by_id(parse_ulid_uuid("id", id)?)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))
}

async fn trainings_to_dto(
    services: &Services,
    trainings: Vec<TrainingRecord>,
) -> Result<Vec<TrainingDto>, ApiError> {
    let mut dtos = Vec::with_capacity(trainings.len());
    for training in trainings {
        dtos.push(training_to_dto(services, training).await?);
    }
    Ok(dtos)
}

async fn training_to_dto(
    services: &Services,
    training: TrainingRecord,
) -> Result<TrainingDto, ApiError> {
    let record_sheet_filing = match training.record_sheet_filing_id {
        Some(filing_id) => Some(
            services
                .db()
                .list_sheet_filing_answer_by_filing(filing_id)
                .await?
                .into_iter()
                .map(SheetFieldAnswerDto::from)
                .collect(),
        ),
        None => None,
    };

    Ok(TrainingDto::from_record(training, record_sheet_filing))
}

fn validate_ownership(
    current_user: &CurrentUser,
    training: &TrainingRecord,
    require_trainer: bool,
) -> Result<(), ApiError> {
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    if training.trainer_id == current_user_id {
        return Ok(());
    }
    if training.trainee_id == current_user_id && !require_trainer {
        return Ok(());
    }
    if current_user.has_role(UserRole::ControllerTrainingMentor) {
        return Ok(());
    }

    Err(ApiError::NotOwned {
        entity: "training".to_string(),
        id: training.id.to_string(),
    })
}

fn validate_view_access(
    current_user: &CurrentUser,
    training: &TrainingRecord,
) -> Result<(), ApiError> {
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    if training.trainer_id == current_user_id
        || training.trainee_id == current_user_id
        || is_training_history_admin(current_user)
    {
        return Ok(());
    }

    Err(ApiError::NotOwned {
        entity: "training".to_string(),
        id: training.id.to_string(),
    })
}

fn is_training_history_admin(current_user: &CurrentUser) -> bool {
    current_user.has_role(UserRole::ControllerTrainingDirectorAssistant)
        || current_user.has_role(UserRole::ControllerTrainingMentor)
}
