use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::error::ApiError;
use crate::modules::event::dto::EventAtcPositionDto;
use crate::modules::user::dto::UserDto;

use super::models::{AtcBooking, AtcBookingSave};
use super::service::AtcBookingView;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AtcBookingSaveRequest {
    pub callsign: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub remarks: Option<String>,
}

impl TryFrom<AtcBookingSaveRequest> for AtcBookingSave {
    type Error = ApiError;

    fn try_from(request: AtcBookingSaveRequest) -> Result<Self, Self::Error> {
        let callsign = request.callsign.trim().to_uppercase();
        if callsign.is_empty() {
            return Err(ApiError::bad_request(
                "callsign",
                "callsign must not be empty",
            ));
        }
        if request.start_at >= request.end_at {
            return Err(ApiError::bad_request(
                "end_at",
                "end_at must be after start_at",
            ));
        }
        let remarks = request
            .remarks
            .map(|remarks| remarks.trim().to_string())
            .filter(|remarks| !remarks.is_empty());
        Ok(Self {
            callsign,
            start_at: request.start_at,
            end_at: request.end_at,
            remarks,
        })
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AtcBookingDto {
    pub id: String,
    pub user: UserDto,
    pub callsign: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub remarks: Option<String>,
    pub event_position: Option<EventAtcPositionDto>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<AtcBookingView> for AtcBookingDto {
    fn from(view: AtcBookingView) -> Self {
        let AtcBooking {
            id,
            callsign,
            start_at,
            end_at,
            remarks,
            created_at,
            updated_at,
            deleted_at,
            ..
        } = view.booking;
        Self {
            id: Ulid::from(id).to_string(),
            user: UserDto::from_user_summary(view.user, true),
            callsign,
            start_at,
            end_at,
            remarks,
            event_position: view.event_position.map(|position| {
                EventAtcPositionDto::from_entity(position.position, position.event, None)
            }),
            created_at,
            updated_at,
            deleted_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn request(start_hour: u32, end_hour: u32) -> AtcBookingSaveRequest {
        AtcBookingSaveRequest {
            callsign: " zbaa_twr ".to_string(),
            start_at: Utc.with_ymd_and_hms(2026, 8, 26, start_hour, 0, 0).unwrap(),
            end_at: Utc.with_ymd_and_hms(2026, 8, 26, end_hour, 0, 0).unwrap(),
            remarks: Some("  training  ".to_string()),
        }
    }

    #[test]
    fn normalizes_booking_input() {
        let save = AtcBookingSave::try_from(request(10, 12)).unwrap();
        assert_eq!(save.callsign, "ZBAA_TWR");
        assert_eq!(save.remarks.as_deref(), Some("training"));
    }

    #[test]
    fn rejects_an_invalid_time_range() {
        let error = AtcBookingSave::try_from(request(12, 10)).unwrap_err();
        assert_eq!(error.identifier(), "BadRequest");
    }
}
