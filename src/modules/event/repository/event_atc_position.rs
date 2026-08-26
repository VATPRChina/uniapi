use chrono::{DateTime, Utc};
use sqlx::FromRow;
use ulid::Ulid;
use uuid::Uuid;

use crate::modules::event::models::{EventAtcBooking, EventAtcPosition, EventAtcPositionSave};

#[derive(Debug, Clone, FromRow)]
pub struct UserAtcPermissionRecord {
    pub state: String,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

fn position_select_sql(where_clause: &str) -> String {
    position_select_sql_from("public.event_atc_position", where_clause)
}

fn position_select_sql_from(source: &str, where_clause: &str) -> String {
    format!(
        r#"
        SELECT event_atc_position.id,
               event_atc_position.event_id,
               event_atc_position.callsign,
               event_atc_position.start_at,
               event_atc_position.end_at,
               event_atc_position.remarks,
               event_atc_position.position_kind_id,
               event_atc_position.minimum_controller_state,
               event_atc_position_booking.atc_booking_id AS booking_id
        FROM {source}
        LEFT JOIN public.event_atc_position_booking
            ON event_atc_position_booking.event_atc_position_id = event_atc_position.id
        {where_clause}
        "#
    )
}

pub(crate) trait EventAtcPositionRepository<'executor> {
    async fn list_event_atc_position_by_event(
        self,
        event_id: Uuid,
    ) -> Result<Vec<EventAtcPosition>, sqlx::Error>;

    async fn find_event_atc_position_by_id(
        self,
        position_id: Uuid,
    ) -> Result<Option<EventAtcPosition>, sqlx::Error>;

    async fn find_event_atc_position_by_event_and_id_in_transaction(
        self,
        event_id: Uuid,
        position_id: Uuid,
        for_update: bool,
    ) -> Result<Option<EventAtcPosition>, sqlx::Error>;

    async fn create_event_atc_position(
        self,
        event_id: Uuid,
        position: EventAtcPositionSave,
    ) -> Result<EventAtcPosition, sqlx::Error>;

    async fn update_event_atc_position(
        self,
        event_id: Uuid,
        position_id: Uuid,
        position: EventAtcPositionSave,
    ) -> Result<Option<EventAtcPosition>, sqlx::Error>;

    async fn delete_event_atc_position(
        self,
        event_id: Uuid,
        position_id: Uuid,
    ) -> Result<bool, sqlx::Error>;

    async fn user_event_atc_position_permission(
        self,
        user_id: Uuid,
        position_kind_id: &str,
    ) -> Result<Option<UserAtcPermissionRecord>, sqlx::Error>;

    async fn find_event_atc_booking(
        self,
        booking_id: Uuid,
    ) -> Result<Option<EventAtcBooking>, sqlx::Error>;
}

impl<'executor, E> EventAtcPositionRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_event_atc_position_by_event(
        self,
        event_id: Uuid,
    ) -> Result<Vec<EventAtcPosition>, sqlx::Error> {
        sqlx::query_as::<_, EventAtcPosition>(&position_select_sql(
            r#"
        WHERE event_atc_position.event_id = $1
        "#,
        ))
        .bind(event_id)
        .fetch_all(self)
        .await
    }

    async fn find_event_atc_position_by_id(
        self,
        position_id: Uuid,
    ) -> Result<Option<EventAtcPosition>, sqlx::Error> {
        sqlx::query_as::<_, EventAtcPosition>(&position_select_sql(
            "WHERE event_atc_position.id = $1",
        ))
        .bind(position_id)
        .fetch_optional(self)
        .await
    }
    async fn find_event_atc_position_by_event_and_id_in_transaction(
        self,
        event_id: Uuid,
        position_id: Uuid,
        for_update: bool,
    ) -> Result<Option<EventAtcPosition>, sqlx::Error> {
        let lock_clause = if for_update {
            "FOR UPDATE OF event_atc_position"
        } else {
            ""
        };
        sqlx::query_as::<_, EventAtcPosition>(&position_select_sql(&format!(
            r#"
        WHERE event_atc_position.event_id = $1 AND event_atc_position.id = $2
        {lock_clause}
        "#
        )))
        .bind(event_id)
        .bind(position_id)
        .fetch_optional(self)
        .await
    }
    async fn create_event_atc_position(
        self,
        event_id: Uuid,
        position: EventAtcPositionSave,
    ) -> Result<EventAtcPosition, sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "src/modules/event/repository/event_atc_position.rs",
            "modifying data"
        );

        let id = Uuid::from(Ulid::new());
        let query = r#"
        WITH inserted_position AS (
            INSERT INTO public.event_atc_position (
                id, event_id, callsign, start_at, end_at, remarks,
                position_kind_id, minimum_controller_state
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
        )
        "#
        .to_string()
            + &position_select_sql_from(
                "inserted_position AS event_atc_position",
                "WHERE event_atc_position.id = $1",
            );
        sqlx::query_as::<_, EventAtcPosition>(&query)
            .bind(id)
            .bind(event_id)
            .bind(position.callsign)
            .bind(position.start_at)
            .bind(position.end_at)
            .bind(position.remarks)
            .bind(position.position_kind_id)
            .bind(position.minimum_controller_state)
            .fetch_one(self)
            .await
    }
    async fn update_event_atc_position(
        self,
        event_id: Uuid,
        position_id: Uuid,
        position: EventAtcPositionSave,
    ) -> Result<Option<EventAtcPosition>, sqlx::Error> {
        tracing::info!(
            operation = "update",
            repository = "src/modules/event/repository/event_atc_position.rs",
            "modifying data"
        );

        let query = r#"
        WITH updated_position AS (
            UPDATE public.event_atc_position
            SET callsign = $3,
                start_at = $4,
                end_at = $5,
                remarks = $6,
                position_kind_id = $7,
                minimum_controller_state = $8
            WHERE event_id = $1 AND id = $2
            RETURNING *
        )
        "#
        .to_string()
            + &position_select_sql_from(
                "updated_position AS event_atc_position",
                "WHERE event_atc_position.id = $2",
            );
        sqlx::query_as::<_, EventAtcPosition>(&query)
            .bind(event_id)
            .bind(position_id)
            .bind(position.callsign)
            .bind(position.start_at)
            .bind(position.end_at)
            .bind(position.remarks)
            .bind(position.position_kind_id)
            .bind(position.minimum_controller_state)
            .fetch_optional(self)
            .await
    }
    async fn delete_event_atc_position(
        self,
        event_id: Uuid,
        position_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        tracing::info!(
            operation = "delete",
            repository = "src/modules/event/repository/event_atc_position.rs",
            "modifying data"
        );

        let result = sqlx::query(
            r#"
        DELETE FROM public.event_atc_position
        WHERE event_id = $1 AND id = $2
        "#,
        )
        .bind(event_id)
        .bind(position_id)
        .execute(self)
        .await?;

        Ok(result.rows_affected() > 0)
    }
    async fn user_event_atc_position_permission(
        self,
        user_id: Uuid,
        position_kind_id: &str,
    ) -> Result<Option<UserAtcPermissionRecord>, sqlx::Error> {
        sqlx::query_as::<_, UserAtcPermissionRecord>(
            r#"
        SELECT state, solo_expires_at
        FROM public.user_atc_permission
        WHERE user_id = $1 AND position_kind_id = $2
        "#,
        )
        .bind(user_id)
        .bind(position_kind_id)
        .fetch_optional(self)
        .await
    }

    async fn find_event_atc_booking(
        self,
        booking_id: Uuid,
    ) -> Result<Option<EventAtcBooking>, sqlx::Error> {
        sqlx::query_as::<_, EventAtcBooking>(
            r#"
        SELECT id, user_id, booked_at
        FROM public.atc_booking
        WHERE id = $1
        "#,
        )
        .bind(booking_id)
        .fetch_optional(self)
        .await
    }
}

