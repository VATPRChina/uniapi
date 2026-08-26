use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::audit_log::models::AuditLogEntity;
use crate::modules::audit_log::service::{AuditLogService, AuditLogServiceError};
use crate::modules::user::models::UserSummary;
use crate::modules::user::service::user::{UserService, UserServiceError};

use super::models::{
    Event, EventAirspace, EventAirspaceSave, EventAtcBooking, EventAtcPosition,
    EventAtcPositionSave, EventBooking, EventSave, EventSlot, EventSlotSave,
};
use super::repository::event::EventRepository;
use super::repository::event_airspace::EventAirspaceRepository;
use super::repository::event_atc_position::{
    EventAtcPositionRepository, EventAtcPositionTransactionRepository, UserAtcPermissionRecord,
};
use super::repository::event_slot::EventSlotRepository;
use super::repository::event_slot_booking::EventSlotBookingRepository;

#[derive(Clone)]
pub struct EventService {
    db: PgPool,
    audit_log: AuditLogService,
    user: UserService,
}

impl EventService {
    pub fn new(db: PgPool, audit_log: AuditLogService, user: UserService) -> Self {
        Self {
            db,
            audit_log,
            user,
        }
    }

    pub async fn list_current(&self) -> Result<Vec<Event>, EventServiceError> {
        Ok(self.db.list_event_current().await?)
    }

