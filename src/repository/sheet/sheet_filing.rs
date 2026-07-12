use std::collections::{HashMap, HashSet};

use chrono::Utc;

use ulid::Ulid;
use uuid::Uuid;

use crate::repository::sheet::sheet_field::SheetFieldRecord;
use crate::repository::sheet::sheet_filing_answer::SheetAnswerSave;

pub trait SheetFilingTransactionExt {
    async fn set_sheet_filing(
        &mut self,
        sheet_id: &str,
        filing_id: Option<Uuid>,
        user_id: Uuid,
        answers: &[SheetAnswerSave],
    ) -> Result<Uuid, sqlx::Error>;
}

impl SheetFilingTransactionExt for sqlx::Transaction<'_, sqlx::Postgres> {
    async fn set_sheet_filing(
        &mut self,
        sheet_id: &str,
        filing_id: Option<Uuid>,
        user_id: Uuid,
        answers: &[SheetAnswerSave],
    ) -> Result<Uuid, sqlx::Error> {
        tracing::info!(
            operation = "set",
            repository = "src/repository/sheet/sheet_filing.rs",
            "modifying data"
        );

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
                .fetch_one(&mut **self)
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
                .execute(&mut **self)
                .await?;
                filing_id
            }
        };

        let fields = sqlx::query_as::<_, SheetFieldRecord>(
            r#"
        SELECT sheet_id, id, sequence, name_zh, name_en, kind,
               single_choice_options, description_zh, description_en, is_deleted
        FROM public.sheet_field
        WHERE sheet_id = $1 AND is_deleted = FALSE
        "#,
        )
        .bind(sheet_id)
        .fetch_all(&mut **self)
        .await?;

        let fields_by_id = fields
            .iter()
            .map(|field| (field.id.as_str(), field))
            .collect::<HashMap<_, _>>();
        let answer_ids = answers
            .iter()
            .map(|answer| answer.field_id.as_str())
            .collect::<HashSet<_>>();
        if answers
            .iter()
            .any(|answer| !fields_by_id.contains_key(answer.field_id.as_str()))
            || fields
                .iter()
                .any(|field| field.kind != "Description" && !answer_ids.contains(field.id.as_str()))
        {
            return Err(sqlx::Error::RowNotFound);
        }

        sqlx::query(
            r#"
        DELETE FROM public.sheet_filing_answer
        WHERE filing_id = $1
        "#,
        )
        .bind(filing_id)
        .execute(&mut **self)
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
            .execute(&mut **self)
            .await?;
        }

        Ok(filing_id)
    }
}
