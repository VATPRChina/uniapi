use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

use super::models::{AtcPosition, AtcPositionCategory, AtcPositionSave};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AtcPositionSaveRequest {
    pub category: AtcPositionCategory,
    pub callsign: String,
    pub is_tier_2: bool,
    pub callsign_zh: Option<String>,
    pub callsign_en: Option<String>,
    /// Radio frequency in MHz, for example `118.500`.
    pub frequency: f64,
    pub cpdlc_code: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AtcPositionDto {
    pub category: AtcPositionCategory,
    pub callsign: String,
    pub is_tier_2: bool,
    pub callsign_zh: Option<String>,
    pub callsign_en: Option<String>,
    /// Radio frequency in MHz.
    pub frequency: f64,
    /// Exact radio frequency in kHz for machine consumers.
    pub frequency_khz: i32,
    pub cpdlc_code: Option<String>,
    pub remarks: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<AtcPositionSaveRequest> for AtcPositionSave {
    type Error = ApiError;

    fn try_from(request: AtcPositionSaveRequest) -> Result<Self, Self::Error> {
        let callsign = normalize_callsign(request.callsign)?;
        let frequency_khz = frequency_to_khz(request.frequency)?;

        Ok(Self {
            category: request.category,
            callsign,
            is_tier_2: request.is_tier_2,
            callsign_zh: optional_trimmed(request.callsign_zh),
            callsign_en: optional_trimmed(request.callsign_en),
            frequency_khz,
            cpdlc_code: optional_trimmed(request.cpdlc_code),
            remarks: optional_trimmed(request.remarks),
        })
    }
}

impl TryFrom<AtcPosition> for AtcPositionDto {
    type Error = ApiError;

    fn try_from(position: AtcPosition) -> Result<Self, Self::Error> {
        let category = position.category.parse().map_err(|_| {
            ApiError::invalid_database_value("atc_position.category", &position.category)
        })?;

        Ok(Self {
            category,
            callsign: position.callsign,
            is_tier_2: position.is_tier_2,
            callsign_zh: position.callsign_zh,
            callsign_en: position.callsign_en,
            frequency: f64::from(position.frequency_khz) / 1000.0,
            frequency_khz: position.frequency_khz,
            cpdlc_code: position.cpdlc_code,
            remarks: position.remarks,
            created_at: position.created_at,
            updated_at: position.updated_at,
        })
    }
}

fn frequency_to_khz(frequency: f64) -> Result<i32, ApiError> {
    if !frequency.is_finite() {
        return Err(ApiError::bad_request(
            "frequency",
            "must be a finite number",
        ));
    }

    let frequency_khz = (frequency * 1000.0).round();
    if !(118_000.0..=136_975.0).contains(&frequency_khz)
        || (frequency * 1000.0 - frequency_khz).abs() > 0.000_1
        || frequency_khz as i32 % 5 != 0
    {
        return Err(ApiError::bad_request(
            "frequency",
            "must be a valid 5 kHz VHF channel between 118.000 and 136.975 MHz",
        ));
    }

    Ok(frequency_khz as i32)
}

pub(crate) fn normalize_callsign(callsign: String) -> Result<String, ApiError> {
    let callsign = required_trimmed("callsign", callsign)?.to_ascii_uppercase();
    if callsign.len() > 32
        || !callsign
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '*'))
    {
        return Err(ApiError::bad_request(
            "callsign",
            "must contain only ASCII letters, digits, underscores, or wildcards and be at most 32 characters",
        ));
    }
    Ok(callsign)
}

fn required_trimmed(field: &str, value: String) -> Result<String, ApiError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ApiError::bad_request(field, "must not be empty"));
    }
    Ok(value.to_owned())
}

fn optional_trimmed(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{frequency_to_khz, normalize_callsign};

    #[test]
    fn converts_valid_frequency_to_exact_khz() {
        assert_eq!(frequency_to_khz(121.825).unwrap(), 121_825);
        assert_eq!(frequency_to_khz(131.010).unwrap(), 131_010);
    }

    #[test]
    fn rejects_invalid_frequency() {
        assert!(frequency_to_khz(117.995).is_err());
        assert!(frequency_to_khz(121.823).is_err());
        assert!(frequency_to_khz(f64::NAN).is_err());
    }

    #[test]
    fn normalizes_and_validates_callsign() {
        assert_eq!(
            normalize_callsign(" zbaa_twr ".to_owned()).unwrap(),
            "ZBAA_TWR"
        );
        assert_eq!(
            normalize_callsign("*_mil_twr".to_owned()).unwrap(),
            "*_MIL_TWR"
        );
        assert!(normalize_callsign("ZBAA TWR".to_owned()).is_err());
        assert!(normalize_callsign("ZBAA_TWR（停用）".to_owned()).is_err());
    }
}
