use ulid::Ulid;
use uuid::Uuid;

use crate::modules::event::models::{EventSlot, EventSlotSave};

fn slot_select_sql(where_clause: &str) -> String {
    slot_select_sql_from("public.event_slot", where_clause)
}

fn slot_select_sql_from(source: &str, where_clause: &str) -> String {
    format!(
        r#"
        SELECT event_slot.id,
               event_slot.event_airspace_id AS airspace_id,
               event_slot.enter_at,
               event_slot.leave_at,
               event_slot.created_at,
               event_slot.updated_at,
               event_slot.callsign,
               event_slot.aircraft_type_icao,
               event_booking.id AS booking_id
        FROM {source}
        JOIN public.event_airspace ON event_airspace.id = event_slot.event_airspace_id
        LEFT JOIN public.event_booking ON event_booking.event_slot_id = event_slot.id
        {where_clause}
        "#
    )
}

pub(crate) trait EventSlotRepository<'executor> {
    async fn list_event_slot_by_event(self, event_id: Uuid) -> Result<Vec<EventSlot>, sqlx::Error>;

    async fn create_event_slot(self, slot: EventSlotSave) -> Result<EventSlot, sqlx::Error>;

    async fn booking_event_slot_export_rows(
        self,
        event_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error>;
}

impl<'executor, E> EventSlotRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_event_slot_by_event(self, event_id: Uuid) -> Result<Vec<EventSlot>, sqlx::Error> {
        sqlx::query_as::<_, EventSlot>(&slot_select_sql(
            r#"
        WHERE event_airspace.event_id = $1
        ORDER BY event_slot.enter_at, event_slot.leave_at
        "#,
        ))
        .bind(event_id)
        .fetch_all(self)
        .await
    }
    async fn create_event_slot(self, slot: EventSlotSave) -> Result<EventSlot, sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "src/modules/event/repository/event_slot.rs",
            "modifying data"
        );

        let id = Uuid::from(Ulid::new());
        let query = r#"
        WITH inserted_slot AS (
            INSERT INTO public.event_slot (
                id, event_airspace_id, enter_at, leave_at, callsign, aircraft_type_icao
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
        )
        "#
        .to_string()
            + &slot_select_sql_from("inserted_slot AS event_slot", "WHERE event_slot.id = $1");
        sqlx::query_as::<_, EventSlot>(&query)
            .bind(id)
            .bind(slot.airspace_id)
            .bind(slot.enter_at)
            .bind(slot.leave_at)
            .bind(slot.callsign)
            .bind(slot.aircraft_type_icao)
            .fetch_one(self)
            .await
    }
    async fn booking_event_slot_export_rows(
        self,
        event_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar::<_, String>(
            r#"
        SELECT "user".cid || ',' || to_char(event_slot.enter_at AT TIME ZONE 'UTC', 'HH24MI')
        FROM public.event_booking
        JOIN public."user" ON "user".id = event_booking.user_id
        JOIN public.event_slot ON event_slot.id = event_booking.event_slot_id
        JOIN public.event_airspace ON event_airspace.id = event_slot.event_airspace_id
        WHERE event_airspace.event_id = $1
        ORDER BY event_slot.enter_at
        "#,
        )
        .bind(event_id)
        .fetch_all(self)
        .await
    }
}
