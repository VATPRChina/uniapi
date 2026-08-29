#![allow(async_fn_in_trait)]

use chrono::Utc;

use super::models::{AtcPosition, AtcPositionSave};

const SELECT: &str = r#"
    SELECT category, callsign, is_tier_2, callsign_zh, callsign_en,
           frequency_khz, cpdlc_code, remarks, created_at, updated_at
    FROM public.atc_position
"#;

pub(crate) trait AtcPositionRepository<'executor> {
    async fn list_atc_positions(self) -> Result<Vec<AtcPosition>, sqlx::Error>;
    async fn find_atc_position(self, callsign: &str) -> Result<Option<AtcPosition>, sqlx::Error>;
    async fn find_atc_position_for_update(
        self,
        callsign: &str,
    ) -> Result<Option<AtcPosition>, sqlx::Error>;
    async fn create_atc_position(
        self,
        position: AtcPositionSave,
    ) -> Result<AtcPosition, sqlx::Error>;
    async fn update_atc_position(
        self,
        callsign: &str,
        position: AtcPositionSave,
    ) -> Result<Option<AtcPosition>, sqlx::Error>;
    async fn delete_atc_position(self, callsign: &str) -> Result<bool, sqlx::Error>;
}

impl<'executor, E> AtcPositionRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_atc_positions(self) -> Result<Vec<AtcPosition>, sqlx::Error> {
        sqlx::query_as::<_, AtcPosition>(&format!(
            r#"{SELECT}
            ORDER BY CASE
                WHEN callsign ~ '_CTR$' THEN 0
                WHEN callsign ~ '_APP$' THEN 1
                WHEN callsign ~ '_TWR$' THEN 2
                WHEN callsign ~ '_GND$' THEN 3
                WHEN callsign ~ '_RMP$' THEN 4
                WHEN callsign ~ '_DEL$' THEN 5
                ELSE 6
            END,
            callsign"#
        ))
        .fetch_all(self)
        .await
    }

    async fn find_atc_position(self, callsign: &str) -> Result<Option<AtcPosition>, sqlx::Error> {
        sqlx::query_as::<_, AtcPosition>(&format!("{SELECT} WHERE callsign = $1"))
            .bind(callsign)
            .fetch_optional(self)
            .await
    }

    async fn find_atc_position_for_update(
        self,
        callsign: &str,
    ) -> Result<Option<AtcPosition>, sqlx::Error> {
        sqlx::query_as::<_, AtcPosition>(&format!("{SELECT} WHERE callsign = $1 FOR UPDATE"))
            .bind(callsign)
            .fetch_optional(self)
            .await
    }

    async fn create_atc_position(
        self,
        position: AtcPositionSave,
    ) -> Result<AtcPosition, sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "atc_position",
            "modifying data"
        );
        let now = Utc::now();
        sqlx::query_as::<_, AtcPosition>(
            r#"
            INSERT INTO public.atc_position (
                category, callsign, is_tier_2, callsign_zh, callsign_en,
                frequency_khz, cpdlc_code, remarks, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
            RETURNING category, callsign, is_tier_2, callsign_zh, callsign_en,
                      frequency_khz, cpdlc_code, remarks, created_at, updated_at
            "#,
        )
        .bind(position.category.to_string())
        .bind(position.callsign)
        .bind(position.is_tier_2)
        .bind(position.callsign_zh)
        .bind(position.callsign_en)
        .bind(position.frequency_khz)
        .bind(position.cpdlc_code)
        .bind(position.remarks)
        .bind(now)
        .fetch_one(self)
        .await
    }

    async fn update_atc_position(
        self,
        callsign: &str,
        position: AtcPositionSave,
    ) -> Result<Option<AtcPosition>, sqlx::Error> {
        tracing::info!(
            operation = "update",
            repository = "atc_position",
            "modifying data"
        );
        sqlx::query_as::<_, AtcPosition>(
            r#"
            UPDATE public.atc_position
            SET category = $2,
                is_tier_2 = $3,
                callsign_zh = $4,
                callsign_en = $5,
                frequency_khz = $6,
                cpdlc_code = $7,
                remarks = $8,
                updated_at = now()
            WHERE callsign = $1
            RETURNING category, callsign, is_tier_2, callsign_zh, callsign_en,
                      frequency_khz, cpdlc_code, remarks, created_at, updated_at
            "#,
        )
        .bind(callsign)
        .bind(position.category.to_string())
        .bind(position.is_tier_2)
        .bind(position.callsign_zh)
        .bind(position.callsign_en)
        .bind(position.frequency_khz)
        .bind(position.cpdlc_code)
        .bind(position.remarks)
        .fetch_optional(self)
        .await
    }

    async fn delete_atc_position(self, callsign: &str) -> Result<bool, sqlx::Error> {
        tracing::info!(
            operation = "delete",
            repository = "atc_position",
            "modifying data"
        );
        Ok(
            sqlx::query("DELETE FROM public.atc_position WHERE callsign = $1")
                .bind(callsign)
                .execute(self)
                .await?
                .rows_affected()
                > 0,
        )
    }
}
