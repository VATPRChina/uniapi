use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AtcBookingRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_cid: String,
    pub user_created_at: DateTime<Utc>,
    pub user_updated_at: DateTime<Utc>,
    pub user_roles: Vec<String>,
    pub callsign: String,
    pub booked_at: DateTime<Utc>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AtcBookingSave {
    pub callsign: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

pub async fn list(db: &PgPool) -> Result<Vec<AtcBookingRecord>, sqlx::Error> {
    sqlx::query_as::<_, AtcBookingRecord>(&booking_select_sql(
        r#"
        ORDER BY atc_booking.start_at DESC
        "#,
    ))
    .fetch_all(db)
    .await
}

pub async fn list_by_user(
    db: &PgPool,
    user_id: Uuid,
) -> Result<Vec<AtcBookingRecord>, sqlx::Error> {
    sqlx::query_as::<_, AtcBookingRecord>(&booking_select_sql(
        r#"
        WHERE atc_booking.user_id = $1
        ORDER BY atc_booking.start_at DESC
        "#,
    ))
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn find_by_id(
    db: &PgPool,
    booking_id: Uuid,
) -> Result<Option<AtcBookingRecord>, sqlx::Error> {
    sqlx::query_as::<_, AtcBookingRecord>(&booking_select_sql(
        r#"
        WHERE atc_booking.id = $1
        "#,
    ))
    .bind(booking_id)
    .fetch_optional(db)
    .await
}

pub async fn create(
    db: &PgPool,
    user_id: Uuid,
    booking: AtcBookingSave,
) -> Result<AtcBookingRecord, sqlx::Error> {
    let id = Uuid::from(Ulid::new());
    sqlx::query(
        r#"
        INSERT INTO public.atc_booking (id, user_id, callsign, booked_at, start_at, end_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(booking.callsign)
    .bind(Utc::now())
    .bind(booking.start_at)
    .bind(booking.end_at)
    .execute(db)
    .await?;

    find_by_id(db, id).await?.ok_or(sqlx::Error::RowNotFound)
}

pub async fn update(
    db: &PgPool,
    booking_id: Uuid,
    booking: AtcBookingSave,
) -> Result<Option<AtcBookingRecord>, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE public.atc_booking
        SET callsign = $2,
            booked_at = $3,
            start_at = $4,
            end_at = $5
        WHERE id = $1
        "#,
    )
    .bind(booking_id)
    .bind(booking.callsign)
    .bind(Utc::now())
    .bind(booking.start_at)
    .bind(booking.end_at)
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    find_by_id(db, booking_id).await
}

pub async fn delete(
    db: &PgPool,
    booking_id: Uuid,
) -> Result<Option<AtcBookingRecord>, sqlx::Error> {
    let booking = find_by_id(db, booking_id).await?;
    if booking.is_none() {
        return Ok(None);
    }

    sqlx::query(
        r#"
        DELETE FROM public.atc_booking
        WHERE id = $1
        "#,
    )
    .bind(booking_id)
    .execute(db)
    .await?;

    Ok(booking)
}

pub async fn has_event_position_booking(
    db: &PgPool,
    booking_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM public.event_atc_position_booking
            WHERE atc_booking_id = $1
        )
        "#,
    )
    .bind(booking_id)
    .fetch_one(db)
    .await
}

fn booking_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT atc_booking.id,
               atc_booking.user_id,
               "user".cid AS user_cid,
               "user".created_at AS user_created_at,
               "user".updated_at AS user_updated_at,
               "user".roles AS user_roles,
               atc_booking.callsign,
               atc_booking.booked_at,
               atc_booking.start_at,
               atc_booking.end_at
        FROM public.atc_booking
        INNER JOIN public."user" ON "user".id = atc_booking.user_id
        {where_clause}
        "#
    )
}
