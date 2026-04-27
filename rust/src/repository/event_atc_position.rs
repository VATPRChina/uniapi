use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct EventAtcPositionRecord {
    pub id: Uuid,
    pub event_id: Uuid,
    pub event_created_at: DateTime<Utc>,
    pub event_updated_at: DateTime<Utc>,
    pub event_title: String,
    pub event_title_en: Option<String>,
    pub event_start_at: DateTime<Utc>,
    pub event_end_at: DateTime<Utc>,
    pub event_start_booking_at: Option<DateTime<Utc>>,
    pub event_end_booking_at: Option<DateTime<Utc>>,
    pub event_start_atc_booking_at: Option<DateTime<Utc>>,
    pub event_image_url: Option<String>,
    pub event_community_link: Option<String>,
    pub event_vatsim_link: Option<String>,
    pub event_description: String,
    pub event_is_in_atc_booking_period: bool,
    pub callsign: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub remarks: Option<String>,
    pub position_kind_id: String,
    pub minimum_controller_state: i32,
    pub booking_user_id: Option<Uuid>,
    pub booking_created_at: Option<DateTime<Utc>>,
    pub booking_user_cid: Option<String>,
    pub booking_user_full_name: Option<String>,
    pub booking_user_created_at: Option<DateTime<Utc>>,
    pub booking_user_updated_at: Option<DateTime<Utc>>,
    pub booking_user_roles: Option<Vec<String>>,
    pub atc_booking_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct EventAtcPositionSave {
    pub callsign: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub remarks: Option<String>,
    pub position_kind_id: String,
    pub minimum_controller_state: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserAtcPermissionRecord {
    pub state: String,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

pub async fn list_by_event(
    db: &PgPool,
    event_id: Uuid,
) -> Result<Vec<EventAtcPositionRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventAtcPositionRecord>(&position_select_sql(
        r#"
        WHERE event_atc_position.event_id = $1
        "#,
    ))
    .bind(event_id)
    .fetch_all(db)
    .await
}

pub async fn find_by_event_and_id(
    db: &PgPool,
    event_id: Uuid,
    position_id: Uuid,
) -> Result<Option<EventAtcPositionRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventAtcPositionRecord>(&position_select_sql(
        r#"
        WHERE event_atc_position.event_id = $1 AND event_atc_position.id = $2
        "#,
    ))
    .bind(event_id)
    .bind(position_id)
    .fetch_optional(db)
    .await
}

pub async fn create(
    db: &PgPool,
    event_id: Uuid,
    position: EventAtcPositionSave,
) -> Result<EventAtcPositionRecord, sqlx::Error> {
    let id = Uuid::from(Ulid::new());
    sqlx::query(
        r#"
        INSERT INTO public.event_atc_position (
            id, event_id, callsign, start_at, end_at, remarks,
            position_kind_id, minimum_controller_state
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(id)
    .bind(event_id)
    .bind(position.callsign)
    .bind(position.start_at)
    .bind(position.end_at)
    .bind(position.remarks)
    .bind(position.position_kind_id)
    .bind(position.minimum_controller_state)
    .execute(db)
    .await?;

    find_by_event_and_id(db, event_id, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn update(
    db: &PgPool,
    event_id: Uuid,
    position_id: Uuid,
    position: EventAtcPositionSave,
) -> Result<Option<EventAtcPositionRecord>, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE public.event_atc_position
        SET callsign = $3,
            start_at = $4,
            end_at = $5,
            remarks = $6,
            position_kind_id = $7,
            minimum_controller_state = $8
        WHERE event_id = $1 AND id = $2
        "#,
    )
    .bind(event_id)
    .bind(position_id)
    .bind(position.callsign)
    .bind(position.start_at)
    .bind(position.end_at)
    .bind(position.remarks)
    .bind(position.position_kind_id)
    .bind(position.minimum_controller_state)
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    find_by_event_and_id(db, event_id, position_id).await
}

pub async fn delete(db: &PgPool, event_id: Uuid, position_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM public.event_atc_position
        WHERE event_id = $1 AND id = $2
        "#,
    )
    .bind(event_id)
    .bind(position_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn user_permission(
    db: &PgPool,
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
    .fetch_optional(db)
    .await
}

pub async fn create_booking(
    transaction: &mut Transaction<'_, Postgres>,
    position: &EventAtcPositionRecord,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let atc_booking_id = Uuid::from(Ulid::new());
    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO public.atc_booking (id, user_id, callsign, booked_at, start_at, end_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(atc_booking_id)
    .bind(user_id)
    .bind(&position.callsign)
    .bind(now)
    .bind(position.start_at)
    .bind(position.end_at)
    .execute(&mut **transaction)
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
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub async fn delete_booking(
    transaction: &mut Transaction<'_, Postgres>,
    position_id: Uuid,
    atc_booking_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM public.event_atc_position_booking WHERE event_atc_position_id = $1")
        .bind(position_id)
        .execute(&mut **transaction)
        .await?;

    if let Some(atc_booking_id) = atc_booking_id {
        sqlx::query("DELETE FROM public.atc_booking WHERE id = $1")
            .bind(atc_booking_id)
            .execute(&mut **transaction)
            .await?;
    }

    Ok(())
}

fn position_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT event_atc_position.id,
               event.id AS event_id,
               event.created_at AS event_created_at,
               event.updated_at AS event_updated_at,
               event.title AS event_title,
               event.title_en AS event_title_en,
               event.start_at AS event_start_at,
               event.end_at AS event_end_at,
               event.start_booking_at AS event_start_booking_at,
               event.end_booking_at AS event_end_booking_at,
               event.start_atc_booking_at AS event_start_atc_booking_at,
               event.image_url AS event_image_url,
               event.community_link AS event_community_link,
               event.vatsim_link AS event_vatsim_link,
               event.description AS event_description,
               (
                   event.start_atc_booking_at IS NULL
                   OR now() > event.start_atc_booking_at
               ) AS event_is_in_atc_booking_period,
               event_atc_position.callsign,
               event_atc_position.start_at,
               event_atc_position.end_at,
               event_atc_position.remarks,
               event_atc_position.position_kind_id,
               event_atc_position.minimum_controller_state,
               event_atc_position_booking.user_id AS booking_user_id,
               event_atc_position_booking.created_at AS booking_created_at,
               "user".cid AS booking_user_cid,
               "user".full_name AS booking_user_full_name,
               "user".created_at AS booking_user_created_at,
               "user".updated_at AS booking_user_updated_at,
               "user".roles AS booking_user_roles,
               event_atc_position_booking.atc_booking_id
        FROM public.event_atc_position
        JOIN public.event ON event.id = event_atc_position.event_id
        LEFT JOIN public.event_atc_position_booking
            ON event_atc_position_booking.event_atc_position_id = event_atc_position.id
        LEFT JOIN public."user" ON "user".id = event_atc_position_booking.user_id
        {where_clause}
        "#
    )
}
