use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use ulid::Ulid;

use crate::error::ApiError;
use crate::model::user_role::UserRole;
use crate::modules::sheet::dto::{SheetDto, SheetFieldAnswerDto};
use crate::modules::sheet::models::SheetAnswerSave;
use crate::modules::training::dto::{TrainingDto, TrainingRecordRequest, TrainingSaveRequest};
use crate::modules::training::service::TrainingView;
use crate::modules::user::middleware::CurrentUser;
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
    Ok(Json(
        services
            .training()
            .list_active(user_id, is_training_history_admin(&current_user))
            .await?
            .into_iter()
            .map(training_to_dto)
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/atc/trainings/by-user/{userId}", tag = "Training", security(("oauth2" = [])), params(("userId" = String, Path, description = "User ULID")), responses((status = 200, description = "Successful response", body = Vec<TrainingDto>)))]
async fn list_by_user(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<TrainingDto>>, ApiError> {
    let user_id = user_id.parse::<Ulid>()?.into();
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    Ok(Json(
        services
            .training()
            .list_by_user(
                user_id,
                current_user_id,
                is_training_history_admin(&current_user),
            )
            .await?
            .into_iter()
            .map(training_to_dto)
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/atc/trainings/finished", tag = "Training", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<TrainingDto>)))]
async fn list_finished(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<TrainingDto>>, ApiError> {
    let user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    Ok(Json(
        services
            .training()
            .list_finished(user_id, is_training_history_admin(&current_user))
            .await?
            .into_iter()
            .map(training_to_dto)
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/atc/trainings/{id}", tag = "Training", security(("oauth2" = [])), params(("id" = String, Path, description = "Training ULID")), responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn get_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<TrainingDto>, ApiError> {
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let training = services
        .training()
        .find_visible(
            id.parse::<Ulid>()?.into(),
            current_user_id,
            is_training_history_admin(&current_user),
        )
        .await?;
    Ok(Json(training_to_dto(training)))
}

#[utoipa::path(post, path = "api/atc/trainings", tag = "Training", security(("oauth2" = [])), request_body = TrainingSaveRequest, responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn create_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<TrainingSaveRequest>,
) -> Result<Json<TrainingDto>, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let training = services
        .training()
        .create(
            request.try_into()?,
            current_user_id,
            current_user.has_role(UserRole::ControllerTrainingDirectorAssistant),
        )
        .await?;
    Ok(Json(training_to_dto(training)))
}

#[utoipa::path(put, path = "api/atc/trainings/{id}", tag = "Training", security(("oauth2" = [])), params(("id" = String, Path, description = "Training ULID")), request_body = TrainingSaveRequest, responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn update_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<TrainingSaveRequest>,
) -> Result<Json<TrainingDto>, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let id = id.parse::<Ulid>()?.into();
    let training = services
        .training()
        .update(
            id,
            request.try_into()?,
            current_user.user_id.ok_or(ApiError::Unauthorized)?,
            current_user.has_role(UserRole::ControllerTrainingMentor),
        )
        .await?;
    Ok(Json(training_to_dto(training)))
}

#[utoipa::path(get, path = "api/atc/trainings/record-sheet", tag = "Training", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = SheetDto)))]
async fn get_record_sheet(State(services): State<Services>) -> Result<Json<SheetDto>, ApiError> {
    let view = services
        .sheet()
        .ensure(RECORD_SHEET_ID, "Training Record Sheet")
        .await?;
    Ok(Json(SheetDto::from_entities(
        view.sheet,
        view.fields
            .into_iter()
            .filter(|field| !field.is_deleted)
            .collect(),
    )))
}

#[utoipa::path(put, path = "api/atc/trainings/{id}/record", tag = "Training", security(("oauth2" = [])), params(("id" = String, Path, description = "Training ULID")), request_body = TrainingRecordRequest, responses((status = 200, description = "Successful response", body = TrainingDto)))]
async fn set_record_sheet(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<TrainingRecordRequest>,
) -> Result<Json<TrainingDto>, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let id = id.parse::<Ulid>()?.into();
    let answers = request
        .request_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    let training = services
        .training()
        .set_record(
            id,
            &answers,
            current_user.user_id.ok_or(ApiError::Unauthorized)?,
            current_user.has_role(UserRole::ControllerTrainingMentor),
        )
        .await?;
    Ok(Json(training_to_dto(training)))
}

#[utoipa::path(delete, path = "api/atc/trainings/{id}", tag = "Training", security(("oauth2" = [])), params(("id" = String, Path, description = "Training ULID")), responses((status = 204, description = "No content")))]
async fn delete_training(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let id = id.parse::<Ulid>()?.into();
    services
        .training()
        .delete(
            id,
            current_user.user_id.ok_or(ApiError::Unauthorized)?,
            current_user.has_role(UserRole::ControllerTrainingMentor),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

fn training_to_dto(view: TrainingView) -> TrainingDto {
    TrainingDto::from_entity(
        view.training,
        view.trainer,
        view.trainee,
        view.record_sheet_filing.map(|answers| {
            answers
                .into_iter()
                .map(|view| SheetFieldAnswerDto::from_entities(view.answer, view.field))
                .collect()
        }),
    )
}

fn is_training_history_admin(current_user: &CurrentUser) -> bool {
    current_user.has_role(UserRole::ControllerTrainingDirectorAssistant)
        || current_user.has_role(UserRole::ControllerTrainingMentor)
}
