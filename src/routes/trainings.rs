use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::user_role::UserRole;
use crate::repository::atc_training::training::{
    self as training_repository, TrainingRecord, TrainingSave,
};
use crate::repository::sheet::sheet_field::{self as sheet_field_repository};
use crate::repository::sheet::sheet_filing_answer::{
    self as sheet_filing_answer_repository, SheetAnswerSave,
};
use crate::repository::sheet::{
    sheet as sheet_repository, sheet_filing as sheet_filing_repository,
};
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    create_training,
    list_active,
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

#[utoipa::path(get, path = "api/atc/trainings/active", tag = "ATC", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<TrainingDto>)))]
async fn list_active(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<TrainingDto>>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    trainings_to_dto(
        &services,
        training_repository::list_active(
            services.db(),
            user_id,
            current_user.has_role(UserRole::ControllerTrainingMentor),
        )
        .await?,
    )
    .await
    .map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/finished", tag = "ATC", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<TrainingDto>)))]
async fn list_finished(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<TrainingDto>>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    trainings_to_dto(
        &services,
        training_repository::list_finished(
            services.db(),
            user_id,
            current_user.has_role(UserRole::ControllerTrainingMentor),
        )
        .await?,
    )
    .await
    .map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/{id}", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "Training ULID")), responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn get_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<TrainingDto>, ApiError> {
    let training = find_training(&services, &id).await?;
    validate_ownership(&current_user, &training, false)?;
    training_to_dto(&services, training).await.map(Json)
}

#[utoipa::path(post, path = "api/atc/trainings", tag = "ATC", security(("oauth2" = [])), request_body = TrainingSaveRequest, responses((status = 200, description = "Successful response", body = TrainingDto)))]
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

    let training = training_repository::create(services.db(), training).await?;
    training_to_dto(&services, training).await.map(Json)
}

#[utoipa::path(put, path = "api/atc/trainings/{id}", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "Training ULID")), request_body = TrainingSaveRequest, responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn update_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<TrainingSaveRequest>,
) -> Result<Json<TrainingDto>, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let id = parse_ulid_uuid("id", &id)?;
    let training = training_repository::find_by_id(services.db(), id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;
    validate_ownership(&current_user, &training, true)?;
    let save = TrainingSave::try_from(request)?;
    if save.trainer_id != training.trainer_id || save.trainee_id != training.trainee_id {
        return Err(ApiError::CannotUpdateTrainerTrainee);
    }

    let training = training_repository::update(services.db(), id, save)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;
    training_to_dto(&services, training).await.map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/record-sheet", tag = "ATC", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = SheetDto)))]
async fn get_record_sheet(State(services): State<Services>) -> Result<Json<SheetDto>, ApiError> {
    sheet_repository::ensure(services.db(), RECORD_SHEET_ID, "Training Record Sheet").await?;
    let sheet = sheet_repository::find(services.db(), RECORD_SHEET_ID)
        .await?
        .ok_or(ApiError::not_found("sheet", "training-record"))?;
    let fields = sheet_field_repository::list(services.db(), RECORD_SHEET_ID).await?;

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

#[utoipa::path(put, path = "api/atc/trainings/{id}/record", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "Training ULID")), request_body = TrainingRecordRequest, responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn set_record_sheet(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<TrainingRecordRequest>,
) -> Result<Json<TrainingDto>, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let id = parse_ulid_uuid("id", &id)?;
    let training = training_repository::find_by_id(services.db(), id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;
    validate_ownership(&current_user, &training, true)?;

    let answers = request
        .request_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    let mut transaction = services.db().begin().await?;
    let filing_id = sheet_filing_repository::set(
        &mut transaction,
        RECORD_SHEET_ID,
        training.record_sheet_filing_id,
        training.trainer_id,
        &answers,
    )
    .await?;
    transaction.commit().await?;
    let training = training_repository::set_record_filing(services.db(), id, filing_id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;

    training_to_dto(&services, training).await.map(Json)
}

#[utoipa::path(delete, path = "api/atc/trainings/{id}", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "Training ULID")), responses((status = 204, description = "No content")))]
async fn delete_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let id = parse_ulid_uuid("id", &id)?;
    let training = training_repository::find_by_id(services.db(), id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;
    validate_ownership(&current_user, &training, true)?;
    if training.start_at <= Utc::now() {
        return Err(ApiError::CannotDeleteStartedTraining);
    }

    training_repository::mark_deleted(services.db(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn find_training(services: &Services, id: &str) -> Result<TrainingRecord, ApiError> {
    training_repository::find_by_id(services.db(), parse_ulid_uuid("id", id)?)
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
            sheet_filing_answer_repository::list_by_filing(services.db(), filing_id)
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
