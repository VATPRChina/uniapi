use ulid::Ulid;
use uuid::Uuid;

use crate::modules::event::models::{EventAirspace, EventAirspaceSave};

pub(crate) trait EventAirspaceRepository<'executor> {
    async fn find_event_airspace_by_id(
        self,
        airspace_id: Uuid,
    ) -> Result<Option<EventAirspace>, sqlx::Error>;

    async fn create_event_airspace(
        self,
        event_id: Uuid,
        airspace: EventAirspaceSave,
    ) -> Result<EventAirspace, sqlx::Error>;
}

impl<'executor, E> EventAirspaceRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn find_event_airspace_by_id(
        self,
        airspace_id: Uuid,
    ) -> Result<Option<EventAirspace>, sqlx::Error> {
        sqlx::query_as::<_, EventAirspace>(
            r#"
        SELECT id, event_id, name, created_at, updated_at, icao_codes, description
        FROM public.event_airspace
        WHERE id = $1
        "#,
        )
        .bind(airspace_id)
        .fetch_optional(self)
        .await
    }

    async fn create_event_airspace(
        self,
        event_id: Uuid,
        airspace: EventAirspaceSave,
    ) -> Result<EventAirspace, sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "src/modules/event/repository/event_airspace.rs",
            "modifying data"
        );

        sqlx::query_as::<_, EventAirspace>(
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
        .fetch_one(self)
        .await
    }
}
