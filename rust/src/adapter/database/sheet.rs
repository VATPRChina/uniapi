use std::collections::{HashMap, HashSet};

use chrono::Utc;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct SheetRecord {
    pub id: String,
    pub name: String,
}

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

pub async fn ensure_sheet(db: &PgPool, sheet_id: &str, name: &str) -> Result<(), sqlx::Error> {
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

pub async fn find_sheet(db: &PgPool, sheet_id: &str) -> Result<Option<SheetRecord>, sqlx::Error> {
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

pub async fn list_fields(
    db: &PgPool,
    sheet_id: &str,
) -> Result<Vec<SheetFieldRecord>, sqlx::Error> {
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

pub async fn list_answers_by_filing(
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

pub async fn set_filing(
    transaction: &mut Transaction<'_, Postgres>,
    sheet_id: &str,
    filing_id: Option<Uuid>,
    user_id: Uuid,
    answers: &[SheetAnswerSave],
) -> Result<Uuid, sqlx::Error> {
    let filing_id = match filing_id {
        Some(filing_id) => {
            sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT id
                FROM public.sheet_filing
                WHERE id = $1
                "#,
            )
            .bind(filing_id)
            .fetch_one(&mut **transaction)
            .await?
        }
        None => {
            let filing_id = Uuid::from(Ulid::new());
            sqlx::query(
                r#"
                INSERT INTO public.sheet_filing (id, sheet_id, user_id, filed_at)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(filing_id)
            .bind(sheet_id)
            .bind(user_id)
            .bind(Utc::now())
            .execute(&mut **transaction)
            .await?;
            filing_id
        }
    };

    validate_answers(transaction, sheet_id, answers).await?;

    sqlx::query(
        r#"
        DELETE FROM public.sheet_filing_answer
        WHERE filing_id = $1
        "#,
    )
    .bind(filing_id)
    .execute(&mut **transaction)
    .await?;

    for answer in answers {
        sqlx::query(
            r#"
            INSERT INTO public.sheet_filing_answer (sheet_id, filing_id, field_id, answer)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(sheet_id)
        .bind(filing_id)
        .bind(&answer.field_id)
        .bind(&answer.answer)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(filing_id)
}

async fn validate_answers(
    transaction: &mut Transaction<'_, Postgres>,
    sheet_id: &str,
    answers: &[SheetAnswerSave],
) -> Result<(), sqlx::Error> {
    let fields = sqlx::query_as::<_, SheetFieldRecord>(
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
        WHERE sheet_id = $1 AND is_deleted = FALSE
        "#,
    )
    .bind(sheet_id)
    .fetch_all(&mut **transaction)
    .await?;

    let fields_by_id = fields
        .iter()
        .map(|field| (field.id.as_str(), field))
        .collect::<HashMap<_, _>>();
    let answer_ids = answers
        .iter()
        .map(|answer| answer.field_id.as_str())
        .collect::<HashSet<_>>();

    for answer in answers {
        if !fields_by_id.contains_key(answer.field_id.as_str()) {
            return Err(sqlx::Error::RowNotFound);
        }
    }

    for field in fields {
        if field.kind != "Description" && !answer_ids.contains(field.id.as_str()) {
            return Err(sqlx::Error::RowNotFound);
        }
    }

    Ok(())
}
