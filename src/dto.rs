use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::repository::compat::FutureControllerRow;
use crate::routes::ApiError;

pub fn parse_ulid_uuid(field: &'static str, id: &str) -> Result<Uuid, ApiError> {
    id.parse::<Ulid>()
        .map(Uuid::from)
        .map_err(|_| ApiError::bad_request(field, "invalid ULID"))
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
