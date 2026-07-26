use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use ulid::Ulid;

use crate::error::ApiError;
use crate::modules::audit_log::dto::AuditLogDto;
use crate::modules::user::middleware::CurrentUser;
use crate::modules::user::models::UserRole;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    list_event_audit_logs,
    list_event_audit_logs_by_event,
    list_atc_application_audit_logs,
    list_atc_application_audit_logs_by_application,
    list_user_audit_logs,
    list_user_audit_logs_by_user,
    list_user_atc_status_audit_logs
))]
pub(crate) struct ApiDoc;

pub fn build_audit_log_routes() -> Router<Services> {
    Router::new()
        .route("/events/audit", get(list_event_audit_logs))
        .route("/events/{id}/audit", get(list_event_audit_logs_by_event))
        .route(
            "/atc/applications/audit",
            get(list_atc_application_audit_logs),
        )
        .route(
            "/atc/applications/{id}/audit",
            get(list_atc_application_audit_logs_by_application),
        )
        .route("/users/audit", get(list_user_audit_logs))
        .route("/users/{id}/audit", get(list_user_audit_logs_by_user))
        .route(
            "/users/{id}/atc/status/audit",
            get(list_user_atc_status_audit_logs),
        )
}

#[utoipa::path(
    get,
    path = "api/events/audit",
    tag = "Audit Logs",
    security(("oauth2" = [])),
    responses(
        (status = 200, description = "Event-related audit logs", body = Vec<AuditLogDto>)
    )
)]
async fn list_event_audit_logs(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<AuditLogDto>>, ApiError> {
    current_user.require_role(UserRole::Volunteer)?;
    let audit_logs = services.audit_log().list_events().await?;

    Ok(Json(audit_logs.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "api/events/{id}/audit",
    tag = "Audit Logs",
    security(("oauth2" = [])),
    params(("id" = String, Path, description = "Event ULID")),
    responses(
        (status = 200, description = "Audit logs for an event", body = Vec<AuditLogDto>)
    )
)]
async fn list_event_audit_logs_by_event(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<AuditLogDto>>, ApiError> {
    current_user.require_role(UserRole::Volunteer)?;
    let id = id.parse::<Ulid>()?.into();
    let audit_logs = services.audit_log().list_events_by_id(id).await?;

    Ok(Json(audit_logs.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "api/atc/applications/audit",
    tag = "Audit Logs",
    security(("oauth2" = [])),
    responses(
        (status = 200, description = "ATC-application-related audit logs", body = Vec<AuditLogDto>)
    )
)]
async fn list_atc_application_audit_logs(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<AuditLogDto>>, ApiError> {
    current_user.require_role(UserRole::Volunteer)?;
    let audit_logs = services.audit_log().list_atc_applications().await?;

    Ok(Json(audit_logs.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "api/atc/applications/{id}/audit",
    tag = "Audit Logs",
    security(("oauth2" = [])),
    params(("id" = String, Path, description = "ATC application ULID")),
    responses(
        (status = 200, description = "Audit logs for an ATC application", body = Vec<AuditLogDto>)
    )
)]
async fn list_atc_application_audit_logs_by_application(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<AuditLogDto>>, ApiError> {
    current_user.require_role(UserRole::Volunteer)?;
    let id = id.parse::<Ulid>()?.into();
    let audit_logs = services.audit_log().list_atc_applications_by_id(id).await?;

    Ok(Json(audit_logs.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "api/users/audit",
    tag = "Audit Logs",
    security(("oauth2" = [])),
    responses(
        (status = 200, description = "User-related audit logs", body = Vec<AuditLogDto>)
    )
)]
async fn list_user_audit_logs(
    State(services): State<Services>,
    current_user: CurrentUser,
) -> Result<Json<Vec<AuditLogDto>>, ApiError> {
    current_user.require_role(UserRole::Volunteer)?;
    let audit_logs = services.audit_log().list_users().await?;

    Ok(Json(audit_logs.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "api/users/{id}/audit",
    tag = "Audit Logs",
    security(("oauth2" = [])),
    params(("id" = String, Path, description = "User ULID")),
    responses(
        (status = 200, description = "Audit logs for a user", body = Vec<AuditLogDto>)
    )
)]
async fn list_user_audit_logs_by_user(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<AuditLogDto>>, ApiError> {
    current_user.require_role(UserRole::Volunteer)?;
    let id = id.parse::<Ulid>()?.into();
    let audit_logs = services.audit_log().list_users_by_id(id).await?;

    Ok(Json(audit_logs.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "api/users/{id}/atc/status/audit",
    tag = "Audit Logs",
    security(("oauth2" = [])),
    params(("id" = String, Path, description = "User ULID")),
    responses(
        (status = 200, description = "ATC-status audit logs for a user", body = Vec<AuditLogDto>)
    )
)]
async fn list_user_atc_status_audit_logs(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<AuditLogDto>>, ApiError> {
    current_user.require_role(UserRole::Volunteer)?;
    let id = id.parse::<Ulid>()?.into();
    let audit_logs = services.audit_log().list_user_atc_status(id).await?;

    Ok(Json(audit_logs.into_iter().map(Into::into).collect()))
}
