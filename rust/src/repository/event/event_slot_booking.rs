use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct EventBookingRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_cid: Option<String>,
    pub user_created_at: Option<DateTime<Utc>>,
    pub user_updated_at: Option<DateTime<Utc>>,
    pub user_roles: Option<Vec<String>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SlotBookingState {
    pub event_exists: bool,
    pub slot_exists: bool,
    pub booking_id: Option<Uuid>,
    pub booking_user_id: Option<Uuid>,
    pub is_in_booking_period: bool,
}

pub async fn find_booking(
    db: &PgPool,
    event_id: Uuid,
    slot_id: Uuid,
) -> Result<Option<EventBookingRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventBookingRecord>(
        r#"
        SELECT event_booking.id,
               event_booking.user_id,
               event_booking.created_at,
               event_booking.updated_at,
               "user".cid AS user_cid,
               "user".created_at AS user_created_at,
               "user".updated_at AS user_updated_at,
               "user".roles AS user_roles
        FROM public.event_booking
        JOIN public.event_slot ON event_slot.id = event_booking.event_slot_id
        JOIN public.event_airspace ON event_airspace.id = event_slot.event_airspace_id
        LEFT JOIN public."user" ON "user".id = event_booking.user_id
        WHERE event_airspace.event_id = $1 AND event_slot.id = $2
        "#,
    )
    .bind(event_id)
    .bind(slot_id)
    .fetch_optional(db)
    .await
}

pub async fn load_state_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    event_id: Uuid,
    slot_id: Uuid,
) -> Result<SlotBookingState, sqlx::Error> {
    sqlx::query_as::<_, SlotBookingState>(
        r#"
        SELECT EXISTS(SELECT 1 FROM public.event WHERE id = $1) AS event_exists,
               event_slot.id IS NOT NULL AS slot_exists,
               event_booking.id AS booking_id,
               event_booking.user_id AS booking_user_id,
               (
                   event.start_booking_at IS NOT NULL
                   AND event.end_booking_at IS NOT NULL
                   AND now() > event.start_booking_at
                   AND now() < event.end_booking_at
               ) AS is_in_booking_period
        FROM (SELECT $1::uuid AS event_id, $2::uuid AS slot_id) input
        LEFT JOIN public.event ON event.id = input.event_id
        LEFT JOIN public.event_airspace ON event_airspace.event_id = event.id
        LEFT JOIN public.event_slot ON event_slot.event_airspace_id = event_airspace.id
            AND event_slot.id = input.slot_id
        LEFT JOIN public.event_booking ON event_booking.event_slot_id = event_slot.id
        FOR UPDATE OF event_slot, event_booking
        "#,
    )
    .bind(event_id)
    .bind(slot_id)
    .fetch_one(&mut **transaction)
    .await
}

pub async fn create_booking(
    transaction: &mut Transaction<'_, Postgres>,
    slot_id: Uuid,
    user_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::from(Ulid::new());
    sqlx::query(
        r#"
        INSERT INTO public.event_booking (id, user_id, event_slot_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(slot_id)
    .execute(&mut **transaction)
    .await?;

    Ok(id)
}

pub async fn delete_booking(
    transaction: &mut Transaction<'_, Postgres>,
    booking_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM public.event_booking WHERE id = $1")
        .bind(booking_id)
        .execute(&mut **transaction)
        .await?;

    Ok(())
}
