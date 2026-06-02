use crate::adapter::flight::Flight;
use crate::flight_plan::validator::{
    Validator, WarningMessage, WarningMessageCode, WarningMessageField,
};

pub struct RvsmValidator;

impl Validator<&Flight> for RvsmValidator {
    fn validate(flight: &Flight) -> impl IntoIterator<Item = WarningMessage> {
        (!flight.equipment.contains('W'))
            .then(|| message(WarningMessageField::Equipment, WarningMessageCode::NoRvsm))
    }
}

pub struct EquipmentRnav1Validator;

impl Validator<&Flight> for EquipmentRnav1Validator {
    fn validate(flight: &Flight) -> impl IntoIterator<Item = WarningMessage> {
        (!flight.equipment.contains('R') && (29000..=41100).contains(&flight.cruising_level))
            .then(|| message(WarningMessageField::Equipment, WarningMessageCode::NoRnav1))
    }
}

pub struct NavigationPerformanceRnav1Validator;

impl Validator<&Flight> for NavigationPerformanceRnav1Validator {
    fn validate(flight: &Flight) -> impl IntoIterator<Item = WarningMessage> {
        (!flight.navigation_performance.contains("D1")
            && !flight.navigation_performance.contains("D2"))
        .then(|| {
            message(
                WarningMessageField::NavigationPerformance,
                WarningMessageCode::NoRnav1,
            )
        })
    }
}

pub struct RnpArWithoutRfValidator;

impl Validator<&Flight> for RnpArWithoutRfValidator {
    fn validate(flight: &Flight) -> impl IntoIterator<Item = WarningMessage> {
        flight.navigation_performance.contains("T2").then(|| {
            message(
                WarningMessageField::NavigationPerformance,
                WarningMessageCode::RnpArWithoutRf,
            )
        })
    }
}

pub struct RnpArValidator;

impl Validator<&Flight> for RnpArValidator {
    fn validate(flight: &Flight) -> impl IntoIterator<Item = WarningMessage> {
        flight.navigation_performance.contains("T1").then(|| {
            message(
                WarningMessageField::NavigationPerformance,
                WarningMessageCode::RnpAr,
            )
        })
    }
}

fn message(field: WarningMessageField, code: WarningMessageCode) -> WarningMessage {
    WarningMessage {
        message_code: code,
        parameter: None,
        field,
        field_index: None,
    }
}
