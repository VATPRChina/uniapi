use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};

use crate::error::ApiError;
use crate::model::user_role::UserRole;
use crate::modules::sheet::dto::{SheetDto, SheetSaveRequest, validate_sheet_request};
use crate::modules::sheet::service::SheetView;
use crate::modules::user::middleware::CurrentUser;
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
    Ok(Json(
        services
            .sheet()
            .list()
            .await?
            .into_iter()
            .map(sheet_to_dto)
            .collect(),
    ))
}

#[utoipa::path(get, path = "api/sheets/{sheetId}", tag = "Sheets", params(("sheetId" = String, Path, description = "Sheet ID")), responses((status = 200, description = "Successful response", body = SheetDto)))]
async fn get_sheet(
    State(services): State<Services>,
    Path(sheet_id): Path<String>,
) -> Result<Json<SheetDto>, ApiError> {
    Ok(Json(sheet_to_dto(services.sheet().find(&sheet_id).await?)))
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

    Ok(Json(sheet_to_dto(
        services.sheet().upsert(&sheet_id, request.into()).await?,
    )))
}

fn sheet_to_dto(view: SheetView) -> SheetDto {
    SheetDto::from_entities(view.sheet, view.fields)
}
