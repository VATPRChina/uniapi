use chrono::{Datelike, TimeZone, Utc};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

const VATSIM_DATA_URL: &str = "https://data.vatsim.net/v3/vatsim-data.json";
const VATSIM_EVENTS_URL: &str = "https://my.vatsim.net/api/v2/events/view/division/PRC";
const VATSIM_METAR_BASE_URL: &str = "https://metar.vatsim.net";
const TRACK_AUDIO_VERSION_URL: &str =
    "https://raw.githubusercontent.com/pierr3/TrackAudio/main/MANDATORY_VERSION";
const VPLAAF_AREAS_URL: &str = "https://airspace.vplaaf.org/Areas.json";

#[derive(Clone)]
pub struct CompatClient {
    http: reqwest::Client,
    metar_endpoint: String,
}

#[derive(Debug, Error)]
pub enum CompatClientError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

impl CompatClient {
    pub fn new(metar_endpoint: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("vatprc-uniapi-rust/0.1"),
        );

        Self {
            http: reqwest::Client::builder()
                .default_headers(headers)
                .timeout(Duration::from_secs(15))
                .build()
                .expect("compat reqwest client should build"),
            metar_endpoint,
        }
    }

    pub async fn get_online_data(&self) -> Result<VatsimData, CompatClientError> {
        Ok(self
            .http
            .get(VATSIM_DATA_URL)
            .send()
            .await?
            .error_for_status()?
            .json::<VatsimData>()
            .await?)
    }

    pub async fn get_vatsim_events(&self) -> Result<String, CompatClientError> {
        self.get_text(VATSIM_EVENTS_URL).await
    }

    pub async fn get_track_audio_version(&self) -> Result<String, CompatClientError> {
        self.get_text(TRACK_AUDIO_VERSION_URL).await
    }

    pub async fn get_vplaaf_areas(&self) -> Result<String, CompatClientError> {
        self.get_text(VPLAAF_AREAS_URL).await
    }

    pub async fn get_metar(&self, icao: &str) -> String {
        let rudi_metar = self
            .get_text(&format!(
                "{}/{}",
                self.metar_endpoint.trim_end_matches('/'),
                icao
            ))
            .await
            .map(|metar| metar.trim().to_string())
            .unwrap_or_default();
        let vatsim_metar = self
            .get_text(&format!("{}/{}", VATSIM_METAR_BASE_URL, icao))
            .await
            .map(|metar| metar.trim().to_string())
            .unwrap_or_default();

        match (metar_time(&rudi_metar), metar_time(&vatsim_metar)) {
            (Some(rudi_time), Some(vatsim_time)) if rudi_time > vatsim_time => rudi_metar,
            (Some(_), Some(_)) => vatsim_metar,
            (Some(_), None) => rudi_metar,
            (None, Some(_)) | (None, None) => vatsim_metar,
        }
    }

    async fn get_text(&self, url: &str) -> Result<String, CompatClientError> {
        Ok(self
            .http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?)
    }
}

fn metar_time(metar: &str) -> Option<chrono::DateTime<Utc>> {
    let time = metar.split_whitespace().nth(1)?;
    if time.len() < 7 || !time.ends_with('Z') {
        return None;
    }

    let day = time.get(0..2)?.parse::<u32>().ok()?;
    let hour = time.get(2..4)?.parse::<u32>().ok()?;
    let minute = time.get(4..6)?.parse::<u32>().ok()?;
    let mut reference = Utc::now();
    if day > reference.day() {
        reference -= chrono::Duration::days(reference.day() as i64);
    }

    Utc.with_ymd_and_hms(reference.year(), reference.month(), day, hour, minute, 0)
        .single()
}

#[derive(Debug, Deserialize)]
pub struct VatsimData {
    pub general: General,
    pub pilots: Vec<Pilot>,
    pub controllers: Vec<Controller>,
}

#[derive(Debug, Deserialize)]
pub struct General {
    pub update_timestamp: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct Pilot {
    pub cid: i64,
    pub name: String,
    pub callsign: String,
    pub flight_plan: Option<FlightPlan>,
}

#[derive(Debug, Deserialize)]
pub struct FlightPlan {
    pub aircraft_short: Option<String>,
    pub departure: Option<String>,
    pub arrival: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Controller {
    pub cid: i64,
    pub name: String,
    pub callsign: String,
    pub frequency: String,
    pub facility: i64,
}
