use super::CheckResult;
use super::{ValidationContext, Validator};
use crate::modules::flight::models::ValidatorStatus;
use crate::modules::flight::models::{ValidationField, ValidationProblem, ValidatorIdentifier};
use crate::modules::navdata::service::NavdataService;

/// Equipment must permit RVSM when cruising at a China metric RVSM flight level.
pub struct EquipmentRvsmValidator;

impl Validator for EquipmentRvsmValidator {
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::EquipmentRvsm;
    const FIELD: ValidationField = ValidationField::Equipment;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        let level = context.flight.flight_plan.cruising_level;
        let (status, problems) = if level <= 0 {
            (ValidatorStatus::Unavailable, Vec::new())
        } else if !(29100..=41100).contains(&level) {
            (ValidatorStatus::Suppressed, Vec::new())
        } else if context.flight.flight_plan.equipment.trim().is_empty() {
            (ValidatorStatus::Unavailable, Vec::new())
        } else if context
            .flight
            .flight_plan
            .equipment
            .to_ascii_uppercase()
            .contains('W')
        {
            (ValidatorStatus::Pass, Vec::new())
        } else {
            (ValidatorStatus::Rejected, vec![ValidationProblem::NoRvsm])
        };
        CheckResult {
            id: Self::IDENT,
            field: Self::FIELD,
            status,
            problems,
        }
    }
}

/// Equipment must support RNAV 1.
pub struct EquipmentRnav1Validator;

impl Validator for EquipmentRnav1Validator {
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::EquipmentRnav1;
    const FIELD: ValidationField = ValidationField::Equipment;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        let equipment = context
            .flight
            .flight_plan
            .equipment
            .trim()
            .to_ascii_uppercase();
        let pbn = context
            .flight
            .flight_plan
            .navigation_performance
            .trim()
            .to_ascii_uppercase();

        let problems = if equipment.contains('R')
            && ["D1", "D2", "D3", "D4"]
                .iter()
                .any(|code| pbn.contains(code))
        {
            Vec::new()
        } else {
            vec![ValidationProblem::NoRnav1]
        };

        CheckResult {
            id: Self::IDENT,
            field: Self::FIELD,
            status: if problems.is_empty() {
                ValidatorStatus::Pass
            } else {
                ValidatorStatus::Rejected
            },
            problems,
        }
    }
}

/// Equipment must support RNP with authorization required (RNP AR).
pub struct EquipmentRnpArValidator;

impl Validator for EquipmentRnpArValidator {
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::EquipmentRnpAr;
    const FIELD: ValidationField = ValidationField::NavigationPerformance;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        let pbn = &context.flight.flight_plan.navigation_performance;
        // T1 includes RF capability; prefer it when both AR codes are declared.
        let code = if pbn.contains("T1") {
            Some(ValidationProblem::RnpAr)
        } else if pbn.contains("T2") {
            Some(ValidationProblem::RnpArWithoutRf)
        } else {
            None
        };
        CheckResult {
            id: Self::IDENT,
            field: Self::FIELD,
            status: if code.is_some() {
                ValidatorStatus::Pass
            } else {
                ValidatorStatus::Suppressed
            },
            problems: code.into_iter().collect(),
        }
    }
}
