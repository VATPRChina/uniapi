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
        atc_application::{self as application_repository, AtcApplicationRecord},
        sheet::{self as sheet_repository, SheetAnswerRecord, SheetAnswerSave, SheetFieldRecord},
    },
    auth::CurrentUser,
    models::user_role::{UserRole, role_closure_from_strings},
    services::Services,
};

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

#[utoipa::path(get, path = "api/atc/applications", tag = "ATC", security(("bearerAuth" = [])), responses((status = 200, description = "Successful response", body = Vec<AtcApplicationSummaryDto>)))]
async fn list_applications(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<AtcApplicationSummaryDto>>, AtcApplicationRouteError> {
    let current_user_id = current_user
        .user_id
        .ok_or(AtcApplicationRouteError::Unauthorized)?;
    let is_admin = current_user.has_role(UserRole::ControllerTrainingDirectorAssistant);
    let applications = application_repository::list(services.db())
        .await
        .map_err(AtcApplicationRouteError::Database)?
        .into_iter()
        .filter(|application| is_admin || application.user_id == current_user_id)
        .map(|application| {
            AtcApplicationSummaryDto::from_record(application, is_admin, current_user_id)
        })
        .collect();

    Ok(Json(applications))
}

#[utoipa::path(post, path = "api/atc/applications", tag = "ATC", security(("bearerAuth" = [])), responses((status = 200, description = "Successful response", body = AtcApplicationSummaryDto)))]
async fn create_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Json(request): Json<AtcApplicationRequest>,
) -> Result<Json<AtcApplicationSummaryDto>, AtcApplicationRouteError> {
    let current_user_id = current_user
        .user_id
        .ok_or(AtcApplicationRouteError::Unauthorized)?;
    if application_repository::count_active_by_user(services.db(), current_user_id)
        .await
        .map_err(AtcApplicationRouteError::Database)?
        > 0
    {
        return Err(AtcApplicationRouteError::ApplicationAlreadyExists);
    }

    let answers = request
        .request_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    let mut transaction = services
        .db()
        .begin()
        .await
        .map_err(AtcApplicationRouteError::Database)?;
    let filing_id = sheet_repository::set_filing(
        &mut transaction,
        APPLICATION_SHEET_ID,
        None,
        current_user_id,
        &answers,
    )
    .await
    .map_err(AtcApplicationRouteError::Database)?;
    transaction
        .commit()
        .await
        .map_err(AtcApplicationRouteError::Database)?;

    let application = application_repository::create(services.db(), current_user_id, filing_id)
        .await
        .map_err(AtcApplicationRouteError::Database)?;

    Ok(Json(AtcApplicationSummaryDto::from_record(
        application,
        false,
        current_user_id,
    )))
}

#[utoipa::path(get, path = "api/atc/applications/{id}", tag = "ATC", security(("bearerAuth" = [])), params(("id" = String, Path, description = "Application ULID")), responses((status = 200, description = "Successful response", body = AtcApplicationDto)))]
async fn get_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<AtcApplicationDto>, AtcApplicationRouteError> {
    let current_user_id = current_user
        .user_id
        .ok_or(AtcApplicationRouteError::Unauthorized)?;
    let is_admin = current_user.has_role(UserRole::ControllerTrainingDirectorAssistant);
    let application =
        get_visible_application(&services, parse_ulid_uuid(&id)?, current_user_id, is_admin)
            .await?;
    application_to_dto(&services, application, is_admin, current_user_id)
        .await
        .map(Json)
}

#[utoipa::path(put, path = "api/atc/applications/{id}", tag = "ATC", security(("bearerAuth" = [])), params(("id" = String, Path, description = "Application ULID")), responses((status = 200, description = "Successful response", body = AtcApplicationDto)))]
async fn update_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<AtcApplicationRequest>,
) -> Result<Json<AtcApplicationDto>, AtcApplicationRouteError> {
    let current_user_id = current_user
        .user_id
        .ok_or(AtcApplicationRouteError::Unauthorized)?;
    let is_admin = current_user.has_role(UserRole::ControllerTrainingDirectorAssistant);
    let application =
        get_visible_application(&services, parse_ulid_uuid(&id)?, current_user_id, is_admin)
            .await?;
    if parse_status(&application.status) != AtcApplicationStatus::Submitted {
        return Err(AtcApplicationRouteError::ApplicationCannotUpdate);
    }

    let answers = request
        .request_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    let mut transaction = services
        .db()
        .begin()
        .await
        .map_err(AtcApplicationRouteError::Database)?;
    sheet_repository::set_filing(
        &mut transaction,
        APPLICATION_SHEET_ID,
        Some(application.application_filing_id),
        current_user_id,
        &answers,
    )
    .await
    .map_err(AtcApplicationRouteError::Database)?;
    transaction
        .commit()
        .await
        .map_err(AtcApplicationRouteError::Database)?;

    let application = application_repository::find_by_id(services.db(), application.id)
        .await
        .map_err(AtcApplicationRouteError::Database)?
        .ok_or(AtcApplicationRouteError::ApplicationNotFound)?;
    application_to_dto(&services, application, false, current_user_id)
        .await
        .map(Json)
}

