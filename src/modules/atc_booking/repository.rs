use chrono::Utc;
use ulid::Ulid;
use uuid::Uuid;

use super::models::{AtcBooking, AtcBookingSave};

const SELECT: &str = r#"
    SELECT atc_booking.id,
           atc_booking.user_id,
           atc_booking.callsign,
           atc_booking.start_at,
           atc_booking.end_at,
           atc_booking.remarks,
           atc_booking.created_at,
           atc_booking.updated_at,
           atc_booking.deleted_at,
           event_atc_position_booking.event_atc_position_id AS event_position_id
    FROM public.atc_booking
    LEFT JOIN public.event_atc_position_booking
      ON event_atc_position_booking.atc_booking_id = atc_booking.id
"#;

pub(crate) trait AtcBookingRepository<'executor> {
    async fn list_upcoming_atc_bookings(self) -> Result<Vec<AtcBooking>, sqlx::Error>;

    async fn list_upcoming_atc_bookings_by_user(
        self,
        user_id: Uuid,
    ) -> Result<Vec<AtcBooking>, sqlx::Error>;

    async fn find_atc_booking_for_update(self, id: Uuid)
    -> Result<Option<AtcBooking>, sqlx::Error>;

    async fn create_atc_booking(
        self,
        user_id: Uuid,
        booking: AtcBookingSave,
    ) -> Result<AtcBooking, sqlx::Error>;

    async fn update_atc_booking(
        self,
        id: Uuid,
        booking: AtcBookingSave,
    ) -> Result<Option<AtcBooking>, sqlx::Error>;

    async fn delete_atc_booking(self, id: Uuid) -> Result<Option<AtcBooking>, sqlx::Error>;
}

impl<'executor, E> AtcBookingRepository<'executor> for E
where
    E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
{
    async fn list_upcoming_atc_bookings(self) -> Result<Vec<AtcBooking>, sqlx::Error> {
        sqlx::query_as::<_, AtcBooking>(&format!(
            "{SELECT} WHERE atc_booking.deleted_at IS NULL AND atc_booking.end_at > now() ORDER BY atc_booking.start_at, atc_booking.callsign"
        ))
        .fetch_all(self)
        .await
    }

    async fn list_upcoming_atc_bookings_by_user(
        self,
        user_id: Uuid,
    ) -> Result<Vec<AtcBooking>, sqlx::Error> {
        sqlx::query_as::<_, AtcBooking>(&format!(
            "{SELECT} WHERE atc_booking.user_id = $1 AND atc_booking.deleted_at IS NULL AND atc_booking.end_at > now() ORDER BY atc_booking.start_at, atc_booking.callsign"
        ))
        .bind(user_id)
        .fetch_all(self)
        .await
    }

    async fn find_atc_booking_for_update(
        self,
        id: Uuid,
    ) -> Result<Option<AtcBooking>, sqlx::Error> {
        sqlx::query_as::<_, AtcBooking>(&format!(
            "{SELECT} WHERE atc_booking.id = $1 AND atc_booking.deleted_at IS NULL FOR UPDATE OF atc_booking"
        ))
        .bind(id)
        .fetch_optional(self)
        .await
    }

    async fn create_atc_booking(
        self,
        user_id: Uuid,
        booking: AtcBookingSave,
    ) -> Result<AtcBooking, sqlx::Error> {
        tracing::info!(
            operation = "create",
            repository = "atc_booking",
            "modifying data"
        );
        let id = Uuid::from(Ulid::new());
        let now = Utc::now();
        sqlx::query_as::<_, AtcBooking>(
            r#"
            WITH inserted AS (
                INSERT INTO public.atc_booking (
                    id, user_id, callsign, booked_at, start_at, end_at, remarks,
                    created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $4, $4)
                RETURNING *
            )
            SELECT inserted.id, inserted.user_id, inserted.callsign, inserted.start_at,
                   inserted.end_at, inserted.remarks, inserted.created_at, inserted.updated_at,
                   inserted.deleted_at, NULL::uuid AS event_position_id
            FROM inserted
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(booking.callsign)
        .bind(now)
        .bind(booking.start_at)
        .bind(booking.end_at)
        .bind(booking.remarks)
        .fetch_one(self)
        .await
    }

    async fn update_atc_booking(
        self,
        id: Uuid,
        booking: AtcBookingSave,
    ) -> Result<Option<AtcBooking>, sqlx::Error> {
        tracing::info!(
            operation = "update",
            repository = "atc_booking",
            "modifying data"
        );
        sqlx::query_as::<_, AtcBooking>(
            r#"
            WITH updated AS (
                UPDATE public.atc_booking
                SET callsign = $2,
                    start_at = $3,
                    end_at = $4,
                    remarks = $5,
                    updated_at = now()
                WHERE id = $1 AND deleted_at IS NULL
                RETURNING *
            )
            SELECT updated.id, updated.user_id, updated.callsign, updated.start_at,
                   updated.end_at, updated.remarks, updated.created_at, updated.updated_at,
                   updated.deleted_at, NULL::uuid AS event_position_id
            FROM updated
            "#,
        )
        .bind(id)
        .bind(booking.callsign)
        .bind(booking.start_at)
        .bind(booking.end_at)
        .bind(booking.remarks)
        .fetch_optional(self)
        .await
    }

    async fn delete_atc_booking(self, id: Uuid) -> Result<Option<AtcBooking>, sqlx::Error> {
        tracing::info!(
            operation = "delete",
            repository = "atc_booking",
            "modifying data"
        );
        sqlx::query_as::<_, AtcBooking>(
            r#"
            WITH deleted AS (
                UPDATE public.atc_booking
                SET deleted_at = now(), updated_at = now()
                WHERE id = $1 AND deleted_at IS NULL
                RETURNING *
            )
            SELECT deleted.id, deleted.user_id, deleted.callsign, deleted.start_at,
                   deleted.end_at, deleted.remarks, deleted.created_at, deleted.updated_at,
                   deleted.deleted_at, NULL::uuid AS event_position_id
            FROM deleted
            "#,
        )
        .bind(id)
        .fetch_optional(self)
        .await
    }
}