pub(crate) trait EventAtcPositionTransactionRepository {
    async fn create_event_atc_position_booking(
        &mut self,
        position: &EventAtcPosition,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error>;

    async fn delete_event_atc_position_booking(
        &mut self,
        position_id: Uuid,
        atc_booking_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error>;

    async fn sync_event_atc_position_booking(
        &mut self,
        position: &EventAtcPosition,
    ) -> Result<(), sqlx::Error>;

    async fn delete_event_atc_booking(&mut self, booking_id: Uuid) -> Result<(), sqlx::Error>;
}

impl EventAtcPositionTransactionRepository for sqlx::Transaction<'_, sqlx::Postgres> {
    async fn create_event_atc_position_booking(
        &mut self,
        position: &EventAtcPosition,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        tracing::info!(
            operation = "create_booking",
            repository = "src/modules/event/repository/event_atc_position.rs",
            "modifying data"
        );

        let atc_booking_id = Uuid::from(Ulid::new());
        let now = Utc::now();
        sqlx::query(
            r#"
        INSERT INTO public.atc_booking (
            id, user_id, callsign, booked_at, start_at, end_at, remarks
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        )
        .bind(atc_booking_id)
        .bind(user_id)
        .bind(&position.callsign)
        .bind(now)
        .bind(position.start_at)
        .bind(position.end_at)
        .bind(&position.remarks)
        .execute(&mut **self)
        .await?;
        sqlx::query(
            r#"
        INSERT INTO public.event_atc_position_booking (
            event_atc_position_id, user_id, created_at, atc_booking_id
        )
        VALUES ($1, $2, $3, $4)
        "#,
        )
        .bind(position.id)
        .bind(user_id)
        .bind(now)
        .bind(atc_booking_id)
        .execute(&mut **self)
        .await?;

        Ok(())
    }

    async fn delete_event_atc_position_booking(
        &mut self,
        position_id: Uuid,
        atc_booking_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        tracing::info!(
            operation = "delete_booking",
            repository = "src/modules/event/repository/event_atc_position.rs",
            "modifying data"
        );

        sqlx::query(
            "DELETE FROM public.event_atc_position_booking WHERE event_atc_position_id = $1",
        )
        .bind(position_id)
        .execute(&mut **self)
        .await?;

        if let Some(atc_booking_id) = atc_booking_id {
            sqlx::query("DELETE FROM public.atc_booking WHERE id = $1")
                .bind(atc_booking_id)
                .execute(&mut **self)
                .await?;
        }

        Ok(())
    }

    async fn sync_event_atc_position_booking(
        &mut self,
        position: &EventAtcPosition,
    ) -> Result<(), sqlx::Error> {
        let Some(booking_id) = position.booking_id else {
            return Ok(());
        };
        sqlx::query(
            r#"
            UPDATE public.atc_booking
            SET callsign = $2,
                start_at = $3,
                end_at = $4,
                remarks = $5,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(booking_id)
        .bind(&position.callsign)
        .bind(position.start_at)
        .bind(position.end_at)
        .bind(&position.remarks)
        .execute(&mut **self)
        .await?;
        Ok(())
    }

    async fn delete_event_atc_booking(&mut self, booking_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM public.atc_booking WHERE id = $1")
            .bind(booking_id)
            .execute(&mut **self)
            .await?;
        Ok(())
    }
}
