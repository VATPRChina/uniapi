use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Sheet {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SheetSave {
    pub name: String,
    pub fields: Vec<SheetFieldSave>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SheetField {
    pub sheet_id: String,
    pub id: String,
    pub sequence: i64,
    pub name_zh: String,
    pub name_en: Option<String>,
    pub kind: String,
    pub single_choice_options: Vec<String>,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone)]
pub struct SheetFieldSave {
    pub id: String,
    pub sequence: i64,
    pub name_zh: String,
    pub name_en: Option<String>,
    pub kind: String,
    pub single_choice_options: Vec<String>,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct SheetAnswer {
    pub answer: String,
    pub sheet_id: String,
    pub field_id: String,
}

#[derive(Debug, Clone)]
pub struct SheetAnswerSave {
    pub field_id: String,
    pub answer: String,
}
