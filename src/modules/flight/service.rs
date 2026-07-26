use std::collections::BTreeMap;
use std::sync::LazyLock;

use crate::adapter::compat::{CompatClient, CompatClientError};
use crate::modules::controller::models::CompatFutureController;
use crate::modules::navdata::models::ResolvedLeg;
use crate::modules::navdata::service::NavdataService;
use crate::modules::user::service::user::{UserService, UserServiceError};
use regex::Regex;

use super::flight_plan::parser::{self, ParserError};
use super::flight_plan::validator::{self, ValidatorError, WarningMessage};
use super::models::{CompatController, CompatPilot, CompatStatus, Flight};
use super::repository::flight::FlightRepository;

static VATPRC_CONTROLLER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(Z[BSGUHWJPLYM][A-Z0-9]{2}(_[A-Z0-9]*)?_(DEL|GND|TWR|APP|DEP|CTR))|(PRC_FSS)$")
        .expect("VATPRC controller regex should compile")
});
static VATPRC_AIRPORT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^Z[BMSPGJYWLH][A-Z]{2}").expect("VATPRC airport regex should compile")
});

#[derive(Clone)]
pub struct FlightService {
    compat: CompatClient,
    navdata: NavdataService,
    user: UserService,
}

impl FlightService {
    pub fn new(compat: CompatClient, navdata: NavdataService, user: UserService) -> Self {
        Self {
            compat,
            navdata,
            user,
        }
    }

    pub async fn list(&self) -> Result<Vec<Flight>, FlightServiceError> {
        Ok(self.compat.list_flights().await?)
    }

    pub async fn compat_online_status(
        &self,
        future_controllers: Vec<CompatFutureController>,
    ) -> Result<CompatStatus, FlightServiceError> {
        let vatsim_data = self.compat.get_online_data().await?;
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
        self.compat.get_metar(icao).await
    }

    pub async fn track_audio_version(&self) -> Result<String, FlightServiceError> {
        Ok(self.compat.get_track_audio_version().await?)
    }

    pub async fn vplaaf_areas(&self) -> Result<String, FlightServiceError> {
        Ok(self.compat.get_vplaaf_areas().await?)
    }

    pub async fn find_by_callsign(&self, callsign: &str) -> Result<Flight, FlightServiceError> {
        self.list()
            .await?
            .into_iter()
            .find(|flight| flight.callsign.eq_ignore_ascii_case(callsign))
            .ok_or_else(|| FlightServiceError::CallsignNotFound(callsign.to_owned()))
    }

    pub async fn find_by_user(&self, user_id: uuid::Uuid) -> Result<Flight, FlightServiceError> {
        let user = self
            .user
            .find_summary_by_id(user_id)
            .await?
            .ok_or(FlightServiceError::UserNotFound(user_id))?;
        self.list()
            .await?
            .into_iter()
            .find(|flight| flight.cid == user.cid)
            .ok_or(FlightServiceError::FlightNotFoundForCid)
    }

    pub async fn route_by_callsign(
        &self,
        callsign: &str,
    ) -> Result<Vec<ResolvedLeg>, FlightServiceError> {
        let flight = self.find_by_callsign(callsign).await?;
        self.route(&flight).await
    }

    pub async fn warnings_by_callsign(
        &self,
        callsign: &str,
    ) -> Result<Vec<WarningMessage>, FlightServiceError> {
        let flight = self.find_by_callsign(callsign).await?;
        self.warnings(&flight).await
    }

    pub async fn route(&self, flight: &Flight) -> Result<Vec<ResolvedLeg>, FlightServiceError> {
        Ok(parser::parse_route(&self.navdata, &route_string(flight)).await?)
    }

    pub async fn warnings(
        &self,
        flight: &Flight,
    ) -> Result<Vec<WarningMessage>, FlightServiceError> {
        let legs = self.route(flight).await?;
        Ok(validator::validate_route(&self.navdata, flight, &legs).await?)
    }

    pub async fn warnings_for_all(
        &self,
    ) -> Result<BTreeMap<String, Vec<WarningMessage>>, FlightServiceError> {
        let validations =
            futures::future::join_all(self.list().await?.into_iter().map(|flight| async move {
                let callsign = flight.callsign.clone();
                self.warnings(&flight)
                    .await
                    .map(|warnings| (callsign, warnings))
            }))
            .await;

        validations.into_iter().collect()
    }
}

fn route_string(flight: &Flight) -> String {
    format!(
        "{} {} {}",
        flight.departure, flight.raw_route, flight.arrival
    )
}

#[derive(Debug, thiserror::Error)]
pub enum FlightServiceError {
    #[error("callsign {0} not found")]
    CallsignNotFound(String),
    #[error("user {0} not found")]
    UserNotFound(uuid::Uuid),
    #[error("flight not found for CID")]
    FlightNotFoundForCid,
    #[error("failed to retrieve flights: {0}")]
    Compat(#[from] CompatClientError),
    #[error("failed to parse flight route: {0}")]
    Parser(#[from] ParserError),
    #[error("failed to validate flight route: {0}")]
    Validator(#[from] ValidatorError),
    #[error("failed to access flight user: {0}")]
    User(#[from] UserServiceError),
}
