use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::adapter::flight::Flight;
use crate::model::controller_info::{ControllerInfo, ControllerPermission, ControllerRating};
use crate::model::navdata::{AnyFix, ResolvedLeg};
use crate::model::user_controller_state::UserControllerState;
use crate::modules::user::dto::UserDto;
use crate::repository::atc::user_atc_permission::{AtcPermissionRecord, AtcPermissionSave};
use crate::repository::atc::user_atc_status::{AtcStatusRecord, AtcStatusSave};
use crate::repository::compat::FutureControllerRow;
use crate::repository::sheet::sheet::{SheetRecord, SheetSave};
use crate::repository::sheet::sheet_field::{SheetFieldRecord, SheetFieldSave};
use crate::repository::sheet::sheet_filing_answer::{SheetAnswerRecord, SheetAnswerSave};
use crate::routes::ApiError;

pub fn parse_ulid_uuid(field: &'static str, id: &str) -> Result<Uuid, ApiError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| ApiError::bad_request(field, "invalid ULID"))
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
        if request.rating.parse::<ControllerRating>().is_err() {
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
}

impl From<ControllerInfo> for AtcStatusDto {
    fn from(controller: ControllerInfo) -> Self {
        let user_id = controller.user.id;
        Self {
            user_id: Ulid::from(user_id).to_string(),
            user: UserDto::from_user_summary(controller.user, true),
            is_visiting: controller.is_visiting,
            is_absent: controller.is_absent,
            rating: controller.rating.to_string(),
            permissions: controller.permissions.into_iter().map(Into::into).collect(),
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

impl From<ControllerPermission> for AtcPermissionDto {
    fn from(permission: ControllerPermission) -> Self {
        Self {
            position_kind_id: permission.position_kind.to_string(),
            state: permission.state,
            solo_expires_at: permission.solo_expires_at,
        }
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
pub struct UploadImageResponse {
    pub url: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SectorPermissionResponse {
    pub has_permission: bool,
    pub sector_type: &'static str,
}
