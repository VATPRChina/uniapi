use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::models::user_role::{UserRole, role_closure_from_strings};
use crate::repository::atc::user_atc_permission as atc_permission_repository;
use crate::repository::atc_training::training_application::{
    self as training_application_repository, TrainingApplicationRecord,
};
use crate::repository::atc_training::training_application_response::{
    self as training_application_response_repository, TrainingApplicationResponseRecord,
};
use crate::repository::atc_training::training_application_slot::{
    self as training_application_slot_repository, TrainingApplicationSlotRecord,
    TrainingApplicationSlotSave,
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

#[utoipa::path(get, path = "api/atc/trainings/applications", operation_id = "list_training_applications", tag = "ATC", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<TrainingApplicationDto>)))]
async fn list_applications(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<TrainingApplicationDto>>, ApiError> {
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let is_admin = is_admin(&current_user);
    let applications =
        training_application_repository::list(services.db(), current_user_id, is_admin).await?;

    applications_to_dto(&services, applications).await.map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/applications/{id}", operation_id = "get_training_application", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), responses((status = 200, description = "Successful response", body = TrainingApplicationDto)))]
async fn get_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<TrainingApplicationDto>, ApiError> {
    let application = find_visible_application(&services, &current_user, &id).await?;
    application_to_dto(&services, application).await.map(Json)
}

#[utoipa::path(delete, path = "api/atc/trainings/applications/{id}", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), responses((status = 200, description = "Successful response", body = TrainingApplicationDto)))]
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

    application_to_dto(&services, application).await.map(Json)
}

#[utoipa::path(post, path = "api/atc/trainings/applications", operation_id = "create_training_application", tag = "ATC", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = TrainingApplicationDto)))]
async fn create_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<TrainingApplicationCreateRequest>,
) -> Result<Json<TrainingApplicationDto>, ApiError> {
    let trainee_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    if !atc_permission_repository::has_any_by_user_id(services.db(), trainee_id).await? {
        return Err(ApiError::forbidden(["controller"]));
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

    application_to_dto(&services, application).await.map(Json)
}

#[utoipa::path(put, path = "api/atc/trainings/applications/{id}", operation_id = "update_training_application", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), responses((status = 200, description = "Successful response", body = TrainingApplicationDto)))]
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

    application_to_dto(&services, application).await.map(Json)
}

#[utoipa::path(get, path = "api/atc/trainings/applications/{id}/responses", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), responses((status = 200, description = "Successful response", body = Vec<TrainingApplicationResponseDto>)))]
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

#[utoipa::path(put, path = "api/atc/trainings/applications/{id}/response", tag = "ATC", security(("oauth2" = [])), params(("id" = String, Path, description = "Training application ULID")), responses((status = 200, description = "Successful response", body = TrainingApplicationResponseDto)))]
async fn respond_to_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<TrainingApplicationResponseRequest>,
) -> Result<Json<TrainingApplicationResponseDto>, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingMentor)?;
    let application_id = parse_ulid_uuid(&id)?;
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
                parse_ulid_uuid(slot_id)?,
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
    let id = parse_ulid_uuid(id)?;
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
) -> Result<Vec<TrainingApplicationDto>, ApiError> {
    let mut dtos = Vec::with_capacity(applications.len());
    for application in applications {
        dtos.push(application_to_dto(services, application).await?);
    }
    Ok(dtos)
}

async fn application_to_dto(
    services: &Services,
    application: TrainingApplicationRecord,
) -> Result<TrainingApplicationDto, ApiError> {
    let slots = training_application_slot_repository::list(services.db(), application.id).await?;

    Ok(TrainingApplicationDto::from_record(application, slots))
}

fn is_admin(current_user: &CurrentUser) -> bool {
    current_user
        .require_any_role(&[
            UserRole::ControllerTrainingDirectorAssistant,
            UserRole::ControllerTrainingMentor,
        ])
        .is_ok()
}

fn parse_ulid_uuid(id: &str) -> Result<Uuid, ApiError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| ApiError::bad_request("id", "invalid ULID"))
}

