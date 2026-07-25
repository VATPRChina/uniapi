use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::adapter::email::EmailContent;
use crate::dto::*;
use crate::model::user_role::UserRole;
use crate::modules::atc_application::dto::{
    AtcApplicationDto, AtcApplicationRequest, AtcApplicationReviewRequest, AtcApplicationStatus,
    AtcApplicationSummaryDto,
};
use crate::modules::atc_application::service::AtcApplicationView;
use crate::modules::authentication::middleware::CurrentUser;
use crate::modules::sheet::dto::{SheetDto, SheetFieldAnswerDto};
use crate::modules::sheet::models::SheetAnswerSave;
use crate::modules::user::dto::UserMoodleInfoDto;
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
        .atc_application()
        .list(current_user_id, is_admin)
        .await?
        .into_iter()
        .map(|view| {
            AtcApplicationSummaryDto::from_entity(
                view.application,
                view.user,
                is_admin,
                current_user_id,
            )
        })
        .collect();

    Ok(Json(applications))
}

#[utoipa::path(post, path = "api/atc/applications", tag = "ATC Application", security(("oauth2" = [])), request_body = AtcApplicationRequest, responses((status = 200, description = "Successful response", body = AtcApplicationSummaryDto)))]
async fn create_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<AtcApplicationRequest>,
) -> Result<Json<AtcApplicationSummaryDto>, ApiError> {
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let answers = request
        .request_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    let view = services
        .atc_application()
        .create(current_user_id, &answers)
        .await?;

    Ok(Json(AtcApplicationSummaryDto::from_entity(
        view.application,
        view.user,
        false,
        current_user_id,
    )))
}

#[utoipa::path(get, path = "api/atc/applications/{id}", tag = "ATC Application", security(("oauth2" = [])), params(("id" = String, Path, description = "Application ULID")), responses((status = 200, description = "Successful response", body = AtcApplicationDto)))]
async fn get_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<AtcApplicationDto>, ApiError> {
    let current_user_id = current_user.user_id.ok_or(ApiError::Unauthorized)?;
    let is_admin = current_user.has_role(UserRole::ControllerTrainingDirectorAssistant);
    let view = services
        .atc_application()
        .find_visible(parse_ulid_uuid("id", &id)?, current_user_id, is_admin)
        .await?;
    application_to_dto(&services, view, is_admin, current_user_id)
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
    let answers = request
        .request_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    let view = services
        .atc_application()
        .update(application_id, current_user_id, is_admin, &answers)
        .await?;

    application_to_dto(&services, view, false, current_user_id)
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
    let answers = request
        .review_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    let view = services
        .atc_application()
        .review(
            application_id,
            current_user_id,
            request.status.into(),
            &answers,
        )
        .await?;

    if let Some(email) = view.user.email.as_deref() {
        services
            .email()
            .send(
                email,
                EmailContent::atc_application_status_change(&view.application),
            )
            .await?;
    }

    if approved {
        services
            .atc_application()
            .ensure_moodle_account(view.user.id)
            .await?;
    }

    application_to_dto(&services, view, true, current_user_id)
        .await
        .map(Json)
}

async fn application_to_dto(
    services: &Services,
    view: AtcApplicationView,
    is_admin: bool,
    current_user_id: Uuid,
) -> Result<AtcApplicationDto, ApiError> {
    let AtcApplicationView { application, user } = view;
    let (application_filing_answers, review_filing_answers) = services
        .atc_application()
        .filing_answers(&application)
        .await?;
    let application_filing_answers = application_filing_answers
        .into_iter()
        .map(|view| SheetFieldAnswerDto::from_entities(view.answer, view.field))
        .collect();
    let review_filing_answers = review_filing_answers.map(|answers| {
        answers
            .into_iter()
            .map(|view| SheetFieldAnswerDto::from_entities(view.answer, view.field))
            .collect()
    });
    let moodle_account = services
        .atc_application()
        .moodle_account(user.id)
        .await?
        .map(|id| UserMoodleInfoDto { id: id.to_string() });

    Ok(AtcApplicationDto::from_entity(
        application,
        user,
        is_admin,
        current_user_id,
        application_filing_answers,
        review_filing_answers,
        moodle_account,
    ))
}

async fn sheet_dto(
    services: &Services,
    sheet_id: &str,
    sheet_name: &str,
) -> Result<SheetDto, ApiError> {
    let view = services.sheet().ensure(sheet_id, sheet_name).await?;
    Ok(SheetDto::from_entities(
        view.sheet,
        view.fields
            .into_iter()
            .filter(|field| !field.is_deleted)
            .collect(),
    ))
}
