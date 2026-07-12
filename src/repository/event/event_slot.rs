use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct EventSlotRecord {
    pub id: Uuid,
    pub event_id: Uuid,
    pub airspace_id: Uuid,
    pub airspace_name: String,
    pub airspace_created_at: DateTime<Utc>,
    pub airspace_updated_at: DateTime<Utc>,
    pub airspace_icao_codes: Vec<String>,
    pub airspace_description: String,
    pub enter_at: DateTime<Utc>,
    pub leave_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub callsign: Option<String>,
    pub aircraft_type_icao: Option<String>,
    pub booking_id: Option<Uuid>,
    pub booking_user_id: Option<Uuid>,
    pub booking_created_at: Option<DateTime<Utc>>,
    pub booking_updated_at: Option<DateTime<Utc>>,
    pub booking_user_cid: Option<String>,
    pub booking_user_created_at: Option<DateTime<Utc>>,
    pub booking_user_updated_at: Option<DateTime<Utc>>,
    pub booking_user_roles: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct EventSlotSave {
    pub airspace_id: Uuid,
    pub enter_at: DateTime<Utc>,
    pub leave_at: Option<DateTime<Utc>>,
    pub callsign: Option<String>,
    pub aircraft_type_icao: Option<String>,
}

fn slot_select_sql(where_clause: &str) -> String {
    slot_select_sql_from("public.event_slot", where_clause)
}

fn slot_select_sql_from(source: &str, where_clause: &str) -> String {
    format!(
        r#"
        SELECT event_slot.id,
               event_airspace.event_id,
               event_slot.event_airspace_id AS airspace_id,
               event_airspace.name AS airspace_name,
               event_airspace.created_at AS airspace_created_at,
               event_airspace.updated_at AS airspace_updated_at,
               event_airspace.icao_codes AS airspace_icao_codes,
               event_airspace.description AS airspace_description,
               event_slot.enter_at,
               event_slot.leave_at,
               event_slot.created_at,
               event_slot.updated_at,
               event_slot.callsign,
               event_slot.aircraft_type_icao,
               event_booking.id AS booking_id,
               event_booking.user_id AS booking_user_id,
               event_booking.created_at AS booking_created_at,
               event_booking.updated_at AS booking_updated_at,
               "user".cid AS booking_user_cid,
               "user".created_at AS booking_user_created_at,
               "user".updated_at AS booking_user_updated_at,
               "user".roles AS booking_user_roles
        FROM {source}
        JOIN public.event_airspace ON event_airspace.id = event_slot.event_airspace_id
        LEFT JOIN public.event_booking ON event_booking.event_slot_id = event_slot.id
        LEFT JOIN public."user" ON "user".id = event_booking.user_id
        {where_clause}
        "#
    )
}

pub trait EventSlotRepositoryExt<'executor> {
    async fn list_event_slot_by_event(
        self,
        event_id: Uuid,
    ) -> Result<Vec<EventSlotRecord>, sqlx::Error>;

    async fn create_event_slot(self, slot: EventSlotSave) -> Result<EventSlotRecord, sqlx::Error>;

    async fn booking_event_slot_export_rows(
        self,
        event_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error>;
}

impl<'executor, E> EventSlotRepositoryExt<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_event_slot_by_event(
        self,
        event_id: Uuid,
    ) -> Result<Vec<EventSlotRecord>, sqlx::Error> {
        sqlx::query_as::<_, EventSlotRecord>(&slot_select_sql(
            r#"
        WHERE event_airspace.event_id = $1
        ORDER BY event_slot.enter_at, event_slot.leave_at
        "#,
        ))
        .bind(event_id)
        .fetch_all(self)
        .await
    }
    async fn create_event_slot(self, slot: EventSlotSave) -> Result<EventSlotRecord, sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "src/repository/event/event_slot.rs",
            "modifying data"
        );

        let id = Uuid::from(Ulid::new());
        let query =
            r#"
        WITH inserted_slot AS (
            INSERT INTO public.event_slot (
                id, event_airspace_id, enter_at, leave_at, callsign, aircraft_type_icao
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
        )
        "#.to_string() + &slot_select_sql_from("inserted_slot AS event_slot", "WHERE event_slot.id = $1");
        sqlx::query_as::<_, EventSlotRecord>(&query)
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
