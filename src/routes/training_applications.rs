use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::adapter::email::EmailContent;
use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::user_role::UserRole;
use crate::repository::atc::user_atc_permission::UserAtcPermissionRepositoryExt;
use crate::repository::atc_training::training_application::TrainingApplicationRecord;
use crate::repository::atc_training::training_application::TrainingApplicationRepositoryExt;
use crate::repository::atc_training::training_application::TrainingApplicationTransactionExt;
use crate::repository::atc_training::training_application_response::TrainingApplicationResponseRepositoryExt;
use crate::repository::atc_training::training_application_response::TrainingApplicationResponseTransactionExt;
use crate::repository::atc_training::training_application_slot::TrainingApplicationSlotRepositoryExt;
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
    let applications = services
        .db()
        .list_training_application(current_user_id, is_admin)
        .await?;

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
    application_to_dto(&services, application, is_admin(&current_user))
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
    services
        .db()
        .mark_training_application_deleted(application.id)
        .await?;
    let application = services
        .db()
        .find_training_application_by_id(application.id)
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
    if !services
        .db()
        .has_user_atc_permission_any_by_user_id(trainee_id)
        .await?
    {
        return Err(ApiError::forbidden([UserRole::Controller]));
    }

    let slots = request
        .slots
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();
    let mut transaction = services.db().begin().await?;
    let id = transaction
        .create_training_application(trainee_id, &request.name, &slots)
        .await?;
    transaction.commit().await?;
    let application = services
        .db()
        .find_training_application_by_id(id)
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
    transaction
        .update_training_application(application.id, &request.name, &slots)
        .await?;
    transaction.commit().await?;
    let application = services
        .db()
        .find_training_application_by_id(application.id)
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
    let responses = services
        .db()
        .list_training_application_response(application.id)
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
    let application = services
        .db()
        .find_training_application_by_id(application_id)
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
            services
                .db()
                .find_training_application_slot(application.id, parse_ulid_uuid("id", slot_id)?)
                .await?
                .ok_or(ApiError::not_found("event slot", "unknown"))?,
        ),
        None => None,
    };

    let mut transaction = services.db().begin().await?;
    let response_id = transaction
        .create_training_application_response(
            &application,
            trainer_id,
            slot.as_ref(),
            &request.comment,
        )
        .await?;
    transaction.commit().await?;
    let response = services
        .db()
        .find_training_application_response(response_id)
        .await?
        .ok_or(ApiError::not_found("resource", "unknown"))?;

    if let Some(email) = application.trainee_email.as_deref() {
        services
            .email()
            .send(
                email,
                EmailContent::training_application_response(&application, &response),
            )
            .await?;
    }

    Ok(Json(TrainingApplicationResponseDto::from(response)))
}

async fn find_visible_application(
    services: &Services,
    current_user: &CurrentUser,
    id: &str,
) -> Result<TrainingApplicationRecord, ApiError> {
    let id = parse_ulid_uuid("id", id)?;
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    services
        .db()
        .find_training_application_visible_by_id(id, current_user_id, is_admin(current_user))
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
    let slots = services
        .db()
        .list_training_application_slot(application.id)
        .await?;

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
