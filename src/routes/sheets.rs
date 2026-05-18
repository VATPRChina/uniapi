use std::collections::HashSet;

use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};

use crate::auth::CurrentUser;
use crate::dto::*;
use crate::model::user_role::UserRole;
use crate::repository::sheet::sheet::{self as sheet_repository};
use crate::repository::sheet::sheet_field::{self as sheet_field_repository};
use crate::routes::ApiError;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_sheets, get_sheet, upsert_sheet))]
pub(crate) struct ApiDoc;

pub fn build_sheet_routes() -> Router<Services> {
    Router::new()
        .route("/", get(list_sheets))
        .route("/{sheet_id}", get(get_sheet))
        .route("/{sheet_id}", put(upsert_sheet))
}

#[utoipa::path(get, path = "api/sheets", tag = "Sheets", responses((status = 200, description = "Successful response", body = Vec<SheetDto>)))]
async fn list_sheets(State(services): State<Services>) -> Result<Json<Vec<SheetDto>>, ApiError> {
    let sheets = sheet_repository::list(services.db()).await?;
    let mut response = Vec::with_capacity(sheets.len());

    for sheet in sheets {
        let fields = sheet_field_repository::list(services.db(), &sheet.id).await?;
        response.push(SheetDto::from_records(sheet, fields));
    }

    Ok(Json(response))
}

#[utoipa::path(get, path = "api/sheets/{sheetId}", tag = "Sheets", params(("sheetId" = String, Path, description = "Sheet ID")), responses((status = 200, description = "Successful response", body = SheetDto)))]
async fn get_sheet(
    State(services): State<Services>,
    Path(sheet_id): Path<String>,
) -> Result<Json<SheetDto>, ApiError> {
    let sheet = sheet_repository::find(services.db(), &sheet_id)
        .await?
        .ok_or_else(|| ApiError::not_found("sheet", &sheet_id))?;
    let fields = sheet_field_repository::list(services.db(), &sheet_id).await?;

    Ok(Json(SheetDto::from_records(sheet, fields)))
}

#[utoipa::path(put, path = "api/sheets/{sheetId}", tag = "Sheets", security(("oauth2" = [])), params(("sheetId" = String, Path, description = "Sheet ID")), request_body = SheetSaveRequest, responses((status = 200, description = "Successful response", body = SheetDto)))]
async fn upsert_sheet(
    State(services): State<Services>,
    current_user: CurrentUser,
    Path(sheet_id): Path<String>,
    Json(request): Json<SheetSaveRequest>,
) -> Result<Json<SheetDto>, ApiError> {
    current_user.require_role(UserRole::Staff)?;
    validate_sheet_request(&request)?;

    let sheet = sheet_repository::upsert(services.db(), &sheet_id, request.into()).await?;
    let fields = sheet_field_repository::list(services.db(), &sheet_id).await?;

    Ok(Json(SheetDto::from_records(sheet, fields)))
}

fn validate_sheet_request(request: &SheetSaveRequest) -> Result<(), ApiError> {
    let mut field_ids = HashSet::with_capacity(request.fields.len());

    for field in &request.fields {
        if field.id.trim().is_empty() {
            return Err(ApiError::bad_request(
                "fields.id",
                "field id cannot be empty",
            ));
        }

        if !field_ids.insert(field.id.as_str()) {
            return Err(ApiError::bad_request(
                "fields.id",
                "field id must be unique",
            ));
        }
    }

    Ok(())
}
