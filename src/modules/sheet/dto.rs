use serde::{Deserialize, Serialize};

use crate::error::ApiError;

use super::models::{Sheet, SheetAnswer, SheetAnswerSave, SheetField, SheetFieldSave, SheetSave};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SheetRequestField {
    pub id: String,
    pub answer: String,
}

impl From<SheetRequestField> for SheetAnswerSave {
    fn from(answer: SheetRequestField) -> Self {
        Self {
            field_id: answer.id,
            answer: answer.answer,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SheetDto {
    pub id: String,
    pub name: String,
    pub fields: Vec<SheetFieldDto>,
}

impl SheetDto {
    pub fn from_entities(sheet: Sheet, fields: Vec<SheetField>) -> Self {
        Self {
            id: sheet.id,
            name: sheet.name,
            fields: fields.into_iter().map(SheetFieldDto::from).collect(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SheetFieldAnswerDto {
    pub field: SheetFieldDto,
    pub answer: String,
}

impl SheetFieldAnswerDto {
    pub fn from_entities(answer: SheetAnswer, field: SheetField) -> Self {
        Self {
            field: SheetFieldDto::from(field),
            answer: answer.answer,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SheetFieldKind {
    ShortText,
    LongText,
    SingleChoice,
}

impl std::fmt::Display for SheetFieldKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.serialize(f)
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SheetFieldDto {
    pub sheet_id: String,
    pub id: String,
    #[schema(format = "uint32")]
    pub sequence: u32,
    pub name_zh: String,
    pub name_en: Option<String>,
    #[schema(value_type = SheetFieldKind)]
    pub kind: String,
    pub single_choice_options: Vec<String>,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
    pub is_deleted: bool,
}

impl From<SheetField> for SheetFieldDto {
    fn from(field: SheetField) -> Self {
        Self {
            sheet_id: field.sheet_id,
            id: field.id,
            sequence: u32::try_from(field.sequence).unwrap_or_default(),
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SheetSaveRequest {
    pub name: String,
    pub fields: Vec<SheetFieldSaveRequest>,
}

impl From<SheetSaveRequest> for SheetSave {
    fn from(request: SheetSaveRequest) -> Self {
        Self {
            name: request.name,
            fields: request
                .fields
                .into_iter()
                .map(SheetFieldSave::from)
                .collect(),
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SheetFieldSaveRequest {
    pub id: String,
    #[schema(format = "uint32")]
    pub sequence: u32,
    pub name_zh: String,
    pub name_en: Option<String>,
    pub kind: SheetFieldKind,
    pub single_choice_options: Vec<String>,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
}

impl From<SheetFieldSaveRequest> for SheetFieldSave {
    fn from(field: SheetFieldSaveRequest) -> Self {
        Self {
            id: field.id,
            sequence: i64::from(field.sequence),
            name_zh: field.name_zh,
            name_en: field.name_en,
            kind: field.kind.to_string(),
            single_choice_options: field.single_choice_options,
            description_zh: field.description_zh,
            description_en: field.description_en,
            is_deleted: false,
        }
    }
}

pub fn validate_sheet_request(request: &SheetSaveRequest) -> Result<(), ApiError> {
    let mut field_ids = std::collections::HashSet::with_capacity(request.fields.len());
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
