use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct EventAirspaceRecord {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub icao_codes: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct EventAirspaceSave {
    pub name: String,
    pub icao_codes: Vec<String>,
    pub description: String,
}

pub async fn list_by_event(
    db: &PgPool,
    event_id: Uuid,
) -> Result<Vec<EventAirspaceRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventAirspaceRecord>(
        r#"
        SELECT id, event_id, name, created_at, updated_at, icao_codes, description
        FROM public.event_airspace
        WHERE event_id = $1
        ORDER BY name
        "#,
    )
    .bind(event_id)
    .fetch_all(db)
    .await
}

pub async fn find_by_event_and_id(
    db: &PgPool,
    event_id: Uuid,
    airspace_id: Uuid,
) -> Result<Option<EventAirspaceRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventAirspaceRecord>(
        r#"
        SELECT id, event_id, name, created_at, updated_at, icao_codes, description
        FROM public.event_airspace
        WHERE event_id = $1 AND id = $2
        "#,
    )
    .bind(event_id)
    .bind(airspace_id)
    .fetch_optional(db)
    .await
}

pub async fn create(
    db: &PgPool,
    event_id: Uuid,
    airspace: EventAirspaceSave,
) -> Result<EventAirspaceRecord, sqlx::Error> {
    sqlx::query_as::<_, EventAirspaceRecord>(
        r#"
        INSERT INTO public.event_airspace (id, event_id, name, icao_codes, description)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, event_id, name, created_at, updated_at, icao_codes, description
        "#,
    )
    .bind(Uuid::from(Ulid::new()))
    .bind(event_id)
    .bind(airspace.name)
    .bind(airspace.icao_codes)
    .bind(airspace.description)
    .fetch_one(db)
    .await
}

pub async fn update(
    db: &PgPool,
    event_id: Uuid,
    airspace_id: Uuid,
    airspace: EventAirspaceSave,
) -> Result<Option<EventAirspaceRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventAirspaceRecord>(
        r#"
        UPDATE public.event_airspace
        SET name = $3,
            icao_codes = $4,
            description = $5,
            updated_at = CURRENT_TIMESTAMP
        WHERE event_id = $1 AND id = $2
        RETURNING id, event_id, name, created_at, updated_at, icao_codes, description
        "#,
    )
    .bind(event_id)
    .bind(airspace_id)
    .bind(airspace.name)
    .bind(airspace.icao_codes)
    .bind(airspace.description)
    .fetch_optional(db)
    .await
}

pub async fn delete(
    db: &PgPool,
    event_id: Uuid,
    airspace_id: Uuid,
) -> Result<Option<EventAirspaceRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventAirspaceRecord>(
        r#"
        DELETE FROM public.event_airspace
        WHERE event_id = $1 AND id = $2
        RETURNING id, event_id, name, created_at, updated_at, icao_codes, description
        "#,
    )
    .bind(event_id)
    .bind(airspace_id)
    .fetch_optional(db)
    .await
}
