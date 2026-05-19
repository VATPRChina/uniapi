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

pub async fn create(
    db: &PgPool,
    event_id: Uuid,
    airspace: EventAirspaceSave,
) -> Result<EventAirspaceRecord, sqlx::Error> {
    tracing::info!(
        operation = "create",
        repository = "src/repository/event/event_airspace.rs",
        "modifying data"
    );

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
