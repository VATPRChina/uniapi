use sqlx::{FromRow, PgPool};

use crate::repository::sheet::sheet_field::SheetFieldSave;

#[derive(Debug, Clone, FromRow)]
pub struct SheetRecord {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SheetSave {
    pub name: String,
    pub fields: Vec<SheetFieldSave>,
}

pub async fn ensure(db: &PgPool, sheet_id: &str, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO public.sheet (id, name)
        VALUES ($1, $2)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(sheet_id)
    .bind(name)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn list(db: &PgPool) -> Result<Vec<SheetRecord>, sqlx::Error> {
    sqlx::query_as::<_, SheetRecord>(
        r#"
        SELECT id, name
        FROM public.sheet
        ORDER BY id
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn find(db: &PgPool, sheet_id: &str) -> Result<Option<SheetRecord>, sqlx::Error> {
    sqlx::query_as::<_, SheetRecord>(
        r#"
        SELECT id, name
        FROM public.sheet
        WHERE id = $1
        "#,
    )
    .bind(sheet_id)
    .fetch_optional(db)
    .await
}

pub async fn upsert(
    db: &PgPool,
    sheet_id: &str,
    sheet: SheetSave,
) -> Result<SheetRecord, sqlx::Error> {
    let mut transaction = db.begin().await?;

    let record = sqlx::query_as::<_, SheetRecord>(
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
    .fetch_one(&mut *transaction)
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
        .execute(&mut *transaction)
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
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(record)
}
