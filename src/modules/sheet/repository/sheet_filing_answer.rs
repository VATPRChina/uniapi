use uuid::Uuid;

use crate::modules::sheet::models::SheetAnswer;

fn list_by_filing_sql() -> &'static str {
    r#"
    SELECT sheet_filing_answer.answer,
           sheet_filing_answer.sheet_id,
           sheet_filing_answer.field_id
    FROM public.sheet_filing_answer
    INNER JOIN public.sheet_field
        ON sheet_field.sheet_id = sheet_filing_answer.sheet_id
       AND sheet_field.id = sheet_filing_answer.field_id
    WHERE sheet_filing_answer.filing_id = $1
    ORDER BY sheet_field.sequence
    "#
}

pub trait SheetFilingAnswerRepository<'executor> {
    async fn list_sheet_filing_answer_by_filing(
        self,
        filing_id: Uuid,
    ) -> Result<Vec<SheetAnswer>, sqlx::Error>;

    async fn list_sheet_filing_answer_by_filing_in_transaction(
        self,
        filing_id: Uuid,
    ) -> Result<Vec<SheetAnswer>, sqlx::Error>;
}

impl<'executor, E> SheetFilingAnswerRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_sheet_filing_answer_by_filing(
        self,
        filing_id: Uuid,
    ) -> Result<Vec<SheetAnswer>, sqlx::Error> {
        sqlx::query_as::<_, SheetAnswer>(list_by_filing_sql())
            .bind(filing_id)
            .fetch_all(self)
            .await
    }
    async fn list_sheet_filing_answer_by_filing_in_transaction(
        self,
        filing_id: Uuid,
    ) -> Result<Vec<SheetAnswer>, sqlx::Error> {
        sqlx::query_as::<_, SheetAnswer>(list_by_filing_sql())
            .bind(filing_id)
            .fetch_all(self)
            .await
    }
}