#[utoipa::path(get, path = "api/atc/applications/sheet", tag = "ATC", security(("bearerAuth" = [])), responses((status = 200, description = "Successful response", body = SheetDto)))]
async fn get_application_sheet(
    State(services): State<Services>,
) -> Result<Json<SheetDto>, AtcApplicationRouteError> {
    sheet_dto(&services, APPLICATION_SHEET_ID, "ATC Application Sheet")
        .await
        .map(Json)
}

#[utoipa::path(get, path = "api/atc/applications/review-sheet", tag = "ATC", security(("bearerAuth" = [])), responses((status = 200, description = "Successful response", body = SheetDto)))]
async fn get_review_sheet(
    State(services): State<Services>,
) -> Result<Json<SheetDto>, AtcApplicationRouteError> {
    sheet_dto(&services, REVIEW_SHEET_ID, "ATC Application Review Sheet")
        .await
        .map(Json)
}

#[utoipa::path(put, path = "api/atc/applications/{id}/review", tag = "ATC", security(("bearerAuth" = [])), params(("id" = String, Path, description = "Application ULID")), responses((status = 200, description = "Successful response", body = AtcApplicationDto)))]
async fn review_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(request): Json<AtcApplicationReviewRequest>,
) -> Result<Json<AtcApplicationDto>, AtcApplicationRouteError> {
    require_role(&current_user, UserRole::ControllerTrainingDirectorAssistant)?;
    let current_user_id = current_user
        .user_id
        .ok_or(AtcApplicationRouteError::Unauthorized)?;
    let application_id = parse_ulid_uuid(&id)?;
    let application = application_repository::find_by_id(services.db(), application_id)
        .await
        .map_err(AtcApplicationRouteError::Database)?
        .ok_or(AtcApplicationRouteError::ApplicationNotFound)?;
    let answers = request
        .review_answers
        .into_iter()
        .map(SheetAnswerSave::from)
        .collect::<Vec<_>>();
    let mut transaction = services
        .db()
        .begin()
        .await
        .map_err(AtcApplicationRouteError::Database)?;
    let filing_id = sheet_repository::set_filing(
        &mut transaction,
        REVIEW_SHEET_ID,
        application.review_filing_id,
        current_user_id,
        &answers,
    )
    .await
    .map_err(AtcApplicationRouteError::Database)?;
    transaction
        .commit()
        .await
        .map_err(AtcApplicationRouteError::Database)?;

    let application = application_repository::set_review(
        services.db(),
        application_id,
        request.status.as_db_str(),
        filing_id,
    )
    .await
    .map_err(AtcApplicationRouteError::Database)?
    .ok_or(AtcApplicationRouteError::ApplicationNotFound)?;

    application_to_dto(&services, application, true, current_user_id)
        .await
        .map(Json)
}

async fn get_visible_application(
    services: &Services,
    id: Uuid,
    current_user_id: Uuid,
    is_admin: bool,
) -> Result<AtcApplicationRecord, AtcApplicationRouteError> {
    let application = application_repository::find_by_id(services.db(), id)
        .await
        .map_err(AtcApplicationRouteError::Database)?
        .ok_or(AtcApplicationRouteError::ApplicationNotFound)?;
    if !is_admin && application.user_id != current_user_id {
        return Err(AtcApplicationRouteError::ApplicationNotFound);
    }

    Ok(application)
}

async fn application_to_dto(
    services: &Services,
    application: AtcApplicationRecord,
    is_admin: bool,
    current_user_id: Uuid,
) -> Result<AtcApplicationDto, AtcApplicationRouteError> {
    let application_filing_answers =
        sheet_repository::list_answers_by_filing(services.db(), application.application_filing_id)
            .await
            .map_err(AtcApplicationRouteError::Database)?
            .into_iter()
            .map(SheetFieldAnswerDto::from)
            .collect();
    let review_filing_answers = match application.review_filing_id {
        Some(review_filing_id) => Some(
            sheet_repository::list_answers_by_filing(services.db(), review_filing_id)
                .await
                .map_err(AtcApplicationRouteError::Database)?
                .into_iter()
                .map(SheetFieldAnswerDto::from)
                .collect(),
        ),
        None => None,
    };

    Ok(AtcApplicationDto::from_record(
        application,
        is_admin,
        current_user_id,
        application_filing_answers,
        review_filing_answers,
    ))
}

async fn sheet_dto(
    services: &Services,
    sheet_id: &str,
    sheet_name: &str,
) -> Result<SheetDto, AtcApplicationRouteError> {
    sheet_repository::ensure_sheet(services.db(), sheet_id, sheet_name)
        .await
        .map_err(AtcApplicationRouteError::Database)?;
    let sheet = sheet_repository::find_sheet(services.db(), sheet_id)
        .await
        .map_err(AtcApplicationRouteError::Database)?
        .ok_or(AtcApplicationRouteError::SheetNotFound)?;
    let fields = sheet_repository::list_fields(services.db(), sheet_id)
        .await
        .map_err(AtcApplicationRouteError::Database)?;

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

fn require_role(
    current_user: &CurrentUser,
    role: UserRole,
) -> Result<(), AtcApplicationRouteError> {
    if current_user.has_role(role) {
        Ok(())
    } else {
        Err(AtcApplicationRouteError::Forbidden)
    }
}

fn parse_ulid_uuid(id: &str) -> Result<Uuid, AtcApplicationRouteError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| AtcApplicationRouteError::InvalidId)
}

