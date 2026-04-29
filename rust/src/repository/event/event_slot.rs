use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
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

pub async fn list_by_event(
    db: &PgPool,
    event_id: Uuid,
) -> Result<Vec<EventSlotRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventSlotRecord>(&slot_select_sql(
        r#"
        WHERE event_airspace.event_id = $1
        ORDER BY event_slot.enter_at, event_slot.leave_at
        "#,
    ))
    .bind(event_id)
    .fetch_all(db)
    .await
}

pub async fn find_by_event_and_id(
    db: &PgPool,
    event_id: Uuid,
    slot_id: Uuid,
) -> Result<Option<EventSlotRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventSlotRecord>(&slot_select_sql(
        r#"
        WHERE event_airspace.event_id = $1 AND event_slot.id = $2
        "#,
    ))
    .bind(event_id)
    .bind(slot_id)
    .fetch_optional(db)
    .await
}

pub async fn find_mine_by_event(
    db: &PgPool,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<Option<EventSlotRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventSlotRecord>(&slot_select_sql(
        r#"
        WHERE event_airspace.event_id = $1 AND event_booking.user_id = $2
        "#,
    ))
    .bind(event_id)
    .bind(user_id)
    .fetch_optional(db)
    .await
}

pub async fn create(db: &PgPool, slot: EventSlotSave) -> Result<EventSlotRecord, sqlx::Error> {
    let id = Uuid::from(Ulid::new());
    sqlx::query(
        r#"
        INSERT INTO public.event_slot (
            id, event_airspace_id, enter_at, leave_at, callsign, aircraft_type_icao
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(id)
    .bind(slot.airspace_id)
    .bind(slot.enter_at)
    .bind(slot.leave_at)
    .bind(slot.callsign)
    .bind(slot.aircraft_type_icao)
    .execute(db)
    .await?;

    find_by_id(db, id).await?.ok_or(sqlx::Error::RowNotFound)
}

pub async fn update(
    db: &PgPool,
    event_id: Uuid,
    slot_id: Uuid,
    slot: EventSlotSave,
) -> Result<Option<EventSlotRecord>, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE public.event_slot
        SET enter_at = $3,
            leave_at = $4,
            callsign = $5,
            aircraft_type_icao = $6,
            updated_at = CURRENT_TIMESTAMP
        FROM public.event_airspace
        WHERE event_slot.event_airspace_id = event_airspace.id
          AND event_airspace.event_id = $1
          AND event_slot.id = $2
        "#,
    )
    .bind(event_id)
    .bind(slot_id)
    .bind(slot.enter_at)
    .bind(slot.leave_at)
    .bind(slot.callsign)
    .bind(slot.aircraft_type_icao)
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    find_by_event_and_id(db, event_id, slot_id).await
}

pub async fn delete(
    db: &PgPool,
    event_id: Uuid,
    slot_id: Uuid,
) -> Result<Option<EventSlotRecord>, sqlx::Error> {
    let slot = find_by_event_and_id(db, event_id, slot_id).await?;
    if slot.is_none() {
        return Ok(None);
    }

    sqlx::query("DELETE FROM public.event_slot WHERE id = $1")
        .bind(slot_id)
        .execute(db)
        .await?;

    Ok(slot)
}

pub async fn booking_export_rows(db: &PgPool, event_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT "user".cid || ',' || to_char(event_slot.enter_at, 'HH24MI')
        FROM public.event_booking
        JOIN public."user" ON "user".id = event_booking.user_id
        JOIN public.event_slot ON event_slot.id = event_booking.event_slot_id
        JOIN public.event_airspace ON event_airspace.id = event_slot.event_airspace_id
        WHERE event_airspace.event_id = $1
        ORDER BY event_slot.enter_at
        "#,
    )
    .bind(event_id)
    .fetch_all(db)
    .await
}

async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<EventSlotRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventSlotRecord>(&slot_select_sql("WHERE event_slot.id = $1"))
        .bind(id)
        .fetch_optional(db)
        .await
}

fn slot_select_sql(where_clause: &str) -> String {
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
        FROM public.event_slot
        JOIN public.event_airspace ON event_airspace.id = event_slot.event_airspace_id
        LEFT JOIN public.event_booking ON event_booking.event_slot_id = event_slot.id
        LEFT JOIN public."user" ON "user".id = event_booking.user_id
        {where_clause}
        "#
    )
}
