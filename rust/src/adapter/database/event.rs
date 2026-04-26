use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct EventRecord {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub title: String,
    pub title_en: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub start_booking_at: Option<DateTime<Utc>>,
    pub end_booking_at: Option<DateTime<Utc>>,
    pub start_atc_booking_at: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
    pub community_link: Option<String>,
    pub vatsim_link: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct EventSave {
    pub title: String,
    pub title_en: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub start_booking_at: Option<DateTime<Utc>>,
    pub end_booking_at: Option<DateTime<Utc>>,
    pub start_atc_booking_at: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
    pub community_link: Option<String>,
    pub vatsim_link: Option<String>,
    pub description: String,
}

pub async fn list_current(db: &PgPool) -> Result<Vec<EventRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventRecord>(
        r#"
        SELECT id, created_at, updated_at, title, title_en, start_at, end_at,
               start_booking_at, end_booking_at, start_atc_booking_at, image_url,
               community_link, vatsim_link, description
        FROM public.event
        WHERE (is_approved = TRUE OR is_approved IS NULL)
          AND now() < end_at
        ORDER BY start_at
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn list_past(
    db: &PgPool,
    until: Option<DateTime<Utc>>,
) -> Result<Vec<EventRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventRecord>(
        r#"
        SELECT id, created_at, updated_at, title, title_en, start_at, end_at,
               start_booking_at, end_booking_at, start_atc_booking_at, image_url,
               community_link, vatsim_link, description
        FROM public.event
        WHERE (is_approved = TRUE OR is_approved IS NULL)
          AND start_at < now()
          AND ($1::timestamptz IS NULL OR start_at <= $1)
        ORDER BY start_at DESC
        "#,
    )
    .bind(until)
    .fetch_all(db)
    .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<EventRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventRecord>(
        r#"
        SELECT id, created_at, updated_at, title, title_en, start_at, end_at,
               start_booking_at, end_booking_at, start_atc_booking_at, image_url,
               community_link, vatsim_link, description
        FROM public.event
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn create(db: &PgPool, event: EventSave) -> Result<EventRecord, sqlx::Error> {
    sqlx::query_as::<_, EventRecord>(
        r#"
        INSERT INTO public.event (
            id, title, title_en, start_at, end_at, start_booking_at, end_booking_at,
            start_atc_booking_at, image_url, community_link, vatsim_link, description
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, created_at, updated_at, title, title_en, start_at, end_at,
                  start_booking_at, end_booking_at, start_atc_booking_at, image_url,
                  community_link, vatsim_link, description
        "#,
    )
    .bind(Uuid::from(Ulid::new()))
    .bind(event.title)
    .bind(event.title_en)
    .bind(event.start_at)
    .bind(event.end_at)
    .bind(event.start_booking_at)
    .bind(event.end_booking_at)
    .bind(event.start_atc_booking_at)
    .bind(event.image_url)
    .bind(event.community_link)
    .bind(event.vatsim_link)
    .bind(event.description)
    .fetch_one(db)
    .await
}

pub async fn update(
    db: &PgPool,
    id: Uuid,
    event: EventSave,
) -> Result<Option<EventRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventRecord>(
        r#"
        UPDATE public.event
        SET title = $2,
            title_en = $3,
            start_at = $4,
            end_at = $5,
            start_booking_at = $6,
            end_booking_at = $7,
            start_atc_booking_at = $8,
            image_url = $9,
            community_link = $10,
            vatsim_link = $11,
            description = $12,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING id, created_at, updated_at, title, title_en, start_at, end_at,
                  start_booking_at, end_booking_at, start_atc_booking_at, image_url,
                  community_link, vatsim_link, description
        "#,
    )
    .bind(id)
    .bind(event.title)
    .bind(event.title_en)
    .bind(event.start_at)
    .bind(event.end_at)
    .bind(event.start_booking_at)
    .bind(event.end_booking_at)
    .bind(event.start_atc_booking_at)
    .bind(event.image_url)
    .bind(event.community_link)
    .bind(event.vatsim_link)
    .bind(event.description)
    .fetch_optional(db)
    .await
}

pub async fn delete(db: &PgPool, id: Uuid) -> Result<Option<EventRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventRecord>(
        r#"
        DELETE FROM public.event
        WHERE id = $1
        RETURNING id, created_at, updated_at, title, title_en, start_at, end_at,
                  start_booking_at, end_booking_at, start_atc_booking_at, image_url,
                  community_link, vatsim_link, description
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}
