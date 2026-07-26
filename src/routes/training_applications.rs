use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::dto::parse_ulid_uuid;
use crate::model::user_role::UserRole;
use crate::modules::training::dto::{
    TrainingApplicationCreateRequest, TrainingApplicationDto, TrainingApplicationResponseDto,
    TrainingApplicationResponseRequest,
};
use crate::modules::training::service::TrainingApplicationView;
use crate::modules::user::middleware::CurrentUser;
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    list_applications,
    create_application,
    get_application,
    update_application,
    delete_application,
    list_responses,
    respond_to_application
))]
pub(crate) struct ApiDoc;

pub fn build_training_application_routes() -> Router<Services> {
    Router::new()
        .route("/", get(list_applications).post(create_application))
        .route(
            "/{id}",
            get(get_application)
                .put(update_application)
                .delete(delete_application),
        )
        .route("/{id}/responses", get(list_responses))
        .route("/{id}/response", axum::routing::put(respond_to_application))
}

#[utoipa::path(get, path = "api/atc/trainings/applications", operation_id = "list_training_applications", tag = "Training Application", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<TrainingApplicationDto>)))]
async fn list_applications(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<TrainingApplicationDto>>, ApiError> {
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let is_admin = is_admin(&current_user);
    Ok(Json(
        services
            .training_application()
            .list(current_user_id, is_admin)
            .await?
            .into_iter()
            .map(|view| application_to_dto(view, is_admin))
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/atc/trainings/applications/{id}", operation_id = "get_training_application", tag = "Training Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), responses((status = 200, description = "Successful response", body = TrainingApplicationDto)))]
async fn get_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<TrainingApplicationDto>, ApiError> {
    let is_admin = is_admin(&current_user);
    let application = services
        .training_application()
        .find_visible(
            parse_ulid_uuid("id", &id)?,
            current_user.user_id.ok_or(ApiError::Unauthorized)?,
            is_admin,
        )
        .await?;
    Ok(Json(application_to_dto(application, is_admin)))
}

#[utoipa::path(delete, path = "api/atc/trainings/applications/{id}", tag = "Training Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), responses((status = 200, description = "Successful response", body = TrainingApplicationDto)))]
async fn delete_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<TrainingApplicationDto>, ApiError> {
    let application = services
        .training_application()
        .delete(
            parse_ulid_uuid("id", &id)?,
            current_user.user_id.ok_or(ApiError::Unauthorized)?,
            is_admin(&current_user),
        )
        .await?;
    Ok(Json(application_to_dto(application, false)))
}

#[utoipa::path(post, path = "api/atc/trainings/applications", operation_id = "create_training_application", tag = "Training Application", security(("oauth2" = [])), request_body = TrainingApplicationCreateRequest, responses((status = 200, description = "Successful response", body = TrainingApplicationDto)))]
async fn create_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<TrainingApplicationCreateRequest>,
) -> Result<Json<TrainingApplicationDto>, ApiError> {
    let trainee_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let slots = request
        .slots
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();
    let application = services
        .training_application()
        .create(trainee_id, &request.name, &slots)
        .await?;
    Ok(Json(application_to_dto(application, false)))
}

#[utoipa::path(put, path = "api/atc/trainings/applications/{id}", operation_id = "update_training_application", tag = "Training Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), request_body = TrainingApplicationCreateRequest, responses((status = 200, description = "Successful response", body = TrainingApplicationDto)))]
async fn update_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<TrainingApplicationCreateRequest>,
) -> Result<Json<TrainingApplicationDto>, ApiError> {
    let slots = request
        .slots
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();
    let application = services
        .training_application()
        .update(
            parse_ulid_uuid("id", &id)?,
            current_user.user_id.ok_or(ApiError::Unauthorized)?,
            is_admin(&current_user),
            &request.name,
            &slots,
        )
        .await?;
    Ok(Json(application_to_dto(application, false)))
}

#[utoipa::path(get, path = "api/atc/trainings/applications/{id}/responses", tag = "Training Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), responses((status = 200, description = "Successful response", body = Vec<TrainingApplicationResponseDto>)))]
async fn list_responses(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<TrainingApplicationResponseDto>>, ApiError> {
    let responses = services
        .training_application()
        .list_responses(
            parse_ulid_uuid("id", &id)?,
            current_user.user_id.ok_or(ApiError::Unauthorized)?,
            is_admin(&current_user),
        )
        .await?
        .into_iter()
        .map(|view| TrainingApplicationResponseDto::from_entity(view.response, view.trainer))
        .collect();
    Ok(Json(responses))
}

#[utoipa::path(put, path = "api/atc/trainings/applications/{id}/response", tag = "Training Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), request_body = TrainingApplicationResponseRequest, responses((status = 200, description = "Successful response", body = TrainingApplicationResponseDto)))]
async fn respond_to_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<TrainingApplicationResponseRequest>,
) -> Result<Json<TrainingApplicationResponseDto>, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let response = services
        .training_application()
        .respond(
            parse_ulid_uuid("id", &id)?,
            current_user.user_id.ok_or(ApiError::Unauthorized)?,
            request
                .slot_id
                .as_deref()
                .map(|id| parse_ulid_uuid("id", id))
                .transpose()?,
            &request.comment,
        )
        .await?;
    Ok(Json(TrainingApplicationResponseDto::from_entity(
        response.response,
        response.trainer,
    )))
}

fn application_to_dto(
    view: TrainingApplicationView,
    include_trainee_email: bool,
) -> TrainingApplicationDto {
    TrainingApplicationDto::from_entity(
        view.application,
        view.trainee,
        view.slots,
        include_trainee_email,
    )
}

fn is_admin(current_user: &CurrentUser) -> bool {
    current_user
        .require_any_role(&[
            UserRole::ControllerTrainingDirectorAssistant,
            UserRole::ControllerTrainingMentor,
        ])
        .is_ok()
}