#[derive(Deserialize, utoipa::ToSchema)]
struct AtcApplicationRequest {
    request_answers: Vec<SheetRequestField>,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct AtcApplicationReviewRequest {
    status: AtcApplicationStatus,
    review_answers: Vec<SheetRequestField>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
enum AtcApplicationStatus {
    Submitted,
    InWaitlist,
    Approved,
    Rejected,
    Aborted,
}

impl AtcApplicationStatus {
    fn as_db_str(self) -> &'static str {
        match self {
            Self::Submitted => "Submitted",
            Self::InWaitlist => "InWaitlist",
            Self::Approved => "Approved",
            Self::Rejected => "Rejected",
            Self::Aborted => "Aborted",
        }
    }
}

fn parse_status(status: &str) -> AtcApplicationStatus {
    match status {
        "InWaitlist" => AtcApplicationStatus::InWaitlist,
        "Approved" => AtcApplicationStatus::Approved,
        "Rejected" => AtcApplicationStatus::Rejected,
        "Aborted" => AtcApplicationStatus::Aborted,
        _ => AtcApplicationStatus::Submitted,
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct AtcApplicationSummaryDto {
    id: String,
    user_id: String,
    user: UserDto,
    applied_at: DateTime<Utc>,
    status: AtcApplicationStatus,
}

impl AtcApplicationSummaryDto {
    fn from_record(
        application: AtcApplicationRecord,
        is_admin: bool,
        current_user_id: Uuid,
    ) -> Self {
        Self {
            id: Ulid::from(application.id).to_string(),
            user_id: Ulid::from(application.user_id).to_string(),
            user: UserDto::from_application_user(
                &application,
                is_admin || application.user_id == current_user_id,
            ),
            applied_at: application.applied_at,
            status: parse_status(&application.status),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct AtcApplicationDto {
    id: String,
    user_id: String,
    user: UserDto,
    applied_at: DateTime<Utc>,
    status: AtcApplicationStatus,
    application_filing_answers: Vec<SheetFieldAnswerDto>,
    review_filing_answers: Option<Vec<SheetFieldAnswerDto>>,
}

impl AtcApplicationDto {
    fn from_record(
        application: AtcApplicationRecord,
        is_admin: bool,
        current_user_id: Uuid,
        application_filing_answers: Vec<SheetFieldAnswerDto>,
        review_filing_answers: Option<Vec<SheetFieldAnswerDto>>,
    ) -> Self {
        Self {
            id: Ulid::from(application.id).to_string(),
            user_id: Ulid::from(application.user_id).to_string(),
            user: UserDto::from_application_user(
                &application,
                is_admin || application.user_id == current_user_id,
            ),
            applied_at: application.applied_at,
            status: parse_status(&application.status),
            application_filing_answers,
            review_filing_answers,
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
    fn from_application_user(application: &AtcApplicationRecord, show_full_name: bool) -> Self {
        Self {
            id: Ulid::from(application.user_id).to_string(),
            cid: application.user_cid.clone(),
            full_name: if show_full_name {
                application.user_full_name.clone()
            } else {
                String::new()
            },
            created_at: application.user_created_at,
            updated_at: application.user_updated_at,
            roles: roles_to_dto(&application.user_roles),
            direct_roles: direct_roles_to_dto(&application.user_roles),
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
enum AtcApplicationRouteError {
    ApplicationAlreadyExists,
    ApplicationCannotUpdate,
    ApplicationNotFound,
    Database(sqlx::Error),
    Forbidden,
    InvalidId,
    SheetNotFound,
    Unauthorized,
}

impl IntoResponse for AtcApplicationRouteError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AtcApplicationRouteError::ApplicationAlreadyExists => (
                StatusCode::CONFLICT,
                "ATC application already exists".into(),
            ),
            AtcApplicationRouteError::ApplicationCannotUpdate => {
                (StatusCode::CONFLICT, "ATC application cannot update".into())
            }
            AtcApplicationRouteError::ApplicationNotFound => {
                (StatusCode::NOT_FOUND, "ATC application not found".into())
            }
            AtcApplicationRouteError::Database(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            AtcApplicationRouteError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            AtcApplicationRouteError::InvalidId => (StatusCode::BAD_REQUEST, "invalid id".into()),
            AtcApplicationRouteError::SheetNotFound => {
                (StatusCode::NOT_FOUND, "sheet not found".into())
            }
            AtcApplicationRouteError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "unauthorized".into())
            }
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct ErrorResponse {
    message: String,
}
