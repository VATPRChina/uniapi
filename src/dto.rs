use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::adapter::flight::Flight;
use crate::model::audit_log::{AuditLog, AuditLogEntity};
use crate::model::navdata::{AnyFix, ResolvedLeg};
use crate::model::user_controller_state::UserControllerState;
use crate::model::user_role::{UserRole, role_closure_from_strings};
use crate::repository::atc::atc::AtcControllerPermissionRecord;
use crate::repository::atc::atc_application::AtcApplicationRecord;
use crate::repository::atc::user_atc_permission::{AtcPermissionRecord, AtcPermissionSave};
use crate::repository::atc::user_atc_status::{AtcStatusRecord, AtcStatusSave};
use crate::repository::atc_training::training::{TrainingRecord, TrainingSave};
use crate::repository::atc_training::training_application::TrainingApplicationRecord;
use crate::repository::atc_training::training_application_response::TrainingApplicationResponseRecord;
use crate::repository::atc_training::training_application_slot::{
    TrainingApplicationSlotRecord, TrainingApplicationSlotSave,
};
use crate::repository::auth::user::UserDetailRecord;
use crate::repository::compat::FutureControllerRow;
use crate::repository::event::event::{EventRecord, EventSave};
use crate::repository::event::event_airspace::{EventAirspaceRecord, EventAirspaceSave};
use crate::repository::event::event_atc_position::{EventAtcPositionRecord, EventAtcPositionSave};
use crate::repository::event::event_slot::{EventSlotRecord, EventSlotSave};
use crate::repository::event::event_slot_booking::EventBookingRecord;
use crate::repository::sheet::sheet::{SheetRecord, SheetSave};
use crate::repository::sheet::sheet_field::{SheetFieldRecord, SheetFieldSave};
use crate::repository::sheet::sheet_filing_answer::{SheetAnswerRecord, SheetAnswerSave};
use crate::routes::ApiError;

const ALLOWED_RATINGS: &[&str] = &["OBS", "S1", "S2", "S3", "C1", "C3", "I1", "I3"];
const POSITION_KINDS: &[&str] = &["DEL", "GND", "TWR", "T2", "APP", "CTR", "FSS", "FMP"];

pub fn parse_ulid_uuid(field: &'static str, id: &str) -> Result<Uuid, ApiError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| ApiError::bad_request(field, "invalid ULID"))
}

fn direct_roles_to_dto(roles: &[String]) -> Vec<UserRole> {
    roles
        .iter()
        .filter_map(|role| role.parse::<UserRole>().ok())
        .collect()
}

fn roles_to_dto(roles: &[String]) -> Vec<UserRole> {
    let mut roles = role_closure_from_strings(roles.iter().map(String::as_str))
        .into_iter()
        .collect::<Vec<_>>();
    roles.sort();
    roles
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AuditLogEntityKindDto {
    Event,
    AtcApplication,
    User,
    UserRole,
    UserAtcPermission,
    EventAtcPosition,
    EventSlot,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AuditLogEntityDto {
    pub kind: AuditLogEntityKindDto,
    pub id: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AuditLogDto {
    pub entity: AuditLogEntityDto,
    pub child_entity: Option<AuditLogEntityDto>,
    pub before: serde_json::Value,
    pub after: serde_json::Value,
    pub operated_by: String,
    pub created_at: DateTime<Utc>,
}

impl From<AuditLogEntity> for (AuditLogEntityDto, Option<AuditLogEntityDto>) {
    fn from(entity: AuditLogEntity) -> Self {
        match entity {
            AuditLogEntity::AtcApplication(id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::AtcApplication,
                    id: Ulid::from(id).to_string(),
                },
                None,
            ),
            AuditLogEntity::Event(id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::Event,
                    id: Ulid::from(id).to_string(),
                },
                None,
            ),
            AuditLogEntity::EventAtcPosition(pid, id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::Event,
                    id: Ulid::from(pid).to_string(),
                },
                Some(AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::EventAtcPosition,
                    id: Ulid::from(id).to_string(),
                }),
            ),
            AuditLogEntity::EventSlot(pid, id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::Event,
                    id: Ulid::from(pid).to_string(),
                },
                Some(AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::EventSlot,
                    id: Ulid::from(id).to_string(),
                }),
            ),
            AuditLogEntity::User(id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::User,
                    id: Ulid::from(id).to_string(),
                },
                None,
            ),
            AuditLogEntity::UserAtcPermission(pid, id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::User,
                    id: Ulid::from(pid).to_string(),
                },
                Some(AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::UserAtcPermission,
                    id: Ulid::from(id).to_string(),
                }),
            ),
            AuditLogEntity::UserRole(pid, id) => (
                AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::User,
                    id: Ulid::from(pid).to_string(),
                },
                Some(AuditLogEntityDto {
                    kind: AuditLogEntityKindDto::UserRole,
                    id: Ulid::from(id).to_string(),
                }),
            ),
        }
    }
}

