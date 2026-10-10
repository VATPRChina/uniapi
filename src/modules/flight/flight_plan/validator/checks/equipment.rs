use super::{ValidationContext, Validator, WarningMessageField};
use crate::modules::flight::flight_plan::validator::{WarningMessage, WarningMessageCode};
use crate::modules::flight::models::{ValidatorResult, ValidatorStatus};
use crate::modules::navdata::service::NavdataService;

/// Equipment must permit RVSM when cruising at a China metric RVSM flight level.
pub struct EquipmentRvsmValidator;

impl Validator for EquipmentRvsmValidator {
    const IDENT: &'static str = "equipment-rvsm";
    const FIELD: WarningMessageField = WarningMessageField::Equipment;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        let level = context.flight.cruising_level;
        let (status, warnings) = if level <= 0 {
            (ValidatorStatus::Unavailable, Vec::new())
        } else if !(29100..=41100).contains(&level) {
            (ValidatorStatus::Suppressed, Vec::new())
        } else if context.flight.equipment.trim().is_empty() {
            (ValidatorStatus::Unavailable, Vec::new())
        } else if context.flight.equipment.to_ascii_uppercase().contains('W') {
            (ValidatorStatus::Pass, Vec::new())
        } else {
            (
                ValidatorStatus::Rejected,
                vec![WarningMessage::new(Self::FIELD, WarningMessageCode::NoRvsm)],
            )
        };
        ValidatorResult {
            validator_ident: Self::IDENT.to_owned(),
            field: Self::FIELD,
            status,
            warnings,
        }
    }
}

/// Equipment must support RNAV 1.
pub struct EquipmentRnav1Validator;

impl Validator for EquipmentRnav1Validator {
    const IDENT: &'static str = "equipment-rnav1";
    const FIELD: WarningMessageField = WarningMessageField::Equipment;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        let equipment = context.flight.equipment.trim().to_ascii_uppercase();
        let pbn = context
            .flight
            .navigation_performance
            .trim()
            .to_ascii_uppercase();

        let mut warnings = Vec::new();

        if !equipment.contains('R') {
            warnings.push(WarningMessage::new(
                Self::FIELD,
                WarningMessageCode::NoRnav1,
            ));
        }

        if !pbn.contains("D1") && !pbn.contains("D2") && !pbn.contains("D3") && !pbn.contains("D4")
        {
            warnings.push(WarningMessage::new(
                WarningMessageField::NavigationPerformance,
                WarningMessageCode::NoRnav1,
            ));
        }

        ValidatorResult {
            validator_ident: Self::IDENT.to_owned(),
            field: Self::FIELD,
            status: if warnings.is_empty() {
                ValidatorStatus::Pass
            } else {
                ValidatorStatus::Rejected
            },
            warnings,
        }
    }
}

/// Equipment must support RNP with authorization required (RNP AR).
pub struct EquipmentRnpArValidator;

impl Validator for EquipmentRnpArValidator {
    const IDENT: &'static str = "equipment-rnp-ar";
    const FIELD: WarningMessageField = WarningMessageField::NavigationPerformance;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        let pbn = &context.flight.navigation_performance;
        // T1 includes RF capability; prefer it when both AR codes are declared.
        let code = if pbn.contains("T1") {
            Some(WarningMessageCode::RnpAr)
        } else if pbn.contains("T2") {
            Some(WarningMessageCode::RnpArWithoutRf)
        } else {
            None
        };
        ValidatorResult {
            validator_ident: Self::IDENT.to_owned(),
            field: Self::FIELD,
            status: if code.is_some() {
                ValidatorStatus::Pass
            } else {
                ValidatorStatus::Suppressed
            },
            warnings: code
                .map(|code| WarningMessage::new(Self::FIELD, code))
                .into_iter()
                .collect(),
        }
    }
}
