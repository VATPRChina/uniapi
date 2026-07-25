use std::collections::BTreeMap;

use crate::adapter::compat::{CompatClient, CompatClientError};
use crate::modules::navdata::models::ResolvedLeg;
use crate::modules::navdata::service::NavdataService;
use crate::modules::user::service::user::{UserService, UserServiceError};

use super::flight_plan::parser::{self, ParserError};
use super::flight_plan::validator::{self, ValidatorError, WarningMessage};
use super::models::Flight;
use super::repository::flight::FlightRepository;

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
