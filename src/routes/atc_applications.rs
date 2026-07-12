use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::adapter::email::EmailContent;
use crate::audit_log_service::AuditLogService;
use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::audit_log::AuditLogEntity;
use crate::model::user_role::UserRole;
use crate::repository::atc::atc_application::AtcApplicationRecord;
use crate::repository::atc::atc_application::AtcApplicationRepositoryExt;
use crate::repository::sheet::sheet::SheetRepositoryExt;
use crate::repository::sheet::sheet_field::SheetFieldRepositoryExt;
use crate::repository::sheet::sheet_filing::SheetFilingTransactionExt;
use crate::repository::sheet::sheet_filing_answer::SheetFilingAnswerRepositoryExt;
use crate::repository::sheet::sheet_filing_answer::{SheetAnswerRecord, SheetAnswerSave};
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    list_applications,
    create_application,
    get_application_sheet,
    get_review_sheet,
    get_application,
    update_application,
    review_application
))]
pub(crate) struct ApiDoc;

const APPLICATION_SHEET_ID: &str = "atc-application";
const REVIEW_SHEET_ID: &str = "atc-application-review";

pub fn build_atc_application_routes() -> Router<Services> {
    Router::new()
        .route("/", get(list_applications).post(create_application))
        .route("/sheet", get(get_application_sheet))
        .route("/review-sheet", get(get_review_sheet))
        .route("/{id}", get(get_application).put(update_application))
        .route("/{id}/review", axum::routing::put(review_application))
}

#[utoipa::path(get, path = "api/atc/applications", tag = "ATC Application", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = Vec<AtcApplicationSummaryDto>)))]
async fn list_applications(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<AtcApplicationSummaryDto>>, ApiError> {
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let is_admin = current_user.has_role(UserRole::ControllerTrainingDirectorAssistant);
    let applications = services
        .db()
        .list_atc_application()
        .await?
        .into_iter()
        .filter(|application| is_admin || application.user_id == current_user_id)
        .map(|application| {
            AtcApplicationSummaryDto::from_record(application, is_admin, current_user_id)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(applications))
}

#[utoipa::path(post, path = "api/atc/applications", tag = "ATC Application", security(("oauth2" = [])), request_body = AtcApplicationRequest, responses((status = 200, description = "Successful response", body = AtcApplicationSummaryDto)))]
async fn create_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<AtcApplicationRequest>,
) -> Result<Json<AtcApplicationSummaryDto>, ApiError> {
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    if services
        .db()
        .count_atc_application_active_by_user(current_user_id)
        .await?
        > 0
    {
        return Err(ApiError::ApplicationAlreadyExists);
    }

    let answers = request
        .request_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    let mut transaction = services.db().begin().await?;
    let filing_id = transaction
        .set_sheet_filing(APPLICATION_SHEET_ID, None, current_user_id, &answers)
        .await?;
    let application = (&mut *transaction)
        .create_atc_application(current_user_id, filing_id)
        .await?;
    let after = application_audit_snapshot(&mut transaction, &application).await?;
    transaction.commit().await?;
    create_application_audit_log(services.audit_log(), None, &after, current_user_id).await?;

    Ok(Json(AtcApplicationSummaryDto::from_record(
        application,
        false,
        current_user_id,
    )?))
}

#[utoipa::path(get, path = "api/atc/applications/{id}", tag = "ATC Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Application ULID")), responses((status = 200, description = "Successful response", body = AtcApplicationDto)))]
async fn get_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<AtcApplicationDto>, ApiError> {
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let is_admin = current_user.has_role(UserRole::ControllerTrainingDirectorAssistant);
    let application = get_visible_application(
        &services,
        parse_ulid_uuid("id", &id)?,
        current_user_id,
        is_admin,
    )
    .await?;
    application_to_dto(&services, application, is_admin, current_user_id)
        .await
        .map(Json)
}

#[utoipa::path(put, path = "api/atc/applications/{id}", tag = "ATC Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Application ULID")), request_body = AtcApplicationRequest, responses((status = 200, description = "Successful response", body = AtcApplicationDto)))]
async fn update_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<AtcApplicationRequest>,
) -> Result<Json<AtcApplicationDto>, ApiError> {
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let is_admin = current_user.has_role(UserRole::ControllerTrainingDirectorAssistant);
    let application_id = parse_ulid_uuid("id", &id)?;
    let mut transaction = services.db().begin().await?;
    let application = (&mut *transaction)
        .find_atc_application_by_id_for_update(application_id)
        .await?
        .ok_or(ApiError::not_found("application", &id))?;
    ensure_application_visible(&application, current_user_id, is_admin)?;
    if AtcApplicationStatus::from_db_str(&application.status)? != AtcApplicationStatus::Submitted {
        return Err(ApiError::ApplicationCannotUpdate);
    }
    let before = application_audit_snapshot(&mut transaction, &application).await?;

    let answers = request
        .request_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    transaction
        .set_sheet_filing(
            APPLICATION_SHEET_ID,
            Some(application.application_filing_id),
            current_user_id,
            &answers,
        )
        .await?;
    let application = (&mut *transaction)
        .find_atc_application_by_id_for_update(application.id)
        .await?
        .ok_or(ApiError::not_found("application", &id))?;
    let after = application_audit_snapshot(&mut transaction, &application).await?;
    transaction.commit().await?;
    create_application_audit_log(services.audit_log(), Some(&before), &after, current_user_id)
        .await?;

    application_to_dto(&services, application, false, current_user_id)
        .await
        .map(Json)
}

