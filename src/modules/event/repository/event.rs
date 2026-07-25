use chrono::{DateTime, Utc};
use ulid::Ulid;
use uuid::Uuid;

use crate::modules::event::models::{Event, EventSave};

pub(crate) trait EventRepository<'executor> {
    async fn list_event_current(self) -> Result<Vec<Event>, sqlx::Error>;

    async fn list_event_past(self, until: Option<DateTime<Utc>>)
    -> Result<Vec<Event>, sqlx::Error>;

    async fn find_event_by_id(self, id: Uuid) -> Result<Option<Event>, sqlx::Error>;

    async fn exists_event(self, id: Uuid) -> Result<bool, sqlx::Error>;

    async fn create_event(self, event: EventSave) -> Result<Event, sqlx::Error>;

    async fn find_event_by_id_for_update(self, id: Uuid) -> Result<Option<Event>, sqlx::Error>;

    async fn update_event(self, id: Uuid, event: EventSave) -> Result<Option<Event>, sqlx::Error>;
}

impl<'executor, E> EventRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_event_current(self) -> Result<Vec<Event>, sqlx::Error> {
        sqlx::query_as::<_, Event>(
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
        .fetch_all(self)
        .await
    }
    async fn list_event_past(
        self,
        until: Option<DateTime<Utc>>,
    ) -> Result<Vec<Event>, sqlx::Error> {
        sqlx::query_as::<_, Event>(
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
        .fetch_all(self)
        .await
    }
    async fn find_event_by_id(self, id: Uuid) -> Result<Option<Event>, sqlx::Error> {
        sqlx::query_as::<_, Event>(
            r#"
        SELECT id, created_at, updated_at, title, title_en, start_at, end_at,
               start_booking_at, end_booking_at, start_atc_booking_at, image_url,
               community_link, vatsim_link, description
        FROM public.event
        WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_optional(self)
        .await
    }
    async fn exists_event(self, id: Uuid) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar::<_, bool>(
            r#"
        SELECT EXISTS (
            SELECT 1
            FROM public.event
            WHERE id = $1
        )
        "#,
        )
        .bind(id)
        .fetch_one(self)
        .await
    }
    async fn create_event(self, event: EventSave) -> Result<Event, sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "src/modules/event/repository/event.rs",
            "modifying data"
        );

        sqlx::query_as::<_, Event>(
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
        .fetch_one(self)
        .await
    }
    async fn find_event_by_id_for_update(self, id: Uuid) -> Result<Option<Event>, sqlx::Error> {
        sqlx::query_as::<_, Event>(
            r#"
        SELECT id, created_at, updated_at, title, title_en, start_at, end_at,
               start_booking_at, end_booking_at, start_atc_booking_at, image_url,
               community_link, vatsim_link, description
        FROM public.event
        WHERE id = $1
        FOR UPDATE
        "#,
        )
        .bind(id)
        .fetch_optional(self)
        .await
    }
    async fn update_event(self, id: Uuid, event: EventSave) -> Result<Option<Event>, sqlx::Error> {
        tracing::info!(
            operation = "update",
            repository = "src/modules/event/repository/event.rs",
            "modifying data"
        );

        sqlx::query_as::<_, Event>(
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
        .fetch_optional(self)
        .await
    }
}
