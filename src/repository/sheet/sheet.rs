use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, FromRow)]
pub struct SheetRecord {
    pub id: String,
    pub name: String,
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