#[utoipa::path(get, path = "api/atc/applications/sheet", tag = "ATC Application", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = SheetDto)))]
async fn get_application_sheet(
    State(services): State<Services>,
) -> Result<Json<SheetDto>, ApiError> {
    sheet_dto(&services, APPLICATION_SHEET_ID, "ATC Application Sheet")
        .await
        .map(Json)
}

#[utoipa::path(get, path = "api/atc/applications/review-sheet", tag = "ATC Application", security(("oauth2" = [])), responses((status = 200, description = "Successful response", body = SheetDto)))]
async fn get_review_sheet(State(services): State<Services>) -> Result<Json<SheetDto>, ApiError> {
    sheet_dto(&services, REVIEW_SHEET_ID, "ATC Application Review Sheet")
        .await
        .map(Json)
}

#[utoipa::path(put, path = "api/atc/applications/{id}/review", tag = "ATC Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Application ULID")), request_body = AtcApplicationReviewRequest, responses((status = 200, description = "Successful response", body = AtcApplicationDto)))]
async fn review_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<AtcApplicationReviewRequest>,
) -> Result<Json<AtcApplicationDto>, ApiError> {
    current_user.require_role(UserRole::ControllerTrainingDirectorAssistant)?;
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let application_id = parse_ulid_uuid("id", &id)?;
    let approved = request.status == AtcApplicationStatus::Approved;
    let mut transaction = services.db().begin().await?;
    let application = (&mut *transaction)
        .find_atc_application_by_id_for_update(application_id)
        .await?
        .ok_or(ApiError::not_found("application", &id))?;
    let before = application_audit_snapshot(&mut transaction, &application).await?;
    let answers = request
        .review_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    let filing_id = transaction
        .set_sheet_filing(
            REVIEW_SHEET_ID,
            application.review_filing_id,
            current_user_id,
            &answers,
        )
        .await?;
    let application = (&mut *transaction)
        .set_atc_application_review(application_id, request.status.as_db_str(), filing_id)
        .await?
        .ok_or(ApiError::not_found("application", &id))?;
    let after = application_audit_snapshot(&mut transaction, &application).await?;
    transaction.commit().await?;
    create_application_audit_log(services.audit_log(), Some(&before), &after, current_user_id)
        .await?;

    if let Some(email) = application.user_email.as_deref() {
        services
            .email()
            .send(
                email,
                EmailContent::atc_application_status_change(&application),
            )
            .await?;
    }

    if approved {
        ensure_moodle_user(&services, &application).await?;
    }

    application_to_dto(&services, application, true, current_user_id)
        .await
        .map(Json)
}

async fn get_visible_application(
    services: &Services,
    id: Uuid,
    current_user_id: Uuid,
    is_admin: bool,
) -> Result<AtcApplicationRecord, ApiError> {
    let application = services
        .db()
        .find_atc_application_by_id(id)
        .await?
        .ok_or(ApiError::not_found("application", id.to_string()))?;
    ensure_application_visible(&application, current_user_id, is_admin)?;

    Ok(application)
}

