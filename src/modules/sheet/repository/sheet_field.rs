use crate::modules::sheet::models::SheetField;

pub trait SheetFieldRepository<'executor> {
    async fn list_sheet_field(self, sheet_id: &str) -> Result<Vec<SheetField>, sqlx::Error>;

    async fn find_sheet_field(
        self,
        sheet_id: &str,
        field_id: &str,
    ) -> Result<Option<SheetField>, sqlx::Error>;
}

impl<'executor, E> SheetFieldRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_sheet_field(self, sheet_id: &str) -> Result<Vec<SheetField>, sqlx::Error> {
        sqlx::query_as::<_, SheetField>(
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
        .fetch_all(self)
        .await
    }

    async fn find_sheet_field(
        self,
        sheet_id: &str,
        field_id: &str,
    ) -> Result<Option<SheetField>, sqlx::Error> {
        sqlx::query_as::<_, SheetField>(
            r#"
        SELECT sheet_id, id, sequence, name_zh, name_en, kind,
               single_choice_options, description_zh, description_en, is_deleted
        FROM public.sheet_field
        WHERE sheet_id = $1 AND id = $2
        "#,
        )
        .bind(sheet_id)
        .bind(field_id)
        .fetch_optional(self)
        .await
    }
}