    pub async fn list_past(
        &self,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Vec<Event>, EventServiceError> {
        Ok(self.db.list_event_past(since, until).await?)
    }

    pub async fn find(&self, id: Uuid) -> Result<Event, EventServiceError> {
        self.db
            .find_event_by_id(id)
            .await?
            .ok_or(EventServiceError::EventNotFound(id))
    }

    pub async fn create(
        &self,
        event: EventSave,
        operated_by: Uuid,
    ) -> Result<Event, EventServiceError> {
        let mut transaction = self.db.begin().await?;
        let event = (&mut *transaction).create_event(event).await?;
        transaction.commit().await?;
        self.audit_log
            .record(
                AuditLogEntity::Event(event.id),
                operated_by,
                None::<&Event>,
                Some(&event),
            )
            .await?;
        Ok(event)
    }

    pub async fn update(
        &self,
        id: Uuid,
        event: EventSave,
        operated_by: Uuid,
    ) -> Result<Event, EventServiceError> {
        let mut transaction = self.db.begin().await?;
        let before = (&mut *transaction)
            .find_event_by_id_for_update(id)
            .await?
            .ok_or(EventServiceError::EventNotFound(id))?;
        let event = (&mut *transaction)
            .update_event(id, event)
            .await?
            .ok_or(EventServiceError::EventNotFound(id))?;
        transaction.commit().await?;
        self.audit_log
            .record(
                AuditLogEntity::Event(event.id),
                operated_by,
                Some(&before),
                Some(&event),
            )
            .await?;
        Ok(event)
    }

    pub async fn create_airspace(
        &self,
        event_id: Uuid,
        airspace: EventAirspaceSave,
    ) -> Result<EventAirspace, EventServiceError> {
        self.ensure_event(event_id).await?;
        Ok(self.db.create_event_airspace(event_id, airspace).await?)
    }

    pub async fn list_slots(
        &self,
        event_id: Uuid,
    ) -> Result<Vec<EventSlotView>, EventServiceError> {
        self.ensure_event(event_id).await?;
        let slots = self.db.list_event_slot_by_event(event_id).await?;
        try_join_all(slots.into_iter().map(|slot| self.with_slot_user(slot))).await
    }

    pub async fn create_slot(
        &self,
        event_id: Uuid,
        slot: EventSlotSave,
        operated_by: Uuid,
    ) -> Result<EventSlotView, EventServiceError> {
        self.ensure_event(event_id).await?;
        let airspace = self
            .db
            .find_event_airspace_by_id(slot.airspace_id)
            .await?
            .ok_or(EventServiceError::AirspaceNotFound(slot.airspace_id))?;
        if airspace.event_id != event_id {
            return Err(EventServiceError::AirspaceNotFound(slot.airspace_id));
        }
        let mut transaction = self.db.begin().await?;
        let slot = (&mut *transaction).create_event_slot(slot).await?;
        transaction.commit().await?;
        self.audit_log
            .record(
                AuditLogEntity::EventSlot(event_id, slot.id),
                operated_by,
                None::<&EventSlot>,
                Some(&slot),
            )
            .await?;
        self.with_slot_user(slot).await
    }

    pub async fn export_slot_bookings(
        &self,
        event_id: Uuid,
    ) -> Result<Vec<String>, EventServiceError> {
        self.ensure_event(event_id).await?;
        Ok(self.db.booking_event_slot_export_rows(event_id).await?)
    }

    pub async fn create_slot_booking(
        &self,
        event_id: Uuid,
        slot_id: Uuid,
        user_id: Uuid,
        is_admin_booking: bool,
    ) -> Result<EventBookingView, EventServiceError> {
        let mut transaction = self.db.begin().await?;
        let state = (&mut *transaction)
            .load_event_slot_booking_state_for_update(event_id, slot_id)
            .await?;
        if !state.event_exists {
            return Err(EventServiceError::EventNotFound(event_id));
        }
        if !state.slot_exists {
            return Err(EventServiceError::SlotNotFound(slot_id));
        }
        if state.booking_id.is_some() {
            return Err(EventServiceError::SlotBooked);
        }
        if !state.is_in_booking_period && !is_admin_booking {
            return Err(EventServiceError::NotInBookingPeriod);
        }
        (&mut *transaction)
            .create_event_slot_booking_booking(slot_id, user_id)
            .await?;
        transaction.commit().await?;
        let booking = self
            .db
            .find_event_slot_booking_booking(event_id, slot_id)
            .await?
            .ok_or(EventServiceError::SlotNotBooked)?;
        self.with_booking_user(booking).await
    }

    pub async fn delete_slot_booking(
        &self,
        event_id: Uuid,
        slot_id: Uuid,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<EventBookingView, EventServiceError> {
        let mut transaction = self.db.begin().await?;
        let state = (&mut *transaction)
            .load_event_slot_booking_state_for_update(event_id, slot_id)
            .await?;
        if !state.slot_exists {
            return Err(EventServiceError::SlotNotFound(slot_id));
        }
        let booking_id = state.booking_id.ok_or(EventServiceError::SlotNotBooked)?;
        if !state.is_in_booking_period && !is_admin {
            return Err(EventServiceError::NotInBookingPeriod);
        }
        if state.booking_user_id != Some(current_user_id) && !is_admin {
            return Err(EventServiceError::SlotBookedByAnotherUser);
        }
        let booking = (&mut *transaction)
            .find_event_slot_booking_booking(event_id, slot_id)
            .await?
            .ok_or(EventServiceError::SlotNotBooked)?;
        (&mut *transaction)
            .delete_event_slot_booking_booking(booking_id)
            .await?;
        transaction.commit().await?;
        self.with_booking_user(booking).await
    }

    pub async fn list_atc_positions(
        &self,
        event_id: Uuid,
    ) -> Result<Vec<EventAtcPositionView>, EventServiceError> {
        let positions = self.db.list_event_atc_position_by_event(event_id).await?;
        try_join_all(
            positions
                .into_iter()
                .map(|position| self.with_position_user(position)),
        )
        .await
    }

    pub async fn create_atc_position(
        &self,
        event_id: Uuid,
        position: EventAtcPositionSave,
        operated_by: Uuid,
    ) -> Result<EventAtcPositionView, EventServiceError> {
        let mut transaction = self.db.begin().await?;
        let position = (&mut *transaction)
            .create_event_atc_position(event_id, position)
            .await?;
        transaction.commit().await?;
        self.record_position_audit(&position, None, Some(&position), operated_by)
            .await?;
        self.with_position_user(position).await
    }

    pub async fn update_atc_position(
        &self,
        event_id: Uuid,
        position_id: Uuid,
        position: EventAtcPositionSave,
        operated_by: Uuid,
    ) -> Result<EventAtcPositionView, EventServiceError> {
        let mut transaction = self.db.begin().await?;
        let before = (&mut *transaction)
            .find_event_atc_position_by_event_and_id_in_transaction(event_id, position_id, true)
            .await?
            .ok_or(EventServiceError::AtcPositionNotFound(position_id))?;
        let position = (&mut *transaction)
            .update_event_atc_position(event_id, position_id, position)
            .await?
            .ok_or(EventServiceError::AtcPositionNotFound(position_id))?;
        transaction
            .sync_event_atc_position_booking(&position)
            .await?;
        transaction.commit().await?;
        self.record_position_audit(&position, Some(&before), Some(&position), operated_by)
            .await?;
        self.with_position_user(position).await
    }

    pub async fn delete_atc_position(
        &self,
        event_id: Uuid,
        position_id: Uuid,
        operated_by: Uuid,
    ) -> Result<(), EventServiceError> {
        let mut transaction = self.db.begin().await?;
        let position = (&mut *transaction)
            .find_event_atc_position_by_event_and_id_in_transaction(event_id, position_id, true)
            .await?
            .ok_or(EventServiceError::AtcPositionNotFound(position_id))?;
        if !(&mut *transaction)
            .delete_event_atc_position(event_id, position_id)
            .await?
        {
            return Err(EventServiceError::AtcPositionNotFound(position_id));
        }
        if let Some(booking_id) = position.booking_id {
            transaction.delete_event_atc_booking(booking_id).await?;
        }
        transaction.commit().await?;
        self.record_position_audit(&position, Some(&position), None, operated_by)
            .await
    }

    pub async fn book_atc_position(
        &self,
        event_id: Uuid,
        position_id: Uuid,
        user_id: Uuid,
        operated_by: Uuid,
        is_admin_booking: bool,
    ) -> Result<EventAtcPositionView, EventServiceError> {
        let mut transaction = self.db.begin().await?;
        let position = (&mut *transaction)
            .find_event_atc_position_by_event_and_id_in_transaction(event_id, position_id, true)
            .await?
            .ok_or(EventServiceError::AtcPositionNotFound(position_id))?;
        if position.booking_id.is_some() {
            return Err(EventServiceError::AtcPositionBooked);
        }
        let event = self.find(position.event_id).await?;
        if event
            .start_atc_booking_at
            .is_some_and(|start_at| Utc::now() <= start_at)
            && !is_admin_booking
        {
            return Err(EventServiceError::NotInBookingPeriod);
        }
        let permission = self
            .db
            .user_event_atc_position_permission(user_id, &position.position_kind_id)
            .await?
            .ok_or(EventServiceError::InsufficientAtcPermission)?;
        if !permission_satisfies(&permission, position.minimum_controller_state) {
            return Err(EventServiceError::InsufficientAtcPermission);
        }
        transaction
            .create_event_atc_position_booking(&position, user_id)
            .await?;
        let after = (&mut *transaction)
            .find_event_atc_position_by_event_and_id_in_transaction(event_id, position_id, false)
            .await?
            .ok_or(EventServiceError::AtcPositionNotFound(position_id))?;
        transaction.commit().await?;
        self.record_position_audit(&after, Some(&position), Some(&after), operated_by)
            .await?;
        self.with_position_user(after).await
    }

    pub async fn cancel_atc_position_booking(
        &self,
        event_id: Uuid,
        position_id: Uuid,
        current_user_id: Uuid,
        is_admin: bool,
    ) -> Result<EventAtcPositionView, EventServiceError> {
        let mut transaction = self.db.begin().await?;
        let position = (&mut *transaction)
            .find_event_atc_position_by_event_and_id_in_transaction(event_id, position_id, true)
            .await?
            .ok_or(EventServiceError::AtcPositionNotFound(position_id))?;
        let booking_id = position
            .booking_id
            .ok_or(EventServiceError::AtcPositionNotBooked)?;
        let booking = self
            .db
            .find_event_atc_booking(booking_id)
            .await?
            .ok_or(EventServiceError::AtcPositionNotBooked)?;
        let booking_user_id = booking.user_id;
        let user = self.optional_user(Some(booking_user_id)).await?;
        let event = self.find(position.event_id).await?;
        if booking_user_id != current_user_id && !is_admin {
            return Err(EventServiceError::AtcPositionBookedByAnotherUser);
        }
        transaction
            .delete_event_atc_position_booking(position.id, position.booking_id)
            .await?;
        let after = (&mut *transaction)
            .find_event_atc_position_by_event_and_id_in_transaction(event_id, position_id, false)
            .await?
            .ok_or(EventServiceError::AtcPositionNotFound(position_id))?;
        transaction.commit().await?;
        self.record_position_audit(&after, Some(&position), Some(&after), current_user_id)
            .await?;
        Ok(EventAtcPositionView {
            position,
            event,
            booking: Some(EventAtcBookingView { booking, user }),
        })
    }

    async fn ensure_event(&self, event_id: Uuid) -> Result<(), EventServiceError> {
        if self.db.exists_event(event_id).await? {
            Ok(())
        } else {
            Err(EventServiceError::EventNotFound(event_id))
        }
    }

    async fn with_slot_user(&self, slot: EventSlot) -> Result<EventSlotView, EventServiceError> {
        let airspace = self
            .db
            .find_event_airspace_by_id(slot.airspace_id)
            .await?
            .ok_or(EventServiceError::AirspaceNotFound(slot.airspace_id))?;
        let booking = match slot.booking_id {
            Some(booking_id) => {
                let booking = self
                    .db
                    .find_event_booking_by_id(booking_id)
                    .await?
                    .ok_or(EventServiceError::SlotNotBooked)?;
                Some(self.with_booking_user(booking).await?)
            }
            None => None,
        };
        Ok(EventSlotView {
            slot,
            airspace,
            booking,
        })
    }

    async fn with_booking_user(
        &self,
        booking: EventBooking,
    ) -> Result<EventBookingView, EventServiceError> {
        let user = self.optional_user(Some(booking.user_id)).await?;
        Ok(EventBookingView { booking, user })
    }

    async fn with_position_user(
        &self,
        position: EventAtcPosition,
    ) -> Result<EventAtcPositionView, EventServiceError> {
        let event = self.find(position.event_id).await?;
        let booking = match position.booking_id {
            Some(booking_id) => {
                let booking = self
                    .db
                    .find_event_atc_booking(booking_id)
                    .await?
                    .ok_or(EventServiceError::AtcPositionNotBooked)?;
                let user = self.optional_user(Some(booking.user_id)).await?;
                Some(EventAtcBookingView { booking, user })
            }
            None => None,
        };
        Ok(EventAtcPositionView {
            position,
            event,
            booking,
        })
    }

    async fn optional_user(
        &self,
        user_id: Option<Uuid>,
    ) -> Result<Option<UserSummary>, EventServiceError> {
        let Some(user_id) = user_id else {
            return Ok(None);
        };
        self.user
            .find_summary_by_id(user_id)
            .await?
            .map(Some)
            .ok_or(EventServiceError::UserNotFound(user_id))
    }

    async fn record_position_audit(
        &self,
        position: &EventAtcPosition,
        before: Option<&EventAtcPosition>,
        after: Option<&EventAtcPosition>,
        operated_by: Uuid,
    ) -> Result<(), EventServiceError> {
        self.audit_log
            .record(
                AuditLogEntity::EventAtcPosition(position.event_id, position.id),
                operated_by,
                before,
                after,
            )
            .await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct EventSlotView {
    pub slot: EventSlot,
    pub airspace: EventAirspace,
    pub booking: Option<EventBookingView>,
}

#[derive(Debug)]
pub struct EventBookingView {
    pub booking: EventBooking,
    pub user: Option<UserSummary>,
}

#[derive(Debug)]
pub struct EventAtcPositionView {
    pub position: EventAtcPosition,
    pub event: Event,
    pub booking: Option<EventAtcBookingView>,
}

#[derive(Debug)]
pub struct EventAtcBookingView {
    pub booking: EventAtcBooking,
    pub user: Option<UserSummary>,
}

fn permission_satisfies(permission: &UserAtcPermissionRecord, minimum_state: i32) -> bool {
    let permission_rank = match permission.state.as_str() {
        "Student" => 0,
        "UnderMentor" => 1,
        "Solo" => {
            if permission
                .solo_expires_at
                .is_some_and(|expires_at| expires_at <= Utc::now())
            {
                return minimum_state <= 1;
            }
            2
        }
        "Certified" => 3,
        "Mentor" => return true,
        _ => return false,
    };
    permission_rank >= minimum_state
}

#[derive(Debug, thiserror::Error)]
pub enum EventServiceError {
    #[error("event {0} not found")]
    EventNotFound(Uuid),
    #[error("event slot {0} not found")]
    SlotNotFound(Uuid),
    #[error("event airspace {0} not found")]
    AirspaceNotFound(Uuid),
    #[error("event ATC position {0} not found")]
    AtcPositionNotFound(Uuid),
    #[error("user {0} referenced by an event was not found")]
    UserNotFound(Uuid),
    #[error("event is not in its booking period")]
    NotInBookingPeriod,
    #[error("event slot is already booked")]
    SlotBooked,
    #[error("event slot is not booked")]
    SlotNotBooked,
    #[error("event slot is booked by another user")]
    SlotBookedByAnotherUser,
    #[error("event ATC position is already booked")]
    AtcPositionBooked,
    #[error("event ATC position is not booked")]
    AtcPositionNotBooked,
    #[error("event ATC position is booked by another user")]
    AtcPositionBookedByAnotherUser,
    #[error("insufficient ATC permission")]
    InsufficientAtcPermission,
    #[error("failed to access event data: {0}")]
    Database(#[from] sqlx::Error),
    #[error("failed to record event audit log: {0}")]
    AuditLog(#[from] AuditLogServiceError),
    #[error("failed to access event user: {0}")]
    User(#[from] UserServiceError),
}