fn ensure_application_visible(
    application: &AtcApplicationRecord,
    current_user_id: Uuid,
    is_admin: bool,
) -> Result<(), ApiError> {
    if !is_admin && application.user_id != current_user_id {
        return Err(ApiError::not_found(
            "application",
            application.id.to_string(),
        ));
    }

    Ok(())
}

#[derive(Serialize)]
struct AtcApplicationAuditSnapshot {
    application: AtcApplicationRecord,
    application_filing_answers: Vec<SheetAnswerRecord>,
    review_filing_answers: Option<Vec<SheetAnswerRecord>>,
}

async fn application_audit_snapshot(
    transaction: &mut Transaction<'_, Postgres>,
    application: &AtcApplicationRecord,
) -> Result<AtcApplicationAuditSnapshot, ApiError> {
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

async fn create_application_audit_log(
    audit_log: &AuditLogService,
    before: Option<&AtcApplicationAuditSnapshot>,
    after: &AtcApplicationAuditSnapshot,
    operated_by: Uuid,
) -> Result<(), ApiError> {
    audit_log
        .record(
            AuditLogEntity::AtcApplication(after.application.id),
            operated_by,
            before,
            Some(after),
        )
        .await?;

    Ok(())
}

async fn application_to_dto(
    services: &Services,
    application: AtcApplicationRecord,
    is_admin: bool,
    current_user_id: Uuid,
) -> Result<AtcApplicationDto, ApiError> {
    let application_filing_answers = services
        .db()
        .list_sheet_filing_answer_by_filing(application.application_filing_id)
        .await?
        .into_iter()
        .map(SheetFieldAnswerDto::from)
        .collect();
    let review_filing_answers = match application.review_filing_id {
        Some(review_filing_id) => Some(
            services
                .db()
                .list_sheet_filing_answer_by_filing(review_filing_id)
                .await?
                .into_iter()
                .map(SheetFieldAnswerDto::from)
                .collect(),
        ),
        None => None,
    };
    let moodle_account = moodle_account(services, &application.user_cid).await?;

    AtcApplicationDto::from_record(
        application,
        is_admin,
        current_user_id,
        application_filing_answers,
        review_filing_answers,
        moodle_account,
    )
}

async fn ensure_moodle_user(
    services: &Services,
    application: &AtcApplicationRecord,
) -> Result<(), ApiError> {
    let moodle_user = services
        .moodle()
        .get_user_by_cid(&application.user_cid)
        .await?;
    if let Some(moodle_user) = moodle_user {
        tracing::info!(
            moodle_user_id = moodle_user.id,
            cid = %application.user_cid,
            "Moodle user found for CID, skipping user creation"
        );
        return Ok(());
    }

    tracing::info!(
        cid = %application.user_cid,
        "No Moodle user found for CID, creating new user"
    );
    let created_users = services
        .moodle()
        .create_user(
            &application.user_cid,
            &application.user_full_name,
            application.user_email.as_deref(),
        )
        .await?;
    for created_user in created_users {
        tracing::info!(
            moodle_user_id = created_user.id,
            moodle_username = %created_user.username,
            cid = %application.user_cid,
            "Created Moodle user"
        );
    }

    Ok(())
}

async fn moodle_account(
    services: &Services,
    cid: &str,
) -> Result<Option<UserMoodleInfoDto>, ApiError> {
    Ok(services
        .moodle()
        .get_user_by_cid(cid)
        .await?
        .map(|user| UserMoodleInfoDto {
            id: user.id.to_string(),
        }))
}

async fn sheet_dto(
    services: &Services,
    sheet_id: &str,
    sheet_name: &str,
) -> Result<SheetDto, ApiError> {
    services.db().ensure_sheet(sheet_id, sheet_name).await?;
    let sheet = services
        .db()
        .find_sheet(sheet_id)
        .await?
        .ok_or(ApiError::not_found("sheet", sheet_id))?;
    let fields = services.db().list_sheet_field(sheet_id).await?;

    Ok(SheetDto {
        id: sheet.id,
        name: sheet.name,
        fields: fields
            .into_iter()
            .filter(|field| !field.is_deleted)
            .map(SheetFieldDto::from)
            .collect(),
    })
}
