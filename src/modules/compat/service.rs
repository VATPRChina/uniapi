use std::sync::LazyLock;

use regex::Regex;
use sqlx::PgPool;

use crate::adapter::compat::{CompatClient, CompatClientError};

use super::models::{CompatController, CompatFutureController, CompatPilot, CompatStatus};
use super::repository::CompatRepository;

static VATPRC_CONTROLLER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(Z[BSGUHWJPLYM][A-Z0-9]{2}(_[A-Z0-9]*)?_(DEL|GND|TWR|APP|DEP|CTR))|(PRC_FSS)$")
        .expect("VATPRC controller regex should compile")
});
static VATPRC_AIRPORT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^Z[BMSPGJYWLH][A-Z]{2}").expect("VATPRC airport regex should compile")
});

#[derive(Clone)]
pub struct CompatService {
    db: PgPool,
    client: CompatClient,
}

impl CompatService {
    pub fn new(db: PgPool, client: CompatClient) -> Self {
        Self { db, client }
    }

    pub async fn online_status(&self) -> Result<CompatStatus, CompatServiceError> {
        let vatsim_data = self.client.get_online_data().await?;
        let future_controllers = self
            .db
            .future_compat_controllers()
            .await?
            .into_iter()
            .map(|controller| CompatFutureController {
                callsign: controller.callsign,
                name: controller.name,
                start_at: controller.start_at,
                end_at: controller.end_at,
            })
            .collect();
        let pilots = vatsim_data
            .pilots
            .into_iter()
            .filter_map(|pilot| {
                let flight_plan = pilot.flight_plan?;
                let departure_matches = flight_plan
                    .departure
                    .as_deref()
                    .is_some_and(|airport| VATPRC_AIRPORT_REGEX.is_match(airport));
                let arrival_matches = flight_plan
                    .arrival
                    .as_deref()
                    .is_some_and(|airport| VATPRC_AIRPORT_REGEX.is_match(airport));
                (departure_matches || arrival_matches).then_some(CompatPilot {
                    cid: pilot.cid as i32,
                    name: pilot.name,
                    callsign: pilot.callsign,
                    departure: flight_plan.departure,
                    arrival: flight_plan.arrival,
                    aircraft: flight_plan.aircraft_short,
                })
            })
            .collect();
        let controllers = vatsim_data
            .controllers
            .into_iter()
            .filter(|controller| VATPRC_CONTROLLER_REGEX.is_match(&controller.callsign))
            .filter(|controller| controller.facility > 0)
            .map(|controller| CompatController {
                cid: controller.cid as i32,
                name: controller.name,
                callsign: controller.callsign,
                frequency: controller.frequency,
            })
            .collect();

        Ok(CompatStatus {
            last_updated: vatsim_data.general.update_timestamp,
            pilots,
            controllers,
            future_controllers,
        })
    }

    pub async fn metar(&self, icao: &str) -> String {
        self.client.get_metar(icao).await
    }

    pub async fn track_audio_version(&self) -> Result<String, CompatServiceError> {
        Ok(self.client.get_track_audio_version().await?)
    }

    pub async fn vplaaf_areas(&self) -> Result<String, CompatServiceError> {
        Ok(self.client.get_vplaaf_areas().await?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompatServiceError {
    #[error("failed to access compat data: {0}")]
    Client(#[from] CompatClientError),
    #[error("failed to query compat data: {0}")]
    Database(#[from] sqlx::Error),
}
