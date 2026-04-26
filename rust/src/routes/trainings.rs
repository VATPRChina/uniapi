use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::{
    adapter::database::{
        sheet::{self as sheet_repository, SheetAnswerRecord, SheetAnswerSave, SheetFieldRecord},
        training::{self as training_repository, TrainingRecord, TrainingSave},
    },
    auth::CurrentUser,
    models::user_role::{UserRole, role_closure_from_strings},
    services::Services,
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    create_training,
    list_active,
    list_finished,
    list_by_user,
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
        .route("/by-user/{user_id}", get(list_by_user))
        .route("/record-sheet", get(get_record_sheet))
        .route(
            "/{id}",
            get(get_training)
                .put(update_training)
                .delete(delete_training),
        )
        .route("/{id}/record", axum::routing::put(set_record_sheet))
}

#[utoipa::path(get, path = "api/atc/trainings/active", tag = "ATC", security(("bearerAuth" = [])), responses((status = 200, description = "Successful response", body = Vec<TrainingDto>)))]
async fn list_active(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<TrainingDto>>, TrainingRouteError> {
    let user_id = current_user
        .user_id
        .ok_or(TrainingRouteError::Unauthorized)?;
    trainings_to_dto(
        &services,
        training_repository::list_active(
            services.db(),
            user_id,
            current_user.has_role(UserRole::ControllerTrainingMentor),
        )
        .await
        .map_err(TrainingRouteError::Database)?,
    )
    .await
    .map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/finished", tag = "ATC", security(("bearerAuth" = [])), responses((status = 200, description = "Successful response", body = Vec<TrainingDto>)))]
async fn list_finished(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<TrainingDto>>, TrainingRouteError> {
    let user_id = current_user
        .user_id
        .ok_or(TrainingRouteError::Unauthorized)?;
    trainings_to_dto(
        &services,
        training_repository::list_finished(
            services.db(),
            user_id,
            current_user.has_role(UserRole::ControllerTrainingMentor),
        )
        .await
        .map_err(TrainingRouteError::Database)?,
    )
    .await
    .map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/by-user/{user_id}", tag = "ATC", security(("bearerAuth" = [])), params(("user_id" = String, Path, description = "User ULID")), responses((status = 200, description = "Successful response", body = Vec<TrainingDto>)))]
async fn list_by_user(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<TrainingDto>>, TrainingRouteError> {
    let user_id = parse_ulid_uuid(&user_id)?;
    let current_user_id = current_user
        .user_id
        .ok_or(TrainingRouteError::Unauthorized)?;
    if user_id != current_user_id && !current_user.has_role(UserRole::ControllerTrainingMentor) {
        return Err(TrainingRouteError::NotOwned);
    }

    trainings_to_dto(
        &services,
        training_repository::list_by_trainee(services.db(), user_id)
            .await
            .map_err(TrainingRouteError::Database)?,
    )
    .await
    .map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/{id}", tag = "ATC", security(("bearerAuth" = [])), params(("id" = String, Path, description = "Training ULID")), responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn get_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<TrainingDto>, TrainingRouteError> {
    let training = find_training(&services, &id).await?;
    validate_ownership(&current_user, &training, false)?;
    training_to_dto(&services, training).await.map(Json)
}

#[utoipa::path(post, path = "api/atc/trainings", tag = "ATC", security(("bearerAuth" = [])), responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn create_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<TrainingSaveRequest>,
) -> Result<Json<TrainingDto>, TrainingRouteError> {
    require_role(&current_user, UserRole::ControllerTrainingMentor)?;
    let current_user_id = current_user
        .user_id
        .ok_or(TrainingRouteError::Unauthorized)?;
    let training = TrainingSave::try_from(request)?;
    if training.trainer_id != current_user_id
        && !current_user.has_role(UserRole::ControllerTrainingDirectorAssistant)
    {
        return Err(TrainingRouteError::CannotCreateForOtherTrainer);
    }

    let training = training_repository::create(services.db(), training)
        .await
        .map_err(TrainingRouteError::Database)?;
    training_to_dto(&services, training).await.map(Json)
}

#[utoipa::path(put, path = "api/atc/trainings/{id}", tag = "ATC", security(("bearerAuth" = [])), params(("id" = String, Path, description = "Training ULID")), responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn update_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<TrainingSaveRequest>,
) -> Result<Json<TrainingDto>, TrainingRouteError> {
    require_role(&current_user, UserRole::ControllerTrainingMentor)?;
    let id = parse_ulid_uuid(&id)?;
    let training = training_repository::find_by_id(services.db(), id)
        .await
        .map_err(TrainingRouteError::Database)?
        .ok_or(TrainingRouteError::NotFound)?;
    validate_ownership(&current_user, &training, true)?;
    let save = TrainingSave::try_from(request)?;
    if save.trainer_id != training.trainer_id || save.trainee_id != training.trainee_id {
        return Err(TrainingRouteError::CannotUpdateTrainerTrainee);
    }

    let training = training_repository::update(services.db(), id, save)
        .await
        .map_err(TrainingRouteError::Database)?
        .ok_or(TrainingRouteError::NotFound)?;
    training_to_dto(&services, training).await.map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/record-sheet", tag = "ATC", security(("bearerAuth" = [])), responses((status = 200, description = "Successful response", body = SheetDto)))]
async fn get_record_sheet(
    State(services): State<Services>,
) -> Result<Json<SheetDto>, TrainingRouteError> {
    sheet_repository::ensure_sheet(services.db(), RECORD_SHEET_ID, "Training Record Sheet")
        .await
        .map_err(TrainingRouteError::Database)?;
    let sheet = sheet_repository::find_sheet(services.db(), RECORD_SHEET_ID)
        .await
        .map_err(TrainingRouteError::Database)?
        .ok_or(TrainingRouteError::SheetNotFound)?;
    let fields = sheet_repository::list_fields(services.db(), RECORD_SHEET_ID)
        .await
        .map_err(TrainingRouteError::Database)?;

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

#[utoipa::path(put, path = "api/atc/trainings/{id}/record", tag = "ATC", security(("bearerAuth" = [])), params(("id" = String, Path, description = "Training ULID")), responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn set_record_sheet(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<TrainingRecordRequest>,
) -> Result<Json<TrainingDto>, TrainingRouteError> {
    require_role(&current_user, UserRole::ControllerTrainingMentor)?;
    let id = parse_ulid_uuid(&id)?;
    let training = training_repository::find_by_id(services.db(), id)
        .await
        .map_err(TrainingRouteError::Database)?
        .ok_or(TrainingRouteError::NotFound)?;
    validate_ownership(&current_user, &training, true)?;

    let answers = request
        .request_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    let mut transaction = services
        .db()
        .begin()
        .await
        .map_err(TrainingRouteError::Database)?;
    let filing_id = sheet_repository::set_filing(
        &mut transaction,
        RECORD_SHEET_ID,
        training.record_sheet_filing_id,
        training.trainer_id,
        &answers,
    )
    .await
    .map_err(TrainingRouteError::Database)?;
    transaction
        .commit()
        .await
        .map_err(TrainingRouteError::Database)?;
    let training = training_repository::set_record_filing(services.db(), id, filing_id)
        .await
        .map_err(TrainingRouteError::Database)?
        .ok_or(TrainingRouteError::NotFound)?;

    training_to_dto(&services, training).await.map(Json)
}

#[utoipa::path(delete, path = "api/atc/trainings/{id}", tag = "ATC", security(("bearerAuth" = [])), params(("id" = String, Path, description = "Training ULID")), responses((status = 204, description = "No content")))]
async fn delete_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<StatusCode, TrainingRouteError> {
    require_role(&current_user, UserRole::ControllerTrainingMentor)?;
    let id = parse_ulid_uuid(&id)?;
    let training = training_repository::find_by_id(services.db(), id)
        .await
        .map_err(TrainingRouteError::Database)?
        .ok_or(TrainingRouteError::NotFound)?;
    validate_ownership(&current_user, &training, true)?;
    if training.start_at <= Utc::now() {
        return Err(TrainingRouteError::CannotDeleteStartedTraining);
    }

    training_repository::mark_deleted(services.db(), id)
        .await
        .map_err(TrainingRouteError::Database)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn find_training(
    services: &Services,
    id: &str,
) -> Result<TrainingRecord, TrainingRouteError> {
    training_repository::find_by_id(services.db(), parse_ulid_uuid(id)?)
        .await
        .map_err(TrainingRouteError::Database)?
        .ok_or(TrainingRouteError::NotFound)
}

async fn trainings_to_dto(
    services: &Services,
    trainings: Vec<TrainingRecord>,
) -> Result<Vec<TrainingDto>, TrainingRouteError> {
    let mut dtos = Vec::with_capacity(trainings.len());
    for training in trainings {
        dtos.push(training_to_dto(services, training).await?);
    }
    Ok(dtos)
}

async fn training_to_dto(
    services: &Services,
    training: TrainingRecord,
) -> Result<TrainingDto, TrainingRouteError> {
    let record_sheet_filing = match training.record_sheet_filing_id {
        Some(filing_id) => Some(
            sheet_repository::list_answers_by_filing(services.db(), filing_id)
                .await
                .map_err(TrainingRouteError::Database)?
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
) -> Result<(), TrainingRouteError> {
    let current_user_id = current_user
        .user_id
        .ok_or(TrainingRouteError::Unauthorized)?;
    if training.trainer_id == current_user_id {
        return Ok(());
    }
    if training.trainee_id == current_user_id && !require_trainer {
        return Ok(());
    }
    if current_user.has_role(UserRole::ControllerTrainingMentor) {
        return Ok(());
    }

    Err(TrainingRouteError::NotOwned)
}

fn require_role(current_user: &CurrentUser, role: UserRole) -> Result<(), TrainingRouteError> {
    if current_user.has_role(role) {
        Ok(())
    } else {
        Err(TrainingRouteError::Forbidden)
    }
}

fn parse_ulid_uuid(id: &str) -> Result<Uuid, TrainingRouteError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| TrainingRouteError::InvalidId)
}

#[derive(Deserialize, utoipa::ToSchema)]
struct TrainingSaveRequest {
    name: String,
    trainer_id: String,
    trainee_id: String,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
}

impl TryFrom<TrainingSaveRequest> for TrainingSave {
    type Error = TrainingRouteError;

    fn try_from(request: TrainingSaveRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            name: request.name,
            trainer_id: parse_ulid_uuid(&request.trainer_id)?,
            trainee_id: parse_ulid_uuid(&request.trainee_id)?,
            start_at: request.start_at,
            end_at: request.end_at,
        })
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
struct TrainingRecordRequest {
    request_answers: Vec<SheetRequestField>,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct SheetRequestField {
    id: String,
    answer: String,
}

impl From<SheetRequestField> for SheetAnswerSave {
    fn from(answer: SheetRequestField) -> Self {
        Self {
            field_id: answer.id,
            answer: answer.answer,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct TrainingDto {
    id: String,
    name: String,
    trainer_id: String,
    trainer: UserDto,
    trainee_id: String,
    trainee: UserDto,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
    record_sheet_filing_id: Option<String>,
    record_sheet_filing: Option<Vec<SheetFieldAnswerDto>>,
}

impl TrainingDto {
    fn from_record(
        training: TrainingRecord,
        record_sheet_filing: Option<Vec<SheetFieldAnswerDto>>,
    ) -> Self {
        Self {
            id: Ulid::from(training.id).to_string(),
            name: training.name,
            trainer_id: Ulid::from(training.trainer_id).to_string(),
            trainer: UserDto::from_parts(
                training.trainer_id,
                training.trainer_cid,
                training.trainer_full_name,
                training.trainer_created_at,
                training.trainer_updated_at,
                training.trainer_roles,
            ),
            trainee_id: Ulid::from(training.trainee_id).to_string(),
            trainee: UserDto::from_parts(
                training.trainee_id,
                training.trainee_cid,
                training.trainee_full_name,
                training.trainee_created_at,
                training.trainee_updated_at,
                training.trainee_roles,
            ),
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

#[derive(Serialize, utoipa::ToSchema)]
struct SheetDto {
    id: String,
    name: String,
    fields: Vec<SheetFieldDto>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct SheetFieldAnswerDto {
    field: SheetFieldDto,
    answer: String,
}

impl From<SheetAnswerRecord> for SheetFieldAnswerDto {
    fn from(answer: SheetAnswerRecord) -> Self {
        Self {
            field: SheetFieldDto {
                sheet_id: answer.sheet_id,
                id: answer.field_id,
                sequence: answer.field_sequence,
                name_zh: answer.field_name_zh,
                name_en: answer.field_name_en,
                kind: answer.field_kind,
                single_choice_options: answer.field_single_choice_options,
                description_zh: answer.field_description_zh,
                description_en: answer.field_description_en,
                is_deleted: answer.field_is_deleted,
            },
            answer: answer.answer,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct SheetFieldDto {
    sheet_id: String,
    id: String,
    sequence: i64,
    name_zh: String,
    name_en: Option<String>,
    kind: String,
    single_choice_options: Vec<String>,
    description_zh: Option<String>,
    description_en: Option<String>,
    is_deleted: bool,
}

impl From<SheetFieldRecord> for SheetFieldDto {
    fn from(field: SheetFieldRecord) -> Self {
        Self {
            sheet_id: field.sheet_id,
            id: field.id,
            sequence: field.sequence,
            name_zh: field.name_zh,
            name_en: field.name_en,
            kind: field.kind,
            single_choice_options: field.single_choice_options,
            description_zh: field.description_zh,
            description_en: field.description_en,
            is_deleted: field.is_deleted,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct UserDto {
    id: String,
    cid: String,
    full_name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    roles: Vec<String>,
    direct_roles: Vec<String>,
    moodle_account: Option<serde_json::Value>,
}

impl UserDto {
    fn from_parts(
        id: Uuid,
        cid: String,
        full_name: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        roles: Vec<String>,
    ) -> Self {
        Self {
            id: Ulid::from(id).to_string(),
            cid,
            full_name,
            created_at,
            updated_at,
            roles: roles_to_dto(&roles),
            direct_roles: direct_roles_to_dto(&roles),
            moodle_account: None,
        }
    }
}

fn direct_roles_to_dto(roles: &[String]) -> Vec<String> {
    roles
        .iter()
        .filter_map(|role| role.parse::<UserRole>().ok())
        .map(|role| role.as_str().to_owned())
        .collect()
}

fn roles_to_dto(roles: &[String]) -> Vec<String> {
    let mut roles = role_closure_from_strings(roles.iter().map(String::as_str))
        .into_iter()
        .map(|role| role.as_str().to_owned())
        .collect::<Vec<_>>();
    roles.sort();
    roles
}

#[derive(Debug)]
enum TrainingRouteError {
    CannotCreateForOtherTrainer,
    CannotDeleteStartedTraining,
    CannotUpdateTrainerTrainee,
    Database(sqlx::Error),
    Forbidden,
    InvalidId,
    NotFound,
    NotOwned,
    SheetNotFound,
    Unauthorized,
}

impl IntoResponse for TrainingRouteError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            TrainingRouteError::CannotCreateForOtherTrainer => (
                StatusCode::FORBIDDEN,
                "cannot create training for other trainers".into(),
            ),
            TrainingRouteError::CannotDeleteStartedTraining => (
                StatusCode::CONFLICT,
                "cannot delete started training".into(),
            ),
            TrainingRouteError::CannotUpdateTrainerTrainee => (
                StatusCode::BAD_REQUEST,
                "cannot update training trainer or trainee".into(),
            ),
            TrainingRouteError::Database(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            TrainingRouteError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            TrainingRouteError::InvalidId => (StatusCode::BAD_REQUEST, "invalid id".into()),
            TrainingRouteError::NotFound => (StatusCode::NOT_FOUND, "training not found".into()),
            TrainingRouteError::NotOwned => (StatusCode::FORBIDDEN, "not owned".into()),
            TrainingRouteError::SheetNotFound => (StatusCode::NOT_FOUND, "sheet not found".into()),
            TrainingRouteError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct ErrorResponse {
    message: String,
}
