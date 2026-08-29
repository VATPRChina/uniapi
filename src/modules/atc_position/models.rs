use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::IntoDeserializer};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AtcPositionCategory {
    Standard,
    ChengduLowArea,
    Military,
    Atis,
}

impl std::fmt::Display for AtcPositionCategory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.serialize(formatter)
    }
}

impl std::str::FromStr for AtcPositionCategory {
    type Err = serde::de::value::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::deserialize(value.into_deserializer())
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AtcPosition {
    pub category: String,
    pub callsign: String,
    pub is_tier_2: bool,
    pub callsign_zh: Option<String>,
    pub callsign_en: Option<String>,
    pub frequency_khz: i32,
    pub cpdlc_code: Option<String>,
    pub remarks: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AtcPositionSave {
    pub category: AtcPositionCategory,
    pub callsign: String,
    pub is_tier_2: bool,
    pub callsign_zh: Option<String>,
    pub callsign_en: Option<String>,
    pub frequency_khz: i32,
    pub cpdlc_code: Option<String>,
    pub remarks: Option<String>,
}
