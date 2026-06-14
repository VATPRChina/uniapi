use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::user_role::UserRole;
use crate::repository::atc::user_atc_permission as atc_permission_repository;
use crate::repository::atc_training::training_application::{
    self as training_application_repository, TrainingApplicationRecord,
};
use crate::repository::atc_training::training_application_response::{
    self as training_application_response_repository,
};
use crate::repository::atc_training::training_application_slot::{
    self as training_application_slot_repository,
};
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
    let applications =
        training_application_repository::list(services.db(), current_user_id, is_admin).await?;

    applications_to_dto(&services, applications, is_admin)
        .await
        .map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/applications/{id}", operation_id = "get_training_application", tag = "Training Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), responses((status = 200, description = "Successful response", body = TrainingApplicationDto)))]
async fn get_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<TrainingApplicationDto>, ApiError> {
    let application = find_visible_application(&services, &current_user, &id).await?;
    application_to_dto(&services, application, false)
        .await
        .map(Json)
}

#[utoipa::path(delete, path = "api/atc/trainings/applications/{id}", tag = "Training Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), responses((status = 200, description = "Successful response", body = TrainingApplicationDto)))]
async fn delete_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<TrainingApplicationDto>, ApiError> {
    let application = find_visible_application(&services, &current_user, &id).await?;
    training_application_repository::mark_deleted(services.db(), application.id).await?;
    let application = training_application_repository::find_by_id(services.db(), application.id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;

    application_to_dto(&services, application, false)
        .await
        .map(Json)
}

#[utoipa::path(post, path = "api/atc/trainings/applications", operation_id = "create_training_application", tag = "Training Application", security(("oauth2" = [])), request_body = TrainingApplicationCreateRequest, responses((status = 200, description = "Successful response", body = TrainingApplicationDto)))]
async fn create_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<TrainingApplicationCreateRequest>,
) -> Result<Json<TrainingApplicationDto>, ApiError> {
    let trainee_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    if !atc_permission_repository::has_any_by_user_id(services.db(), trainee_id).await? {
        return Err(ApiError::forbidden([UserRole::Controller]));
    }

    let slots = request
        .slots
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();
    let mut transaction = services.db().begin().await?;
    let id = training_application_repository::create(
        &mut transaction,
        trainee_id,
        &request.name,
        &slots,
    )
    .await?;
    transaction.commit().await?;
    let application = training_application_repository::find_by_id(services.db(), id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;

    application_to_dto(&services, application, false)
        .await
        .map(Json)
}

#[utoipa::path(put, path = "api/atc/trainings/applications/{id}", operation_id = "update_training_application", tag = "Training Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), request_body = TrainingApplicationCreateRequest, responses((status = 200, description = "Successful response", body = TrainingApplicationDto)))]
async fn update_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<TrainingApplicationCreateRequest>,
) -> Result<Json<TrainingApplicationDto>, ApiError> {
    let application = find_visible_application(&services, &current_user, &id).await?;
    let slots = request
        .slots
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();
    let mut transaction = services.db().begin().await?;
    training_application_repository::update(
        &mut transaction,
        application.id,
        &request.name,
        &slots,
    )
    .await?;
    transaction.commit().await?;
    let application = training_application_repository::find_by_id(services.db(), application.id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;

    application_to_dto(&services, application, false)
        .await
        .map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/applications/{id}/responses", tag = "Training Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), responses((status = 200, description = "Successful response", body = Vec<TrainingApplicationResponseDto>)))]
async fn list_responses(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<TrainingApplicationResponseDto>>, ApiError> {
    let application = find_visible_application(&services, &current_user, &id).await?;
    let responses = training_application_response_repository::list(services.db(), application.id)
        .await?
        .into_iter()
        .map(TrainingApplicationResponseDto::from)
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
    let application_id = parse_ulid_uuid("id", &id)?;
    let trainer_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let application = training_application_repository::find_by_id(services.db(), application_id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;
    if application.train_id.is_some() {
        return Err(ApiError::TrainingApplicationAlreadyAccepted);
    }
    if application.deleted_at.is_some() {
        return Err(ApiError::not_found("resource", "unknown"));
    }

    let slot = match request.slot_id.as_deref() {
        Some(slot_id) => Some(
            training_application_slot_repository::find(
                services.db(),
                application.id,
                parse_ulid_uuid("id", slot_id)?,
            )
            .await?
            .ok_or(ApiError::not_found("event slot", "unknown"))?,
        ),
        None => None,
    };

    let mut transaction = services.db().begin().await?;
    let response_id = training_application_response_repository::create(
        &mut transaction,
        &application,
        trainer_id,
        slot.as_ref(),
        &request.comment,
    )
    .await?;
    transaction.commit().await?;
    let response = training_application_response_repository::find(services.db(), response_id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;

    Ok(Json(TrainingApplicationResponseDto::from(response)))
}

async fn find_visible_application(
    services: &Services,
    current_user: &CurrentUser,
    id: &str,
) -> Result<TrainingApplicationRecord, ApiError> {
    let id = parse_ulid_uuid("id", id)?;
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    training_application_repository::find_visible_by_id(
        services.db(),
        id,
        current_user_id,
        is_admin(current_user),
    )
    .await?
    .ok_or(ApiError::not_found("resource", "unknown"))
}

async fn applications_to_dto(
    services: &Services,
    applications: Vec<TrainingApplicationRecord>,
    include_trainee_email: bool,
) -> Result<Vec<TrainingApplicationDto>, ApiError> {
    let mut dtos = Vec::with_capacity(applications.len());
    for application in applications {
        dtos.push(application_to_dto(services, application, include_trainee_email).await?);
    }
    Ok(dtos)
}

async fn application_to_dto(
    services: &Services,
    application: TrainingApplicationRecord,
    include_trainee_email: bool,
) -> Result<TrainingApplicationDto, ApiError> {
    let slots = training_application_slot_repository::list(services.db(), application.id).await?;

    Ok(TrainingApplicationDto::from_record(
        application,
        slots,
        include_trainee_email,
    ))
}

fn is_admin(current_user: &CurrentUser) -> bool {
    current_user
        .require_any_role(&[
            UserRole::ControllerTrainingDirectorAssistant,
            UserRole::ControllerTrainingMentor,
        ])
        .is_ok()
}
