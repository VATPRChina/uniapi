use std::{path::Path, sync::Arc};

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::model::navdata::{LevelRestrictionType, PreferredRoute};

#[derive(Debug, thiserror::Error)]
pub enum StaticPreferredRouteError {
    #[error("failed to read preferred route CSV: {0}")]
    Csv(#[from] csv::Error),
    #[error("invalid preferred route cruising level restriction: {0}")]
    InvalidLevelRestriction(String),
    #[error("invalid preferred route altitude: {0}")]
    InvalidAltitude(String),
}

#[derive(Debug, Clone)]
pub struct StaticPreferredRouteAdapter {
    preferred_routes: Arc<Vec<PreferredRoute>>,
}

impl StaticPreferredRouteAdapter {
    pub fn from_csv_path(path: impl AsRef<Path>) -> Result<Self, StaticPreferredRouteError> {
        let mut reader = csv::Reader::from_path(path)?;
        let preferred_routes = reader
            .deserialize::<PreferredRouteCsvRecord>()
            .map(|record| PreferredRoute::try_from(record?))
            .collect::<Result<Vec<_>, StaticPreferredRouteError>>()?;
        let preferred_routes = Arc::new(preferred_routes);

        Ok(Self { preferred_routes })
    }

    pub fn list_preferred_routes(&self, departure: &str, arrival: &str) -> Vec<&PreferredRoute> {
        self.preferred_routes
            .iter()
            .filter(|route| {
                route.departure.eq_ignore_ascii_case(departure)
                    && route.arrival.eq_ignore_ascii_case(arrival)
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PreferredRouteCsvRecord {
    dep: String,
    arr: String,
    name: String,
    even_odd: String,
    alt_list: String,
    min_alt: String,
    route: String,
    remarks: String,
}

impl TryFrom<PreferredRouteCsvRecord> for PreferredRoute {
    type Error = StaticPreferredRouteError;

    fn try_from(record: PreferredRouteCsvRecord) -> Result<Self, Self::Error> {
        let cruising_level_restriction = LevelRestrictionType::from_csv_str(&record.even_odd)
            .map_err(|_| {
                StaticPreferredRouteError::InvalidLevelRestriction(record.even_odd.clone())
            })?;

        Ok(Self {
            name: record.name.trim().to_owned(),
            departure: record.dep.trim().to_ascii_uppercase(),
            arrival: record.arr.trim().to_ascii_uppercase(),
            raw_route: record.route.trim().to_owned(),
            cruising_level_restriction,
            allowed_altitudes: parse_altitude_list(&record.alt_list)?,
            minimal_altitude: parse_optional_i32(&record.min_alt)?,
            remarks: record.remarks.trim().to_owned(),
            valid_from: None::<DateTime<Utc>>,
            valid_until: None::<DateTime<Utc>>,
            is_public: !record.remarks.to_ascii_lowercase().contains("aip route"),
        })
    }
}

fn parse_altitude_list(value: &str) -> Result<Vec<i32>, StaticPreferredRouteError> {
    value
        .split('/')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(parse_altitude)
        .collect()
}

fn parse_altitude(value: &str) -> Result<i32, StaticPreferredRouteError> {
    if let Some(level) = value.strip_prefix('F') {
        return level
            .parse::<i32>()
            .map(|level| level * 100)
            .map_err(|_| StaticPreferredRouteError::InvalidAltitude(value.to_owned()));
    }

    if let Some(level) = value.strip_prefix('S') {
        let metric_altitude = level
            .parse::<i32>()
            .map(|level| level * 100)
            .map_err(|_| StaticPreferredRouteError::InvalidAltitude(value.to_owned()))?;

        return standard_altitude_to_flight_level(metric_altitude)
            .ok_or_else(|| StaticPreferredRouteError::InvalidAltitude(value.to_owned()));
    }

    value
        .parse::<i32>()
        .map_err(|_| StaticPreferredRouteError::InvalidAltitude(value.to_owned()))
}

fn parse_optional_i32(value: &str) -> Result<i32, StaticPreferredRouteError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(0);
    }

    value
        .parse()
        .map_err(|_| StaticPreferredRouteError::InvalidAltitude(value.to_owned()))
}

fn standard_altitude_to_flight_level(standard_altitude: i32) -> Option<i32> {
    STANDARD_ALTITUDE_TO_FLIGHT_LEVEL
        .iter()
        .find_map(|(standard, flight)| (*standard == standard_altitude).then_some(*flight))
}

const STANDARD_ALTITUDE_TO_FLIGHT_LEVEL: &[(i32, i32)] = &[
    (300, 1000),
    (900, 3000),
    (1500, 4900),
    (2100, 6900),
    (2700, 8900),
    (3300, 10800),
    (3900, 12800),
    (4500, 14800),
    (5100, 16700),
    (5700, 18700),
    (6300, 20700),
    (6900, 22600),
    (7500, 24600),
    (8100, 26600),
    (8900, 29100),
    (9500, 31100),
    (10100, 33100),
    (10700, 35100),
    (11300, 37100),
    (11900, 39100),
    (12500, 41100),
    (13700, 44900),
    (14900, 48900),
    (600, 2000),
    (1200, 3900),
    (1800, 5900),
    (2400, 7900),
    (3000, 9800),
    (3600, 11800),
    (4200, 13800),
    (4800, 15700),
    (5400, 17700),
    (6000, 19700),
    (6600, 21700),
    (7200, 23600),
    (7800, 25600),
    (8400, 27600),
    (9200, 30100),
    (9800, 32100),
    (10400, 34100),
    (11000, 36100),
    (11600, 38100),
    (12200, 40100),
    (13100, 43000),
    (14300, 46900),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flight_level_altitudes() {
        assert_eq!(
            parse_altitude_list("F250/F290/F310").unwrap(),
            vec![25000, 29000, 31000]
        );
    }

    #[test]
    fn parses_standard_altitudes_as_flight_levels() {
        assert_eq!(parse_altitude_list("S24/S84").unwrap(), vec![7900, 27600]);
    }

    #[test]
    fn loads_and_filters_project_csv() {
        let adapter = StaticPreferredRouteAdapter::from_csv_path("assets/test/routes.csv").unwrap();
        let routes = adapter.list_preferred_routes("zbaa", "zspd");

        assert!(!routes.is_empty());
        assert!(routes.iter().all(|route| route.departure == "ZBAA"));
        assert!(routes.iter().all(|route| route.arrival == "ZSPD"));
    }
}
