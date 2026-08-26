use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::event::repository::event::EventRepository;
use crate::modules::event::repository::event_atc_position::EventAtcPositionRepository;
use crate::modules::event::service::EventAtcPositionView;
use crate::modules::user::models::UserSummary;
use crate::modules::user::service::user::{UserService, UserServiceError};

use super::models::{AtcBooking, AtcBookingSave};
use super::repository::AtcBookingRepository;

#[derive(Clone)]
pub struct AtcBookingService {
    db: PgPool,
    user: UserService,
}

impl AtcBookingService {
    pub fn new(db: PgPool, user: UserService) -> Self {
        Self { db, user }
    }

    pub async fn list_upcoming(&self) -> Result<Vec<AtcBookingView>, AtcBookingServiceError> {
        let bookings = self.db.list_upcoming_atc_bookings().await?;
        self.with_details(bookings).await
    }

    pub async fn list_mine_upcoming(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<AtcBookingView>, AtcBookingServiceError> {
        let bookings = self.db.list_upcoming_atc_bookings_by_user(user_id).await?;
        self.with_details(bookings).await
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        booking: AtcBookingSave,
    ) -> Result<AtcBookingView, AtcBookingServiceError> {
        let booking = self.db.create_atc_booking(user_id, booking).await?;
        self.with_detail(booking).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        booking: AtcBookingSave,
    ) -> Result<AtcBookingView, AtcBookingServiceError> {
        let mut transaction = self.db.begin().await?;
        let current = (&mut *transaction)
            .find_atc_booking_for_update(id)
            .await?
            .ok_or(AtcBookingServiceError::NotFound(id))?;
        ensure_editable(&current, user_id)?;
        let booking = (&mut *transaction)
            .update_atc_booking(id, booking)
            .await?
            .ok_or(AtcBookingServiceError::NotFound(id))?;
        transaction.commit().await?;
        self.with_detail(booking).await
    }

    pub async fn delete(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<AtcBookingView, AtcBookingServiceError> {
        let mut transaction = self.db.begin().await?;
        let current = (&mut *transaction)
            .find_atc_booking_for_update(id)
            .await?
            .ok_or(AtcBookingServiceError::NotFound(id))?;
        ensure_editable(&current, user_id)?;
        let booking = (&mut *transaction)
            .delete_atc_booking(id)
            .await?
            .ok_or(AtcBookingServiceError::NotFound(id))?;
        transaction.commit().await?;
        self.with_detail(booking).await
    }

    async fn with_details(
        &self,
        bookings: Vec<AtcBooking>,
    ) -> Result<Vec<AtcBookingView>, AtcBookingServiceError> {
        let mut users = self
            .user
            .get_users_bulk(bookings.iter().map(|booking| booking.user_id))
            .await?;
        let mut views = Vec::with_capacity(bookings.len());
        for booking in bookings {
            let user = users
                .remove(&booking.user_id)
                .ok_or(AtcBookingServiceError::UserNotFound(booking.user_id))?;
            let event_position = self.event_position(&booking).await?;
            views.push(AtcBookingView {
                booking,
                user,
                event_position,
            });
        }
        Ok(views)
    }

    async fn with_detail(
        &self,
        booking: AtcBooking,
    ) -> Result<AtcBookingView, AtcBookingServiceError> {
        let user = self
            .user
            .find_summary_by_id(booking.user_id)
            .await?
            .ok_or(AtcBookingServiceError::UserNotFound(booking.user_id))?;
        let event_position = self.event_position(&booking).await?;
        Ok(AtcBookingView {
            booking,
            user,
            event_position,
        })
    }

    async fn event_position(
        &self,
        booking: &AtcBooking,
    ) -> Result<Option<EventAtcPositionView>, AtcBookingServiceError> {
        let Some(position_id) = booking.event_position_id else {
            return Ok(None);
        };
        let Some(position) = self.db.find_event_atc_position_by_id(position_id).await? else {
            return Ok(None);
        };
        let Some(event) = self.db.find_event_by_id(position.event_id).await? else {
            return Ok(None);
        };
        Ok(Some(EventAtcPositionView {
            position,
            event,
            booking: None,
        }))
    }
}

fn ensure_editable(booking: &AtcBooking, user_id: Uuid) -> Result<(), AtcBookingServiceError> {
    if booking.user_id != user_id {
        return Err(AtcBookingServiceError::NotOwned(booking.id));
    }
    if booking.event_position_id.is_some() {
        return Err(AtcBookingServiceError::EventLinked);
    }
    Ok(())
}

pub struct AtcBookingView {
    pub booking: AtcBooking,
    pub user: UserSummary,
    pub event_position: Option<EventAtcPositionView>,
}

#[derive(Debug, thiserror::Error)]
pub enum AtcBookingServiceError {
    #[error("ATC booking {0} not found")]
    NotFound(Uuid),
    #[error("ATC booking {0} is not owned by current user")]
    NotOwned(Uuid),
    #[error("event-linked ATC bookings can only be changed through the event API")]
    EventLinked,
    #[error("user {0} not found")]
    UserNotFound(Uuid),
    #[error("failed to query ATC booking repository: {0}")]
    Database(#[from] sqlx::Error),
    #[error("failed to query user: {0}")]
    User(#[from] UserServiceError),
}
