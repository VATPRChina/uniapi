use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::modules::controller::models::CompatFutureController;
use crate::modules::navdata::models::{AnyFix, ResolvedLeg};

use super::models::{CompatController, CompatPilot, CompatStatus, Flight};

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

#[derive(Deserialize, utoipa::ToSchema)]
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
