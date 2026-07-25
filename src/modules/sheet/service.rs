use futures::future::try_join_all;
use sqlx::PgPool;

use uuid::Uuid;

use super::models::{Sheet, SheetAnswer, SheetField, SheetSave};
use super::repository::sheet::{SheetRepository, SheetTransactionRepository};
use super::repository::sheet_field::SheetFieldRepository;
use super::repository::sheet_filing_answer::SheetFilingAnswerRepository;

#[derive(Clone)]
pub struct SheetService {
    db: PgPool,
}

impl SheetService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list(&self) -> Result<Vec<SheetView>, SheetServiceError> {
        let sheets = self.db.list_sheet().await?;
        try_join_all(sheets.into_iter().map(|sheet| self.with_fields(sheet))).await
    }

    pub async fn find(&self, sheet_id: &str) -> Result<SheetView, SheetServiceError> {
        let sheet = self
            .db
            .find_sheet(sheet_id)
            .await?
            .ok_or_else(|| SheetServiceError::NotFound(sheet_id.to_string()))?;
        self.with_fields(sheet).await
    }

    pub async fn ensure(&self, sheet_id: &str, name: &str) -> Result<SheetView, SheetServiceError> {
        self.db.ensure_sheet(sheet_id, name).await?;
        self.find(sheet_id).await
    }

    pub async fn upsert(
        &self,
        sheet_id: &str,
        sheet: SheetSave,
    ) -> Result<SheetView, SheetServiceError> {
        let mut transaction = self.db.begin().await?;
        let sheet = transaction.upsert_sheet(sheet_id, sheet).await?;
        transaction.commit().await?;
        self.with_fields(sheet).await
    }

    pub async fn filing_answers(
        &self,
        filing_id: Uuid,
    ) -> Result<Vec<SheetAnswerView>, SheetServiceError> {
        let answers = self
            .db
            .list_sheet_filing_answer_by_filing(filing_id)
            .await?;
        try_join_all(answers.into_iter().map(|answer| self.with_field(answer))).await
    }

    async fn with_fields(&self, sheet: Sheet) -> Result<SheetView, SheetServiceError> {
        let fields = self.db.list_sheet_field(&sheet.id).await?;
        Ok(SheetView { sheet, fields })
    }

    async fn with_field(&self, answer: SheetAnswer) -> Result<SheetAnswerView, SheetServiceError> {
        let field = self
            .db
            .find_sheet_field(&answer.sheet_id, &answer.field_id)
            .await?
            .ok_or_else(|| SheetServiceError::FieldNotFound {
                sheet_id: answer.sheet_id.clone(),
                field_id: answer.field_id.clone(),
            })?;
        Ok(SheetAnswerView { answer, field })
    }
}

#[derive(Debug)]
pub struct SheetView {
    pub sheet: Sheet,
    pub fields: Vec<SheetField>,
}

#[derive(Debug)]
pub struct SheetAnswerView {
    pub answer: SheetAnswer,
    pub field: SheetField,
}

#[derive(Debug, thiserror::Error)]
pub enum SheetServiceError {
    #[error("sheet {0} not found")]
    NotFound(String),
    #[error("field {field_id} in sheet {sheet_id} not found")]
    FieldNotFound { sheet_id: String, field_id: String },
    #[error("failed to access sheet data: {0}")]
    Database(#[from] sqlx::Error),
}