#[derive(Deserialize, utoipa::ToSchema)]
struct TrainingApplicationCreateRequest {
    name: String,
    slots: Vec<TrainingApplicationCreateRequestSlot>,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct TrainingApplicationCreateRequestSlot {
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
}

impl From<TrainingApplicationCreateRequestSlot> for TrainingApplicationSlotSave {
    fn from(slot: TrainingApplicationCreateRequestSlot) -> Self {
        Self {
            start_at: slot.start_at,
            end_at: slot.end_at,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
struct TrainingApplicationResponseRequest {
    slot_id: Option<String>,
    comment: String,
}

#[derive(Serialize, utoipa::ToSchema)]
struct TrainingApplicationDto {
    id: String,
    trainee_id: String,
    trainee: UserDto,
    status: TrainingApplicationStatus,
    name: String,
    train_id: Option<String>,
    slots: Vec<TrainingApplicationSlotDto>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TrainingApplicationDto {
    fn from_record(
        application: TrainingApplicationRecord,
        slots: Vec<TrainingApplicationSlotRecord>,
    ) -> Self {
        let status = application_status(&application, &slots);
        Self {
            id: Ulid::from(application.id).to_string(),
            trainee_id: Ulid::from(application.trainee_id).to_string(),
            trainee: UserDto {
                id: Ulid::from(application.trainee_id).to_string(),
                cid: application.trainee_cid,
                full_name: application.trainee_full_name,
                created_at: application.trainee_created_at,
                updated_at: application.trainee_updated_at,
                roles: roles_to_dto(&application.trainee_roles),
                direct_roles: direct_roles_to_dto(&application.trainee_roles),
                moodle_account: None,
            },
            status,
            name: application.name,
            train_id: application.train_id.map(|id| Ulid::from(id).to_string()),
            slots: slots
                .into_iter()
                .map(TrainingApplicationSlotDto::from)
                .collect(),
            created_at: application.created_at,
            updated_at: application.updated_at,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
enum TrainingApplicationStatus {
    Pending,
    Accepted,
    Rejected,
    Cancelled,
}

fn application_status(
    application: &TrainingApplicationRecord,
    slots: &[TrainingApplicationSlotRecord],
) -> TrainingApplicationStatus {
    if application.train_id.is_some() {
        TrainingApplicationStatus::Accepted
    } else if application.deleted_at.is_some() {
        TrainingApplicationStatus::Cancelled
    } else if slots
        .iter()
        .map(|slot| slot.end_at)
        .max()
        .is_some_and(|end_at| end_at < Utc::now())
    {
        TrainingApplicationStatus::Rejected
    } else {
        TrainingApplicationStatus::Pending
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct TrainingApplicationSlotDto {
    id: String,
    application_id: String,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
}

impl From<TrainingApplicationSlotRecord> for TrainingApplicationSlotDto {
    fn from(slot: TrainingApplicationSlotRecord) -> Self {
        Self {
            id: Ulid::from(slot.id).to_string(),
            application_id: Ulid::from(slot.application_id).to_string(),
            start_at: slot.start_at,
            end_at: slot.end_at,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct TrainingApplicationResponseDto {
    id: String,
    application_id: String,
    trainer_id: String,
    trainer: UserDto,
    is_accepted: bool,
    comment: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TrainingApplicationResponseRecord> for TrainingApplicationResponseDto {
    fn from(response: TrainingApplicationResponseRecord) -> Self {
        Self {
            id: Ulid::from(response.id).to_string(),
            application_id: Ulid::from(response.application_id).to_string(),
            trainer_id: Ulid::from(response.trainer_id).to_string(),
            trainer: UserDto {
                id: Ulid::from(response.trainer_id).to_string(),
                cid: response.trainer_cid,
                full_name: response.trainer_full_name,
                created_at: response.trainer_created_at,
                updated_at: response.trainer_updated_at,
                roles: roles_to_dto(&response.trainer_roles),
                direct_roles: direct_roles_to_dto(&response.trainer_roles),
                moodle_account: None,
            },
            is_accepted: response.slot_id.is_some(),
            comment: response.comment,
            created_at: response.created_at,
            updated_at: response.updated_at,
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
