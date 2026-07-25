use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::models::{CompatController, CompatFutureController, CompatPilot, CompatStatus};

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

impl From<CompatStatus> for CompatVatprcStatusDto {
    fn from(status: CompatStatus) -> Self {
        Self {
            last_updated: status.last_updated,
            pilots: status.pilots.into_iter().map(Into::into).collect(),
            controllers: status.controllers.into_iter().map(Into::into).collect(),
            future_controllers: status
                .future_controllers
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
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

impl From<CompatPilot> for CompatPilotDto {
    fn from(pilot: CompatPilot) -> Self {
        Self {
            cid: pilot.cid,
            name: pilot.name,
            callsign: pilot.callsign,
            departure: pilot.departure,
            arrival: pilot.arrival,
            aircraft: pilot.aircraft,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatControllerDto {
    pub cid: i32,
    pub name: String,
    pub callsign: String,
    pub frequency: String,
}

impl From<CompatController> for CompatControllerDto {
    fn from(controller: CompatController) -> Self {
        Self {
            cid: controller.cid,
            name: controller.name,
            callsign: controller.callsign,
            frequency: controller.frequency,
        }
    }
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

impl From<CompatFutureController> for CompatFutureControllerDto {
    fn from(controller: CompatFutureController) -> Self {
        Self {
            callsign: controller.callsign,
            name: controller.name,
            start: controller.start_at.format("%d %H:%M").to_string(),
            start_utc: controller.start_at,
            end: controller.end_at.format("%d %H:%M").to_string(),
            end_utc: controller.end_at,
        }
    }
}
