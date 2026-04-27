use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, FromRow)]
pub struct SheetFieldRecord {
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

pub async fn list(db: &PgPool, sheet_id: &str) -> Result<Vec<SheetFieldRecord>, sqlx::Error> {
    sqlx::query_as::<_, SheetFieldRecord>(
        r#"
        SELECT sheet_id,
               id,
               sequence,
               name_zh,
               name_en,
               kind,
               single_choice_options,
               description_zh,
               description_en,
               is_deleted
        FROM public.sheet_field
        WHERE sheet_id = $1
        ORDER BY sequence
        "#,
    )
    .bind(sheet_id)
    .fetch_all(db)
    .await
}
