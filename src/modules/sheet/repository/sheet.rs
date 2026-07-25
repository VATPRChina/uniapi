use crate::modules::sheet::models::{Sheet, SheetSave};

pub trait SheetTransactionRepository {
    async fn upsert_sheet(
        &mut self,
        sheet_id: &str,
        sheet: SheetSave,
    ) -> Result<Sheet, sqlx::Error>;
}

impl SheetTransactionRepository for sqlx::Transaction<'_, sqlx::Postgres> {
    async fn upsert_sheet(
        &mut self,
        sheet_id: &str,
        sheet: SheetSave,
    ) -> Result<Sheet, sqlx::Error> {
        tracing::info!(
            operation = "upsert",
            repository = "src/modules/sheet/repository/sheet.rs",
            "modifying data"
        );

        let record = sqlx::query_as::<_, Sheet>(
            r#"
        INSERT INTO public.sheet (id, name)
        VALUES ($1, $2)
        ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name
        RETURNING id, name
        "#,
        )
        .bind(sheet_id)
        .bind(sheet.name)
        .fetch_one(&mut **self)
        .await?;

        for field in &sheet.fields {
            sqlx::query(
                r#"
            INSERT INTO public.sheet_field (
                sheet_id,
                id,
                sequence,
                name_zh,
                name_en,
                kind,
                single_choice_options,
                description_zh,
                description_en,
                is_deleted
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (sheet_id, id) DO UPDATE
            SET sequence = EXCLUDED.sequence,
                name_zh = EXCLUDED.name_zh,
                name_en = EXCLUDED.name_en,
                kind = EXCLUDED.kind,
                single_choice_options = EXCLUDED.single_choice_options,
                description_zh = EXCLUDED.description_zh,
                description_en = EXCLUDED.description_en,
                is_deleted = EXCLUDED.is_deleted
            "#,
            )
            .bind(sheet_id)
            .bind(&field.id)
            .bind(field.sequence)
            .bind(&field.name_zh)
            .bind(&field.name_en)
            .bind(&field.kind)
            .bind(&field.single_choice_options)
            .bind(&field.description_zh)
            .bind(&field.description_en)
            .bind(field.is_deleted)
            .execute(&mut **self)
            .await?;
        }

        let field_ids = sheet
            .fields
            .iter()
            .map(|field| field.id.clone())
            .collect::<Vec<_>>();
        sqlx::query(
            r#"
        UPDATE public.sheet_field
        SET is_deleted = TRUE
        WHERE sheet_id = $1
          AND NOT (id = ANY($2::text[]))
        "#,
        )
        .bind(sheet_id)
        .bind(&field_ids)
        .execute(&mut **self)
        .await?;

        Ok(record)
    }
}

pub trait SheetRepository<'executor> {
    async fn ensure_sheet(self, sheet_id: &str, name: &str) -> Result<(), sqlx::Error>;

    async fn list_sheet(self) -> Result<Vec<Sheet>, sqlx::Error>;

    async fn find_sheet(self, sheet_id: &str) -> Result<Option<Sheet>, sqlx::Error>;
}

impl<'executor, E> SheetRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn ensure_sheet(self, sheet_id: &str, name: &str) -> Result<(), sqlx::Error> {
        tracing::info!(
            operation = "ensure",
            repository = "src/modules/sheet/repository/sheet.rs",
            "modifying data"
        );

        sqlx::query(
            r#"
        INSERT INTO public.sheet (id, name)
        VALUES ($1, $2)
        ON CONFLICT (id) DO NOTHING
        "#,
        )
        .bind(sheet_id)
        .bind(name)
        .execute(self)
        .await?;

        Ok(())
    }
    async fn list_sheet(self) -> Result<Vec<Sheet>, sqlx::Error> {
        sqlx::query_as::<_, Sheet>(
            r#"
        SELECT id, name
        FROM public.sheet
        ORDER BY id
        "#,
        )
        .fetch_all(self)
        .await
    }
    async fn find_sheet(self, sheet_id: &str) -> Result<Option<Sheet>, sqlx::Error> {
        sqlx::query_as::<_, Sheet>(
            r#"
        SELECT id, name
        FROM public.sheet
        WHERE id = $1
        "#,
        )
        .bind(sheet_id)
        .fetch_optional(self)
        .await
    }
}
