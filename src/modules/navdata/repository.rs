use std::path::Path;

use sqlx::{FromRow, SqlitePool, sqlite::SqliteConnectOptions};

use crate::modules::navdata::models::{LevelRestrictionType, PreferredRoute};

#[derive(Debug, thiserror::Error)]
pub enum PreferredRouteRepositoryError {
    #[error("failed to read preferred route database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("invalid preferred route cruising level restriction: {0}")]
    InvalidLevelRestriction(String),
    #[error("invalid preferred route altitude: {0}")]
    InvalidAltitude(String),
}

#[derive(Debug, Clone)]
pub struct PreferredRouteRepository {
    db: SqlitePool,
}

impl PreferredRouteRepository {
    pub async fn from_db_path(
        path: impl AsRef<Path>,
    ) -> Result<Self, PreferredRouteRepositoryError> {
        let options = SqliteConnectOptions::new().filename(path).read_only(true);
        let db = SqlitePool::connect_with(options).await?;
        Ok(Self { db })
    }

    pub async fn list_preferred_routes(
        &self,
        departure: &str,
        arrival: &str,
    ) -> Result<Vec<PreferredRoute>, PreferredRouteRepositoryError> {
        // Keep the source order: matching uses the first suitable route in each priority tier.
        let records = sqlx::query_as::<_, PreferredRouteRecord>(
            r#"
            SELECT Dep AS dep, Arr AS arr, Waypoint1 AS waypoint1, Waypoint2 AS waypoint2,
                Name AS name, EvenOdd AS even_odd, AltList AS alt_list, MinAlt AS min_alt,
                Route AS route, Remarks AS remarks
            FROM Routes
            WHERE coalesce(nullif(trim(Dep), ''), trim(Waypoint1)) = ? COLLATE NOCASE
                AND coalesce(nullif(trim(Arr), ''), trim(Waypoint2)) = ? COLLATE NOCASE
            ORDER BY rowid
            "#,
        )
        .bind(departure)
        .bind(arrival)
        .fetch_all(&self.db)
        .await?;
        records.into_iter().map(PreferredRoute::try_from).collect()
    }
}

#[derive(Debug, FromRow)]
struct PreferredRouteRecord {
    dep: String,
    arr: String,
    waypoint1: String,
    waypoint2: String,
    name: String,
    even_odd: String,
    alt_list: String,
    min_alt: String,
    route: String,
    remarks: String,
}

impl TryFrom<PreferredRouteRecord> for PreferredRoute {
    type Error = PreferredRouteRepositoryError;

    fn try_from(record: PreferredRouteRecord) -> Result<Self, Self::Error> {
        let cruising_level_restriction = LevelRestrictionType::from_code(&record.even_odd)
            .map_err(|_| {
                PreferredRouteRepositoryError::InvalidLevelRestriction(record.even_odd.clone())
            })?;

        Ok(Self {
            name: record.name.trim().to_owned(),
            departure: endpoint(&record.dep, &record.waypoint1),
            arrival: endpoint(&record.arr, &record.waypoint2),
            raw_route: record.route.trim().to_owned(),
            cruising_level_restriction,
            allowed_altitudes: parse_altitude_list(&record.alt_list)?,
            minimal_altitude: parse_optional_i32(&record.min_alt)?,
            remarks: record.remarks.trim().to_owned(),
            valid_from: None,
            valid_until: None,
            is_public: !record.remarks.to_ascii_lowercase().contains("aip route"),
        })
    }
}

fn endpoint(airport: &str, waypoint: &str) -> String {
    if airport.trim().is_empty() {
        waypoint
    } else {
        airport
    }
    .trim()
    .to_ascii_uppercase()
}

fn parse_altitude_list(value: &str) -> Result<Vec<i32>, PreferredRouteRepositoryError> {
    value
        .split('/')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(parse_altitude)
        .collect()
}

fn parse_altitude(value: &str) -> Result<i32, PreferredRouteRepositoryError> {
    if let Some(level) = value.strip_prefix('F') {
        return level
            .parse::<i32>()
            .map(|level| level * 100)
            .map_err(|_| PreferredRouteRepositoryError::InvalidAltitude(value.to_owned()));
    }

    if let Some(level) = value.strip_prefix('S') {
        let metric_altitude = level
            .parse::<i32>()
            .map(|level| level * 100)
            .map_err(|_| PreferredRouteRepositoryError::InvalidAltitude(value.to_owned()))?;

        return standard_altitude_to_flight_level(metric_altitude)
            .ok_or_else(|| PreferredRouteRepositoryError::InvalidAltitude(value.to_owned()));
    }

    value
        .parse::<i32>()
        .map_err(|_| PreferredRouteRepositoryError::InvalidAltitude(value.to_owned()))
}

fn parse_optional_i32(value: &str) -> Result<i32, PreferredRouteRepositoryError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(0);
    }

    value
        .parse()
        .map_err(|_| PreferredRouteRepositoryError::InvalidAltitude(value.to_owned()))
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
