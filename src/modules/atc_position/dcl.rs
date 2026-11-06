use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DclAirports(Arc<HashSet<String>>);

impl DclAirports {
    pub async fn load(url: &str) -> Result<Self, DclAirportsError> {
        let airports = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<String>>()
            .await?;

        Self::try_from_codes(airports)
    }

    pub fn try_from_codes(airports: Vec<String>) -> Result<Self, DclAirportsError> {
        if airports.is_empty() {
            return Err(DclAirportsError::Empty);
        }

        let mut normalized = HashSet::with_capacity(airports.len());
        for airport in airports {
            if airport.len() != 4 || !airport.bytes().all(|byte| byte.is_ascii_uppercase()) {
                return Err(DclAirportsError::InvalidAirport(airport));
            }
            normalized.insert(airport);
        }

        Ok(Self(Arc::new(normalized)))
    }

    pub fn contains(&self, airport: &str) -> bool {
        self.0.contains(airport)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DclAirportsError {
    #[error("failed to load DCL airport list: {0}")]
    Request(#[from] reqwest::Error),
    #[error("DCL airport list must not be empty")]
    Empty,
    #[error("invalid DCL airport code: {0}")]
    InvalidAirport(String),
}

#[cfg(test)]
mod tests {
    use super::DclAirports;

    #[test]
    fn validates_airport_codes() {
        assert!(DclAirports::try_from_codes(vec!["ZBAA".to_owned()]).is_ok());
        assert!(DclAirports::try_from_codes(Vec::new()).is_err());
        assert!(DclAirports::try_from_codes(vec!["zbaa".to_owned()]).is_err());
        assert!(DclAirports::try_from_codes(vec!["ZBAA_TWR".to_owned()]).is_err());
    }
}