impl From<AuditLog> for AuditLogDto {
    fn from(audit_log: AuditLog) -> Self {
        let (entity, child_entity) = audit_log.entity.into();

        Self {
            entity,
            child_entity,
            before: audit_log.before,
            after: audit_log.after,
            operated_by: Ulid::from(audit_log.operated_by).to_string(),
            created_at: audit_log.created_at,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserMoodleInfoDto {
    pub id: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserDto {
    pub id: String,
    pub cid: String,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub roles: Vec<UserRole>,
    pub direct_roles: Vec<UserRole>,
    pub moodle_account: Option<UserMoodleInfoDto>,
}

impl UserDto {
    pub fn from_role_strings(
        id: Uuid,
        cid: String,
        full_name: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        roles: Vec<String>,
    ) -> Self {
        Self {
            id: Ulid::from(id).to_string(),
            cid,
            full_name,
            created_at,
            updated_at,
            roles: roles_to_dto(&roles),
            direct_roles: direct_roles_to_dto(&roles),
            moodle_account: None,
        }
    }

    pub fn from_user_detail(
        user: UserDetailRecord,
        moodle_account: Option<UserMoodleInfoDto>,
        show_full_name: bool,
        roles_override: Option<Vec<UserRole>>,
    ) -> Self {
        let direct_roles = user
            .roles
            .iter()
            .filter_map(|role| role.parse::<UserRole>().ok())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let roles = roles_override
            .unwrap_or_else(|| {
                role_closure_from_strings(user.roles.iter().map(String::as_str))
                    .into_iter()
                    .collect()
            })
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        Self {
            id: Ulid::from(user.id).to_string(),
            cid: user.cid,
            full_name: if show_full_name {
                user.full_name
            } else {
                String::new()
            },
            created_at: user.created_at,
            updated_at: user.updated_at,
            roles,
            direct_roles,
            moodle_account,
        }
    }

    pub fn from_application_user(
        application: &AtcApplicationRecord,
        show_full_name: bool,
        moodle_account: Option<UserMoodleInfoDto>,
    ) -> Self {
        Self {
            id: Ulid::from(application.user_id).to_string(),
            cid: application.user_cid.clone(),
            full_name: if show_full_name {
                application.user_full_name.clone()
            } else {
                String::new()
            },
            created_at: application.user_created_at,
            updated_at: application.user_updated_at,
            roles: roles_to_dto(&application.user_roles),
            direct_roles: direct_roles_to_dto(&application.user_roles),
            moodle_account,
        }
    }
}

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

impl From<EventRecord> for EventDto {
    fn from(event: EventRecord) -> Self {
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

impl From<&EventAtcPositionRecord> for EventDto {
    fn from(position: &EventAtcPositionRecord) -> Self {
        Self {
            id: Ulid::from(position.event_id).to_string(),
            created_at: position.event_created_at,
            updated_at: position.event_updated_at,
            title: position.event_title.clone(),
            title_en: position.event_title_en.clone(),
            start_at: position.event_start_at,
            end_at: position.event_end_at,
            start_booking_at: position.event_start_booking_at,
            end_booking_at: position.event_end_booking_at,
            start_atc_booking_at: position.event_start_atc_booking_at,
            image_url: position.event_image_url.clone(),
            community_link: position.event_community_link.clone(),
            vatsim_link: position.event_vatsim_link.clone(),
            description: position.event_description.clone(),
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

impl From<EventAirspaceRecord> for EventAirspaceDto {
    fn from(airspace: EventAirspaceRecord) -> Self {
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
            airspace_id: parse_ulid_uuid("airspace_id", &request.airspace_id)?,
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
    pub fn from_record(slot: EventSlotRecord, include_booking_user: bool) -> Self {
        let booking = EventBookingDto::from_slot_record(&slot, include_booking_user);
        Self {
            id: Ulid::from(slot.id).to_string(),
            event_id: Ulid::from(slot.event_id).to_string(),
            airspace_id: Ulid::from(slot.airspace_id).to_string(),
            airspace: EventAirspaceDto {
                id: Ulid::from(slot.airspace_id).to_string(),
                event_id: Ulid::from(slot.event_id).to_string(),
                name: slot.airspace_name,
                created_at: slot.airspace_created_at,
                updated_at: slot.airspace_updated_at,
                icao_codes: slot.airspace_icao_codes,
                description: slot.airspace_description,
            },
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
    pub fn from_slot_record(slot: &EventSlotRecord, include_user: bool) -> Option<Self> {
        let id = slot.booking_id?;
        Some(Self {
            id: Ulid::from(id).to_string(),
            user_id: Ulid::from(slot.booking_user_id?).to_string(),
            user: if include_user {
                Some(UserDto::from_role_strings(
                    slot.booking_user_id?,
                    slot.booking_user_cid.clone()?,
                    String::new(),
                    slot.booking_user_created_at?,
                    slot.booking_user_updated_at?,
                    slot.booking_user_roles.clone().unwrap_or_default(),
                ))
            } else {
                None
            },
            created_at: slot.booking_created_at?,
            updated_at: slot.booking_updated_at?,
        })
    }

    pub fn from_booking_record(booking: EventBookingRecord, include_user: bool) -> Self {
        Self {
            id: Ulid::from(booking.id).to_string(),
            user_id: Ulid::from(booking.user_id).to_string(),
            user: if include_user {
                Some(UserDto::from_role_strings(
                    booking.user_id,
                    booking.user_cid.unwrap_or_default(),
                    String::new(),
                    booking.user_created_at.unwrap_or(booking.created_at),
                    booking.user_updated_at.unwrap_or(booking.updated_at),
                    booking.user_roles.unwrap_or_default(),
                ))
            } else {
                None
            },
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
        if !POSITION_KINDS.contains(&request.position_kind_id.as_str()) {
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

impl From<EventAtcPositionRecord> for EventAtcPositionDto {
    fn from(position: EventAtcPositionRecord) -> Self {
        Self {
            id: Ulid::from(position.id).to_string(),
            event: EventDto::from(&position),
            callsign: position.callsign.clone(),
            start_at: position.start_at,
            end_at: position.end_at,
            remarks: position.remarks.clone(),
            position_kind_id: position.position_kind_id.clone(),
            minimum_controller_state: UserControllerState::from_db_value(
                position.minimum_controller_state,
            ),
            booking: EventAtcPositionBookingDto::try_from(position).ok(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct EventAtcPositionBookingDto {
    pub user_id: String,
    pub user: UserDto,
    pub booked_at: DateTime<Utc>,
}

impl TryFrom<EventAtcPositionRecord> for EventAtcPositionBookingDto {
    type Error = ApiError;

    fn try_from(position: EventAtcPositionRecord) -> Result<Self, Self::Error> {
        let user_id = position
            .booking_user_id
            .ok_or(ApiError::PositionNotBooked)?;
        Ok(Self {
            user_id: Ulid::from(user_id).to_string(),
            user: UserDto::from_role_strings(
                user_id,
                position.booking_user_cid.unwrap_or_default(),
                position.booking_user_full_name.unwrap_or_default(),
                position
                    .booking_user_created_at
                    .ok_or(ApiError::PositionNotBooked)?,
                position
                    .booking_user_updated_at
                    .ok_or(ApiError::PositionNotBooked)?,
                position.booking_user_roles.unwrap_or_default(),
            ),
            booked_at: position
                .booking_created_at
                .ok_or(ApiError::PositionNotBooked)?,
        })
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AtcStatusRequest {
    pub is_visiting: bool,
    pub is_absent: bool,
    pub rating: String,
    pub permissions: Vec<AtcPermissionRequest>,
}

impl TryFrom<AtcStatusRequest> for AtcStatusSave {
    type Error = ApiError;

    fn try_from(request: AtcStatusRequest) -> Result<Self, Self::Error> {
        if !ALLOWED_RATINGS.contains(&request.rating.as_str()) {
            return Err(ApiError::bad_request("rating", "invalid ATC rating"));
        }

        if request.permissions.iter().any(|permission| {
            permission.state == UserControllerState::Solo && permission.solo_expires_at.is_none()
        }) {
            return Err(ApiError::SoloExpirationNotProvided);
        }

        if request.is_absent
            && request.permissions.iter().any(|permission| {
                permission.state.to_db_value() > UserControllerState::UnderMentor.to_db_value()
            })
        {
            return Err(ApiError::bad_request(
                "permissions",
                "absent users cannot have ATC permission higher than under mentor",
            ));
        }

        Ok(Self {
            is_visiting: request.is_visiting,
            is_absent: request.is_absent,
            rating: request.rating,
            permissions: request
                .permissions
                .into_iter()
                .map(AtcPermissionSave::from)
                .collect(),
        })
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AtcPermissionRequest {
    pub position_kind_id: String,
    pub state: UserControllerState,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

impl From<AtcPermissionRequest> for AtcPermissionSave {
    fn from(permission: AtcPermissionRequest) -> Self {
        Self {
            position_kind_id: permission.position_kind_id,
            state: permission.state.as_db_str().to_owned(),
            solo_expires_at: permission.solo_expires_at,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AtcStatusDto {
    pub user_id: String,
    pub user: UserDto,
    pub is_visiting: bool,
    pub is_absent: bool,
    pub rating: String,
    pub permissions: Vec<AtcPermissionDto>,
}

impl AtcStatusDto {
    pub fn from_records(
        status: AtcStatusRecord,
        permissions: Vec<AtcPermissionRecord>,
    ) -> Result<Self, ApiError> {
        Ok(Self {
            user_id: Ulid::from(status.user_id).to_string(),
            user: UserDto::from_role_strings(
                status.user_id,
                status.user_cid,
                status.user_full_name,
                status.user_created_at,
                status.user_updated_at,
                status.user_roles,
            ),
            is_visiting: status.is_visiting.unwrap_or(false),
            is_absent: status.is_absent.unwrap_or(false),
            rating: status.rating.unwrap_or_else(|| "OBS".to_owned()),
            permissions: permissions
                .into_iter()
                .map(AtcPermissionDto::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    pub fn from_controller_rows(
        rows: Vec<AtcControllerPermissionRecord>,
    ) -> Result<Vec<Self>, ApiError> {
        let mut statuses = std::collections::BTreeMap::<Uuid, AtcStatusControllerBuilder>::new();
        for row in rows {
            let permission = AtcPermissionDto::try_from(&row)?;
            statuses
                .entry(row.user_id)
                .or_insert_with(|| AtcStatusControllerBuilder::from(&row))
                .permissions
                .push(permission);
        }

        Ok(statuses
            .into_values()
            .map(AtcStatusDto::from)
            .collect::<Vec<_>>())
    }
}

struct AtcStatusControllerBuilder {
    user_id: Uuid,
    user: UserDto,
    is_visiting: bool,
    is_absent: bool,
    rating: String,
    permissions: Vec<AtcPermissionDto>,
}

impl AtcStatusControllerBuilder {
    fn from(row: &AtcControllerPermissionRecord) -> Self {
        Self {
            user_id: row.user_id,
            user: UserDto::from_role_strings(
                row.user_id,
                row.user_cid.clone(),
                row.user_full_name.clone(),
                row.user_created_at,
                row.user_updated_at,
                row.user_roles.clone(),
            ),
            is_visiting: row.is_visiting.unwrap_or(false),
            is_absent: row.is_absent.unwrap_or(false),
            rating: row.rating.clone().unwrap_or_else(|| "OBS".to_owned()),
            permissions: Vec::new(),
        }
    }
}

impl From<AtcStatusControllerBuilder> for AtcStatusDto {
    fn from(status: AtcStatusControllerBuilder) -> Self {
        Self {
            user_id: Ulid::from(status.user_id).to_string(),
            user: status.user,
            is_visiting: status.is_visiting,
            is_absent: status.is_absent,
            rating: status.rating,
            permissions: status.permissions,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AtcPermissionDto {
    pub position_kind_id: String,
    pub state: UserControllerState,
    pub solo_expires_at: Option<DateTime<Utc>>,
}

impl TryFrom<AtcPermissionRecord> for AtcPermissionDto {
    type Error = ApiError;

    fn try_from(permission: AtcPermissionRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            position_kind_id: permission.position_kind_id,
            state: permission.state.parse().map_err(|_| {
                ApiError::invalid_database_value(
                    "user_atc_permission.state",
                    permission.state.clone(),
                )
            })?,
            solo_expires_at: permission.solo_expires_at,
        })
    }
}

impl TryFrom<&AtcControllerPermissionRecord> for AtcPermissionDto {
    type Error = ApiError;

    fn try_from(permission: &AtcControllerPermissionRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            position_kind_id: permission.position_kind_id.clone(),
            state: permission.state.parse().map_err(|_| {
                ApiError::invalid_database_value(
                    "user_atc_permission.state",
                    permission.state.clone(),
                )
            })?,
            solo_expires_at: permission.solo_expires_at,
        })
    }
}

#[cfg(test)]
mod atc_permission_dto_tests {
    use super::*;

    fn atc_status_request(is_absent: bool, state: UserControllerState) -> AtcStatusRequest {
        AtcStatusRequest {
            is_visiting: false,
            is_absent,
            rating: "S2".to_owned(),
            permissions: vec![AtcPermissionRequest {
                position_kind_id: "TWR".to_owned(),
                state,
                solo_expires_at: Some(Utc::now()),
            }],
        }
    }

    #[test]
    fn invalid_controller_state_returns_error() {
        let record = AtcPermissionRecord {
            position_kind_id: "APP".to_owned(),
            state: "student".to_owned(),
            solo_expires_at: None,
        };

        assert!(AtcPermissionDto::try_from(record).is_err());
    }

    #[test]
    fn absent_user_cannot_have_permission_higher_than_under_mentor() {
        for state in [
            UserControllerState::Solo,
            UserControllerState::Certified,
            UserControllerState::Mentor,
        ] {
            assert!(matches!(
                AtcStatusSave::try_from(atc_status_request(true, state)),
                Err(ApiError::BadRequest { field, .. }) if field == "permissions"
            ));
        }
    }

    #[test]
    fn absent_user_can_have_permission_up_to_under_mentor() {
        for state in [
            UserControllerState::Student,
            UserControllerState::UnderMentor,
        ] {
            assert!(AtcStatusSave::try_from(atc_status_request(true, state)).is_ok());
        }
    }

    #[test]
    fn non_absent_user_can_have_permission_higher_than_under_mentor() {
        assert!(
            AtcStatusSave::try_from(atc_status_request(false, UserControllerState::Mentor)).is_ok()
        );
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SheetRequestField {
    pub id: String,
    pub answer: String,
}

impl From<SheetRequestField> for SheetAnswerSave {
    fn from(answer: SheetRequestField) -> Self {
        Self {
            field_id: answer.id,
            answer: answer.answer,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SheetDto {
    pub id: String,
    pub name: String,
    pub fields: Vec<SheetFieldDto>,
}

impl SheetDto {
    pub fn from_records(sheet: SheetRecord, fields: Vec<SheetFieldRecord>) -> Self {
        Self {
            id: sheet.id,
            name: sheet.name,
            fields: fields.into_iter().map(SheetFieldDto::from).collect(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SheetFieldAnswerDto {
    pub field: SheetFieldDto,
    pub answer: String,
}

impl From<SheetAnswerRecord> for SheetFieldAnswerDto {
    fn from(answer: SheetAnswerRecord) -> Self {
        Self {
            field: SheetFieldDto {
                sheet_id: answer.sheet_id,
                id: answer.field_id,
                sequence: u32::try_from(answer.field_sequence).unwrap_or_default(),
                name_zh: answer.field_name_zh,
                name_en: answer.field_name_en,
                kind: answer.field_kind,
                single_choice_options: answer.field_single_choice_options,
                description_zh: answer.field_description_zh,
                description_en: answer.field_description_en,
                is_deleted: answer.field_is_deleted,
            },
            answer: answer.answer,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SheetFieldKind {
    ShortText,
    LongText,
    SingleChoice,
}

impl std::fmt::Display for SheetFieldKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.serialize(f)
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SheetFieldDto {
    pub sheet_id: String,
    pub id: String,
    #[schema(format = "uint32")]
    pub sequence: u32,
    pub name_zh: String,
    pub name_en: Option<String>,
    #[schema(value_type = SheetFieldKind)]
    pub kind: String,
    pub single_choice_options: Vec<String>,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
    pub is_deleted: bool,
}

impl From<SheetFieldRecord> for SheetFieldDto {
    fn from(field: SheetFieldRecord) -> Self {
        Self {
            sheet_id: field.sheet_id,
            id: field.id,
            sequence: u32::try_from(field.sequence).unwrap_or_default(),
            name_zh: field.name_zh,
            name_en: field.name_en,
            kind: field.kind,
            single_choice_options: field.single_choice_options,
            description_zh: field.description_zh,
            description_en: field.description_en,
            is_deleted: field.is_deleted,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SheetSaveRequest {
    pub name: String,
    pub fields: Vec<SheetFieldSaveRequest>,
}

impl From<SheetSaveRequest> for SheetSave {
    fn from(request: SheetSaveRequest) -> Self {
        Self {
            name: request.name,
            fields: request
                .fields
                .into_iter()
                .map(SheetFieldSave::from)
                .collect(),
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SheetFieldSaveRequest {
    pub id: String,
    #[schema(format = "uint32")]
    pub sequence: u32,
    pub name_zh: String,
    pub name_en: Option<String>,
    pub kind: SheetFieldKind,
    pub single_choice_options: Vec<String>,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
}

impl From<SheetFieldSaveRequest> for SheetFieldSave {
    fn from(field: SheetFieldSaveRequest) -> Self {
        Self {
            id: field.id,
            sequence: i64::from(field.sequence),
            name_zh: field.name_zh,
            name_en: field.name_en,
            kind: field.kind.to_string(),
            single_choice_options: field.single_choice_options,
            description_zh: field.description_zh,
            description_en: field.description_en,
            is_deleted: false,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AtcApplicationRequest {
    pub request_answers: Vec<SheetRequestField>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AtcApplicationReviewRequest {
    pub status: AtcApplicationStatus,
    pub review_answers: Vec<SheetRequestField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AtcApplicationStatus {
    Submitted,
    InWaitlist,
    Approved,
    Rejected,
    Aborted,
}

impl AtcApplicationStatus {
    pub fn as_db_str(self) -> &'static str {
        match self {
            Self::Submitted => "Submitted",
            Self::InWaitlist => "InWaitlist",
            Self::Approved => "Approved",
            Self::Rejected => "Rejected",
            Self::Aborted => "Aborted",
        }
    }

    pub fn from_db_str(status: &str) -> Result<Self, ApiError> {
        match status {
            "Submitted" => Ok(Self::Submitted),
            "InWaitlist" => Ok(Self::InWaitlist),
            "Approved" => Ok(Self::Approved),
            "Rejected" => Ok(Self::Rejected),
            "Aborted" => Ok(Self::Aborted),
            _ => Err(ApiError::invalid_database_value(
                "atc_application.status",
                status,
            )),
        }
    }
}

#[cfg(test)]
mod atc_application_status_tests {
    use super::*;

    #[test]
    fn invalid_database_status_returns_error() {
        assert!(AtcApplicationStatus::from_db_str("submitted").is_err());
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AtcApplicationSummaryDto {
    pub id: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_email: Option<String>,
    pub user: UserDto,
    pub applied_at: DateTime<Utc>,
    pub status: AtcApplicationStatus,
}

impl AtcApplicationSummaryDto {
    pub fn from_record(
        application: AtcApplicationRecord,
        is_admin: bool,
        current_user_id: Uuid,
    ) -> Result<Self, ApiError> {
        let user_email = is_admin.then_some(application.user_email.clone()).flatten();
        Ok(Self {
            id: Ulid::from(application.id).to_string(),
            user_id: Ulid::from(application.user_id).to_string(),
            user_email,
            user: UserDto::from_application_user(
                &application,
                is_admin || application.user_id == current_user_id,
                None,
            ),
            applied_at: application.applied_at,
            status: AtcApplicationStatus::from_db_str(&application.status)?,
        })
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AtcApplicationDto {
    pub id: String,
    pub user_id: String,
    pub user: UserDto,
    pub applied_at: DateTime<Utc>,
    pub status: AtcApplicationStatus,
    pub application_filing_answers: Vec<SheetFieldAnswerDto>,
    pub review_filing_answers: Option<Vec<SheetFieldAnswerDto>>,
}

impl AtcApplicationDto {
    pub fn from_record(
        application: AtcApplicationRecord,
        is_admin: bool,
        current_user_id: Uuid,
        application_filing_answers: Vec<SheetFieldAnswerDto>,
        review_filing_answers: Option<Vec<SheetFieldAnswerDto>>,
        moodle_account: Option<UserMoodleInfoDto>,
    ) -> Result<Self, ApiError> {
        Ok(Self {
            id: Ulid::from(application.id).to_string(),
            user_id: Ulid::from(application.user_id).to_string(),
            user: UserDto::from_application_user(
                &application,
                is_admin || application.user_id == current_user_id,
                moodle_account,
            ),
            applied_at: application.applied_at,
            status: AtcApplicationStatus::from_db_str(&application.status)?,
            application_filing_answers,
            review_filing_answers,
        })
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TrainingSaveRequest {
    pub name: String,
    pub trainer_id: String,
    pub trainee_id: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

impl TryFrom<TrainingSaveRequest> for TrainingSave {
    type Error = ApiError;

    fn try_from(request: TrainingSaveRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            name: request.name,
            trainer_id: parse_ulid_uuid("id", &request.trainer_id)?,
            trainee_id: parse_ulid_uuid("id", &request.trainee_id)?,
            start_at: request.start_at,
            end_at: request.end_at,
        })
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TrainingRecordRequest {
    pub request_answers: Vec<SheetRequestField>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TrainingDto {
    pub id: String,
    pub name: String,
    pub trainer_id: String,
    pub trainer: UserDto,
    pub trainee_id: String,
    pub trainee: UserDto,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub record_sheet_filing_id: Option<String>,
    pub record_sheet_filing: Option<Vec<SheetFieldAnswerDto>>,
}

impl TrainingDto {
    pub fn from_record(
        training: TrainingRecord,
        record_sheet_filing: Option<Vec<SheetFieldAnswerDto>>,
    ) -> Self {
        Self {
            id: Ulid::from(training.id).to_string(),
            name: training.name,
            trainer_id: Ulid::from(training.trainer_id).to_string(),
            trainer: UserDto::from_role_strings(
                training.trainer_id,
                training.trainer_cid,
                training.trainer_full_name,
                training.trainer_created_at,
                training.trainer_updated_at,
                training.trainer_roles,
            ),
            trainee_id: Ulid::from(training.trainee_id).to_string(),
            trainee: UserDto::from_role_strings(
                training.trainee_id,
                training.trainee_cid,
                training.trainee_full_name,
                training.trainee_created_at,
                training.trainee_updated_at,
                training.trainee_roles,
            ),
            start_at: training.start_at,
            end_at: training.end_at,
            created_at: training.created_at,
            updated_at: training.updated_at,
            deleted_at: training.deleted_at,
            record_sheet_filing_id: training
                .record_sheet_filing_id
                .map(|id| Ulid::from(id).to_string()),
            record_sheet_filing,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TrainingApplicationCreateRequest {
    pub name: String,
    pub slots: Vec<TrainingApplicationCreateRequestSlot>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TrainingApplicationCreateRequestSlot {
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

impl From<TrainingApplicationCreateRequestSlot> for TrainingApplicationSlotSave {
    fn from(slot: TrainingApplicationCreateRequestSlot) -> Self {
        Self {
            start_at: slot.start_at,
            end_at: slot.end_at,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TrainingApplicationResponseRequest {
    pub slot_id: Option<String>,
    pub comment: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TrainingApplicationDto {
    pub id: String,
    pub trainee_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trainee_email: Option<String>,
    pub trainee: UserDto,
    pub status: TrainingApplicationStatus,
    pub name: String,
    pub train_id: Option<String>,
    pub slots: Vec<TrainingApplicationSlotDto>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TrainingApplicationDto {
    pub fn from_record(
        application: TrainingApplicationRecord,
        slots: Vec<TrainingApplicationSlotRecord>,
        include_trainee_email: bool,
    ) -> Self {
        let status = TrainingApplicationStatus::from_record(&application, &slots);
        let trainee_email = include_trainee_email
            .then_some(application.trainee_email)
            .flatten();
        Self {
            id: Ulid::from(application.id).to_string(),
            trainee_id: Ulid::from(application.trainee_id).to_string(),
            trainee_email,
            trainee: UserDto::from_role_strings(
                application.trainee_id,
                application.trainee_cid,
                application.trainee_full_name,
                application.trainee_created_at,
                application.trainee_updated_at,
                application.trainee_roles,
            ),
            status,
            name: application.name,
            train_id: application.train_id.map(|id| Ulid::from(id).to_string()),
            slots: slots
                .into_iter()
                .map(TrainingApplicationSlotDto::from)
                .collect(),
            created_at: application.created_at,
            updated_at: application.updated_at,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TrainingApplicationStatus {
    Pending,
    Accepted,
    Rejected,
    Cancelled,
}

impl TrainingApplicationStatus {
    fn from_record(
        application: &TrainingApplicationRecord,
        slots: &[TrainingApplicationSlotRecord],
    ) -> Self {
        if application.train_id.is_some() {
            Self::Accepted
        } else if application.deleted_at.is_some() {
            Self::Cancelled
        } else if slots
            .iter()
            .map(|slot| slot.end_at)
            .max()
            .is_some_and(|end_at| end_at < Utc::now())
        {
            Self::Rejected
        } else {
            Self::Pending
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TrainingApplicationSlotDto {
    pub id: String,
    pub application_id: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

impl From<TrainingApplicationSlotRecord> for TrainingApplicationSlotDto {
    fn from(slot: TrainingApplicationSlotRecord) -> Self {
        Self {
            id: Ulid::from(slot.id).to_string(),
            application_id: Ulid::from(slot.application_id).to_string(),
            start_at: slot.start_at,
            end_at: slot.end_at,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TrainingApplicationResponseDto {
    pub id: String,
    pub application_id: String,
    pub trainer_id: String,
    pub trainer: UserDto,
    pub is_accepted: bool,
    pub comment: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TrainingApplicationResponseRecord> for TrainingApplicationResponseDto {
    fn from(response: TrainingApplicationResponseRecord) -> Self {
        Self {
            id: Ulid::from(response.id).to_string(),
            application_id: Ulid::from(response.application_id).to_string(),
            trainer_id: Ulid::from(response.trainer_id).to_string(),
            trainer: UserDto::from_role_strings(
                response.trainer_id,
                response.trainer_cid,
                response.trainer_full_name,
                response.trainer_created_at,
                response.trainer_updated_at,
                response.trainer_roles,
            ),
            is_accepted: response.slot_id.is_some(),
            comment: response.comment,
            created_at: response.created_at,
            updated_at: response.updated_at,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
#[allow(dead_code)]
pub struct PreferredRouteSaveRequest {
    pub departure: String,
    pub arrival: String,
    pub raw_route: String,
    pub cruising_level_restriction: LevelRestrictionType,
    #[serde(default)]
    pub allowed_altitudes: Vec<i32>,
    pub minimal_altitude: i32,
    pub remarks: String,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LevelRestrictionType {
    StandardEven,
    StandardOdd,
    Standard,
    FlightLevelEven,
    FlightLevelOdd,
    FlightLevel,
}

#[derive(Serialize, utoipa::ToSchema)]
#[allow(dead_code)]
pub struct PreferredRouteDto {
    pub id: String,
    pub departure: String,
    pub arrival: String,
    pub raw_route: String,
    pub cruising_level_restriction: LevelRestrictionType,
    pub allowed_altitudes: Vec<i32>,
    pub minimal_altitude: i32,
    pub remarks: String,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[allow(dead_code)]
pub struct TemporaryFlightQuery {
    pub departure: String,
    pub arrival: String,
    #[serde(default)]
    pub aircraft: String,
    #[serde(default)]
    pub equipment: String,
    #[serde(default)]
    pub navigation_performance: String,
    #[serde(default)]
    pub transponder: String,
    #[serde(default)]
    pub raw_route: String,
    #[serde(default)]
    pub cruising_level: i64,
}

impl From<TemporaryFlightQuery> for Flight {
    fn from(query: TemporaryFlightQuery) -> Self {
        Self {
            id: Ulid::new(),
            cid: String::new(),
            callsign: String::new(),
            last_observed_at: Utc::now(),
            departure: query.departure,
            arrival: query.arrival,
            equipment: query.equipment,
            navigation_performance: query.navigation_performance,
            transponder: query.transponder,
            raw_route: query.raw_route,
            aircraft: query.aircraft,
            altitude: 0,
            cruising_level: query.cruising_level,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct FlightDto {
    pub id: String,
    pub cid: String,
    pub callsign: String,
    pub last_observed_at: DateTime<Utc>,
    pub departure: String,
    pub arrival: String,
    pub equipment: String,
    pub navigation_performance: String,
    pub transponder: String,
    pub raw_route: String,
    pub aircraft: String,
    pub altitude: i64,
    pub cruising_level: i64,
}

impl From<Flight> for FlightDto {
    fn from(flight: Flight) -> Self {
        Self {
            id: flight.id.to_string(),
            cid: flight.cid,
            callsign: flight.callsign,
            last_observed_at: flight.last_observed_at,
            departure: flight.departure,
            arrival: flight.arrival,
            equipment: flight.equipment,
            navigation_performance: flight.navigation_performance,
            transponder: flight.transponder,
            raw_route: flight.raw_route,
            aircraft: flight.aircraft,
            altitude: flight.altitude,
            cruising_level: flight.cruising_level,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct FlightLeg {
    pub from: FlightFix,
    pub to: FlightFix,
    pub leg_identifier: String,
}

impl From<ResolvedLeg> for FlightLeg {
    fn from(leg: ResolvedLeg) -> Self {
        Self {
            from: FlightFix::from(&leg.from),
            to: FlightFix::from(&leg.to),
            leg_identifier: leg.identifier.unwrap_or_default(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct FlightFix {
    pub identifier: String,
}

impl From<&AnyFix> for FlightFix {
    fn from(fix: &AnyFix) -> Self {
        Self {
            identifier: fix.identifier().unwrap_or_default().to_owned(),
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct MetarQuery {
    pub id: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatVatprcStatusDto {
    pub last_updated: DateTime<Utc>,
    pub pilots: Vec<CompatPilotDto>,
    pub controllers: Vec<CompatControllerDto>,
    pub future_controllers: Vec<CompatFutureControllerDto>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatPilotDto {
    pub cid: i32,
    pub name: String,
    pub callsign: String,
    pub departure: Option<String>,
    pub arrival: Option<String>,
    pub aircraft: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatControllerDto {
    pub cid: i32,
    pub name: String,
    pub callsign: String,
    pub frequency: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatFutureControllerDto {
    pub callsign: String,
    pub name: String,
    pub start: String,
    pub start_utc: DateTime<Utc>,
    pub end: String,
    pub end_utc: DateTime<Utc>,
}

impl From<FutureControllerRow> for CompatFutureControllerDto {
    fn from(row: FutureControllerRow) -> Self {
        Self {
            callsign: row.callsign,
            name: row.name,
            start: row.start_at.format("%d %H:%M").to_string(),
            start_utc: row.start_at,
            end: row.end_at.format("%d %H:%M").to_string(),
            end_utc: row.end_at,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TokenDto {
    pub user: UserDto,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AuthorizeQuery {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: Option<String>,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct AuthenticationState {
    pub auth_type: AuthenticationStateType,
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub user_code: Option<String>,
    pub state: Option<String>,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationStateType {
    Code,
    Device,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DeviceConfirmQuery {
    pub user_code: Option<String>,
    pub confirm: Option<bool>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct LoginQuery {
    pub state: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct VatsimCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DeviceAuthorizationRequest {
    pub client_id: String,
    #[allow(dead_code)]
    pub scope: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct DeviceAuthorizationResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_uri_complete: Option<String>,
    pub expires_in: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AccessTokenRequest {
    #[serde(default)]
    pub grant_type: String,
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub device_code: String,
    #[serde(default)]
    pub refresh_token: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub code_verifier: String,
    #[serde(default)]
    pub client_secret: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UnsafeAssumeUserRequest {
    pub id: Option<String>,
    pub cid: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub scope: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UploadImageResponse {
    pub url: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SectorPermissionResponse {
    pub has_permission: bool,
    pub sector_type: &'static str,
}
