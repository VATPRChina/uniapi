use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

use super::dcl::DclAirports;
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

impl AtcPositionDto {
    pub fn try_from_position(
        position: AtcPosition,
        dcl_airports: &DclAirports,
    ) -> Result<Self, ApiError> {
        let category = position.category.parse().map_err(|_| {
            ApiError::invalid_database_value("atc_position.category", &position.category)
        })?;
        let dcl_code = dcl_airport_code(&position.callsign, dcl_airports).map(str::to_owned);

        Ok(Self {
            category,
            callsign: position.callsign,
            is_tier_2: position.is_tier_2,
            callsign_zh: position.callsign_zh,
            callsign_en: position.callsign_en,
            frequency: f64::from(position.frequency_khz) / 1000.0,
            frequency_khz: position.frequency_khz,
            cpdlc_code: dcl_code.or(position.cpdlc_code),
            remarks: position.remarks,
            created_at: position.created_at,
            updated_at: position.updated_at,
        })
    }
}

fn dcl_airport_code<'a>(callsign: &'a str, dcl_airports: &DclAirports) -> Option<&'a str> {
    let parts: Vec<_> = callsign.split('_').collect();
    let airport = *parts.first()?;
    if !dcl_airports.contains(airport) {
        return None;
    }

    match parts.last().copied() {
        Some("TWR" | "DEL") => Some(airport),
        Some("GND")
            if !parts[1..parts.len() - 1]
                .iter()
                .any(|part| part.starts_with('A')) =>
        {
            Some(airport)
        }
        _ => None,
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
    use chrono::Utc;

    use super::{AtcPositionDto, dcl_airport_code, frequency_to_khz, normalize_callsign};
    use crate::modules::atc_position::dcl::DclAirports;
    use crate::modules::atc_position::models::AtcPosition;

    fn dcl_airports() -> DclAirports {
        DclAirports::try_from_codes(vec!["ZBAA".to_owned()]).unwrap()
    }

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

    #[test]
    fn recognizes_only_dcl_capable_airport_positions() {
        let dcl_airports = dcl_airports();
        for callsign in [
            "ZBAA_TWR",
            "ZBAA_E_TWR",
            "ZBAA_DEL",
            "ZBAA_GND",
            "ZBAA_W_GND",
        ] {
            assert_eq!(
                dcl_airport_code(callsign, &dcl_airports),
                Some("ZBAA"),
                "{callsign}"
            );
        }

        for callsign in [
            "ZBAA_ATIS",
            "ZBAA_RMP",
            "ZBAA_A_GND",
            "ZBAA_AW_GND",
            "ZBAA_AX_GND",
            "ZBAA_APP",
            "ZZZZ_TWR",
        ] {
            assert_eq!(
                dcl_airport_code(callsign, &dcl_airports),
                None,
                "{callsign}"
            );
        }
    }

    #[test]
    fn returns_airport_icao_as_cpdlc_code_for_dcl_positions() {
        let now = Utc::now();
        let dto = AtcPositionDto::try_from_position(
            AtcPosition {
                category: "standard".to_owned(),
                callsign: "ZBAA_TWR".to_owned(),
                is_tier_2: false,
                callsign_zh: None,
                callsign_en: None,
                frequency_khz: 118_500,
                cpdlc_code: None,
                remarks: None,
                created_at: now,
                updated_at: now,
            },
            &dcl_airports(),
        )
        .unwrap();

        assert_eq!(dto.cpdlc_code.as_deref(), Some("ZBAA"));
    }

    #[test]
    fn preserves_stored_cpdlc_code_for_non_dcl_positions() {
        let now = Utc::now();
        let dto = AtcPositionDto::try_from_position(
            AtcPosition {
                category: "standard".to_owned(),
                callsign: "ZBAA_APP".to_owned(),
                is_tier_2: false,
                callsign_zh: None,
                callsign_en: None,
                frequency_khz: 119_000,
                cpdlc_code: Some("BJA".to_owned()),
                remarks: None,
                created_at: now,
                updated_at: now,
            },
            &dcl_airports(),
        )
        .unwrap();

        assert_eq!(dto.cpdlc_code.as_deref(), Some("BJA"));
    }
}
