use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct SheetAnswerRecord {
    pub answer: String,
    pub sheet_id: String,
    pub field_id: String,
    pub field_sequence: i64,
    pub field_name_zh: String,
    pub field_name_en: Option<String>,
    pub field_kind: String,
    pub field_single_choice_options: Vec<String>,
    pub field_description_zh: Option<String>,
    pub field_description_en: Option<String>,
    pub field_is_deleted: bool,
}

#[derive(Debug, Clone)]
pub struct SheetAnswerSave {
    pub field_id: String,
    pub answer: String,
}

pub async fn list_by_filing(
    db: &PgPool,
    filing_id: Uuid,
) -> Result<Vec<SheetAnswerRecord>, sqlx::Error> {
    sqlx::query_as::<_, SheetAnswerRecord>(
        r#"
        SELECT sheet_filing_answer.answer,
               sheet_field.sheet_id,
               sheet_field.id AS field_id,
               sheet_field.sequence AS field_sequence,
               sheet_field.name_zh AS field_name_zh,
               sheet_field.name_en AS field_name_en,
               sheet_field.kind AS field_kind,
               sheet_field.single_choice_options AS field_single_choice_options,
               sheet_field.description_zh AS field_description_zh,
               sheet_field.description_en AS field_description_en,
               sheet_field.is_deleted AS field_is_deleted
        FROM public.sheet_filing_answer
        INNER JOIN public.sheet_field
            ON sheet_field.sheet_id = sheet_filing_answer.sheet_id
           AND sheet_field.id = sheet_filing_answer.field_id
        WHERE sheet_filing_answer.filing_id = $1
        ORDER BY sheet_field.sequence
        "#,
    )
    .bind(filing_id)
    .fetch_all(db)
    .await
}
