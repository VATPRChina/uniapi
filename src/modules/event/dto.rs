use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::modules::controller::models::{ControllerPositionKind, UserControllerState};
use crate::modules::user::dto::UserDto;
use crate::modules::user::models::UserSummary;
use crate::routes::ApiError;

use super::models::{
    Event, EventAirspace, EventAirspaceSave, EventAtcBooking, EventAtcPosition,
    EventAtcPositionSave, EventBooking, EventSave, EventSlot, EventSlotSave,
};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ListPastQuery {
    pub until: Option<DateTime<Utc>>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct EventSaveRequest {
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

impl TryFrom<EventSaveRequest> for EventSave {
    type Error = ApiError;

    fn try_from(request: EventSaveRequest) -> Result<Self, Self::Error> {
        if request.start_booking_at.is_some() ^ request.end_booking_at.is_some() {
            return Err(ApiError::bad_request(
                "start_booking_at",
                "start_booking_at and end_booking_at must be both set or null",
            ));
        }
        Ok(Self {
            title: request.title,
            title_en: request.title_en,
            start_at: request.start_at,
            end_at: request.end_at,
            start_booking_at: request.start_booking_at,
            end_booking_at: request.end_booking_at,
            start_atc_booking_at: request.start_atc_booking_at,
            image_url: request.image_url,
            community_link: request.community_link,
            vatsim_link: request.vatsim_link,
            description: request.description,
        })
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct EventDto {
    pub id: String,
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

impl From<Event> for EventDto {
    fn from(event: Event) -> Self {
        Self {
            id: Ulid::from(event.id).to_string(),
            created_at: event.created_at,
            updated_at: event.updated_at,
            title: event.title,
            title_en: event.title_en,
            start_at: event.start_at,
            end_at: event.end_at,
            start_booking_at: event.start_booking_at,
            end_booking_at: event.end_booking_at,
            start_atc_booking_at: event.start_atc_booking_at,
            image_url: event.image_url,
            community_link: event.community_link,
            vatsim_link: event.vatsim_link,
            description: event.description,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct EventAirspaceSaveRequest {
    pub name: String,
    pub icao_codes: Vec<String>,
    pub description: String,
}

impl From<EventAirspaceSaveRequest> for EventAirspaceSave {
    fn from(request: EventAirspaceSaveRequest) -> Self {
        Self {
            name: request.name,
            icao_codes: request.icao_codes,
            description: request.description,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct EventAirspaceDto {
    pub id: String,
    pub event_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub icao_codes: Vec<String>,
    pub description: String,
}

impl From<EventAirspace> for EventAirspaceDto {
    fn from(airspace: EventAirspace) -> Self {
        Self {
            id: Ulid::from(airspace.id).to_string(),
            event_id: Ulid::from(airspace.event_id).to_string(),
            name: airspace.name,
            created_at: airspace.created_at,
            updated_at: airspace.updated_at,
            icao_codes: airspace.icao_codes,
            description: airspace.description,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct EventSlotSaveRequest {
    pub airspace_id: String,
    pub enter_at: DateTime<Utc>,
    pub leave_at: Option<DateTime<Utc>>,
    pub callsign: Option<String>,
    pub aircraft_type_icao: Option<String>,
}

impl TryFrom<EventSlotSaveRequest> for EventSlotSave {
    type Error = ApiError;

    fn try_from(request: EventSlotSaveRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            airspace_id: request.airspace_id.parse::<Ulid>()?.into(),
            enter_at: request.enter_at,
            leave_at: request.leave_at,
            callsign: request.callsign,
            aircraft_type_icao: request.aircraft_type_icao,
        })
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct EventSlotDto {
    pub id: String,
    pub event_id: String,
    pub airspace_id: String,
    pub airspace: EventAirspaceDto,
    pub enter_at: DateTime<Utc>,
    pub leave_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub booking: Option<EventBookingDto>,
    pub callsign: Option<String>,
    pub aircraft_type_icao: Option<String>,
}

impl EventSlotDto {
    pub fn from_entity(
        slot: EventSlot,
        airspace: EventAirspace,
        booking: Option<(EventBooking, Option<UserSummary>)>,
        include_booking_user: bool,
    ) -> Self {
        let booking = booking.map(|(booking, user)| {
            EventBookingDto::from_entity(booking, user, include_booking_user)
        });
        Self {
            id: Ulid::from(slot.id).to_string(),
            event_id: Ulid::from(airspace.event_id).to_string(),
            airspace_id: Ulid::from(slot.airspace_id).to_string(),
            airspace: EventAirspaceDto::from(airspace),
            enter_at: slot.enter_at,
            leave_at: slot.leave_at,
            created_at: slot.created_at,
            updated_at: slot.updated_at,
            booking,
            callsign: slot.callsign,
            aircraft_type_icao: slot.aircraft_type_icao,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct EventBookingDto {
    pub id: String,
    pub user_id: String,
    pub user: Option<UserDto>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EventBookingDto {
    pub fn from_entity(
        booking: EventBooking,
        user: Option<UserSummary>,
        include_user: bool,
    ) -> Self {
        Self {
            id: Ulid::from(booking.id).to_string(),
            user_id: Ulid::from(booking.user_id).to_string(),
            user: include_user
                .then(|| user.map(|user| UserDto::from_user_summary(user, false)))
                .flatten(),
            created_at: booking.created_at,
            updated_at: booking.updated_at,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct EventSlotBookingRequest {
    pub user_id: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct EventAtcPositionSaveRequest {
    pub callsign: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub remarks: Option<String>,
    pub position_kind_id: String,
    pub minimum_controller_state: UserControllerState,
}

impl TryFrom<EventAtcPositionSaveRequest> for EventAtcPositionSave {
    type Error = ApiError;

    fn try_from(request: EventAtcPositionSaveRequest) -> Result<Self, Self::Error> {
        if request
            .position_kind_id
            .parse::<ControllerPositionKind>()
            .is_err()
        {
            return Err(ApiError::bad_request(
                "position_kind_id",
                "invalid ATC position kind",
            ));
        }
        Ok(Self {
            callsign: request.callsign,
            start_at: request.start_at,
            end_at: request.end_at,
            remarks: request.remarks,
            position_kind_id: request.position_kind_id,
            minimum_controller_state: request.minimum_controller_state.to_db_value(),
        })
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct EventAtcPositionBookRequest {
    pub user_id: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct EventAtcPositionDto {
    pub id: String,
    pub event: EventDto,
    pub callsign: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub remarks: Option<String>,
    pub position_kind_id: String,
    pub minimum_controller_state: UserControllerState,
    pub booking: Option<EventAtcPositionBookingDto>,
}

impl EventAtcPositionDto {
    pub fn from_entity(
        position: EventAtcPosition,
        event: Event,
        booking: Option<(EventAtcBooking, Option<UserSummary>)>,
    ) -> Self {
        Self {
            id: Ulid::from(position.id).to_string(),
            event: EventDto::from(event),
            callsign: position.callsign.clone(),
            start_at: position.start_at,
            end_at: position.end_at,
            remarks: position.remarks.clone(),
            position_kind_id: position.position_kind_id.clone(),
            minimum_controller_state: UserControllerState::from_db_value(
                position.minimum_controller_state,
            ),
            booking: booking.and_then(|(booking, user)| {
                EventAtcPositionBookingDto::from_entity(booking, user).ok()
            }),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct EventAtcPositionBookingDto {
    pub user_id: String,
    pub user: UserDto,
    pub booked_at: DateTime<Utc>,
}

impl EventAtcPositionBookingDto {
    pub fn from_entity(
        booking: EventAtcBooking,
        user: Option<UserSummary>,
    ) -> Result<Self, ApiError> {
        Ok(Self {
            user_id: Ulid::from(booking.user_id).to_string(),
            user: UserDto::from_user_summary(user.ok_or(ApiError::PositionNotBooked)?, true),
            booked_at: booking.booked_at,
        })
    }
}
