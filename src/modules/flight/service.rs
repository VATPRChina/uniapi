use std::collections::BTreeMap;
use std::sync::LazyLock;

use crate::adapter::compat::{CompatClient, CompatClientError};
use crate::modules::controller::models::CompatFutureController;
use crate::modules::flight::flight_plan::v1::validator::WarningMessage;
use crate::modules::flight::flight_plan::{ParseRouteError, parse_route, v1};
use crate::modules::flight::models::ParsedRoute;
use crate::modules::navdata::service::NavdataService;
use crate::modules::user::service::user::{UserService, UserServiceError};
use regex::Regex;

use super::flight_plan::validator::{self, ValidatorError};
use super::models::{CompatController, CompatPilot, CompatStatus, Flight, ValidatorExecution};
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
            .find(|flight| flight.flight_plan.callsign.eq_ignore_ascii_case(callsign))
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

    /// Parse and expand a complete route using the v2 pipeline.
    pub async fn route_v2(&self, route: &str) -> Result<ParsedRoute, FlightServiceError> {
        parse_route(&self.navdata, route)
            .await
            .map_err(|error| match error {
                ParseRouteError::InvalidRoute(reason) => FlightServiceError::InvalidRoute(reason),
                ParseRouteError::Navdata(source) => {
                    FlightServiceError::Parser(ParseRouteError::Navdata(source))
                }
            })
    }

    pub async fn by_departure(&self, icao: &str) -> Result<Vec<Flight>, FlightServiceError> {
        Ok(self
            .list()
            .await?
            .into_iter()
            .filter(|flight| flight.flight_plan.departure.eq_ignore_ascii_case(icao))
            .collect())
    }

    pub async fn by_arrival(&self, icao: &str) -> Result<Vec<Flight>, FlightServiceError> {
        Ok(self
            .list()
            .await?
            .into_iter()
            .filter(|flight| flight.flight_plan.arrival.eq_ignore_ascii_case(icao))
            .collect())
    }

    pub async fn flight_route(
        &self,
        flight: &Flight,
    ) -> Result<super::dto::FlightRouteDto, FlightServiceError> {
        let plan = &flight.flight_plan;
        let route = self
            .route_v2(&format!(
                "{} {} {}",
                plan.departure, plan.raw_route, plan.arrival
            ))
            .await?;
        let segments = super::flight_plan::RouteSegmenter::new(self.navdata.china_boundaries())
            .segment(&route.legs);
        Ok(super::dto::FlightRouteDto::from_segments(&segments))
    }

    pub async fn validate(
        &self,
        flight: &Flight,
    ) -> Result<Vec<ValidatorExecution>, FlightServiceError> {
        let plan = &flight.flight_plan;
        let route = parse_route(
            &self.navdata,
            &format!("{} {} {}", plan.departure, plan.raw_route, plan.arrival),
        )
        .await
        .map_err(|error| match error {
            ParseRouteError::InvalidRoute(reason) => FlightServiceError::InvalidRoute(reason),
            error => FlightServiceError::Parser(error),
        })?;
        Ok(validator::validate_route(flight, &route, &self.navdata).await?)
    }

    #[deprecated]
    #[allow(deprecated)]
    pub async fn route_v1(&self, flight: &Flight) -> Result<ParsedRoute, FlightServiceError> {
        let legs = v1::parser::parse_route(
            &self.navdata,
            &format!(
                "{} {} {}",
                flight.flight_plan.departure,
                flight.flight_plan.raw_route,
                flight.flight_plan.arrival
            ),
        )
        .await?;
        Ok(ParsedRoute { legs })
    }

    #[deprecated]
    #[allow(deprecated)]
    pub async fn warnings_by_callsign(
        &self,
        callsign: &str,
    ) -> Result<Vec<WarningMessage>, FlightServiceError> {
        let flight = self.find_by_callsign(callsign).await?;
        self.warnings(&flight).await
    }

    #[deprecated]
    #[allow(deprecated)]
    pub async fn warnings(
        &self,
        flight: &Flight,
    ) -> Result<Vec<WarningMessage>, FlightServiceError> {
        let legs = self.route_v1(flight).await?;
        v1::validator::validate_route(&self.navdata, flight, &legs.legs)
            .await
            .map_err(Into::into)
    }

    #[deprecated]
    #[allow(deprecated)]
    pub async fn warnings_for_all(
        &self,
    ) -> Result<BTreeMap<String, Vec<WarningMessage>>, FlightServiceError> {
        let validations =
            futures::future::join_all(self.list().await?.into_iter().map(|flight| async move {
                let callsign = flight.flight_plan.callsign.clone();
                self.warnings(&flight)
                    .await
                    .map(|warnings| (callsign, warnings))
            }))
            .await;

        validations.into_iter().collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FlightServiceError {
    #[error("invalid flight route: {0}")]
    InvalidRoute(String),
    #[error("callsign {0} not found")]
    CallsignNotFound(String),
    #[error("user {0} not found")]
    UserNotFound(uuid::Uuid),
    #[error("flight not found for CID")]
    FlightNotFoundForCid,
    #[error("failed to retrieve flights: {0}")]
    Compat(#[from] CompatClientError),
    #[error("failed to parse flight route: {0}")]
    Parser(#[from] ParseRouteError),
    #[error("failed to parse flight route: {0}")]
    #[deprecated]
    ParserV1(#[from] v1::parser::ParserError),
    #[error("failed to validate flight route: {0}")]
    Validator(#[from] ValidatorError),
    #[error("failed to access flight user: {0}")]
    User(#[from] UserServiceError),
}
