use crate::adapter::flight::Flight;
use crate::flight_plan::validator::{
    MessageContainer, WarningMessage, WarningMessageCode, WarningMessageField,
};

pub fn validate_plan(flight: &Flight) -> impl IntoIterator<Item = WarningMessage> {
    MessageContainer::new()
        .validate_plan::<RvsmValidator>(flight)
        .validate_plan::<EquipmentRnav1Validator>(flight)
        .validate_plan::<NavigationPerformanceRnav1Validator>(flight)
        .validate_plan::<RnpArWithoutRfValidator>(flight)
        .validate_plan::<RnpArValidator>(flight)
        .build()
}

impl<T: IntoIterator<Item = WarningMessage>> MessageContainer<T> {
    fn validate_plan<V: FlightValidator>(
        self,
        flight: &Flight,
    ) -> MessageContainer<impl IntoIterator<Item = WarningMessage>> {
        MessageContainer(self.0.into_iter().chain(V::validate_plan(flight)))
    }
}

trait FlightValidator {
    #[must_use]
    fn validate_plan(flight: &Flight) -> impl IntoIterator<Item = WarningMessage>;
}

struct RvsmValidator;

impl FlightValidator for RvsmValidator {
    fn validate_plan(flight: &Flight) -> impl IntoIterator<Item = WarningMessage> {
        (!flight.equipment.contains('W'))
            .then(|| message(WarningMessageField::Equipment, WarningMessageCode::NoRvsm))
    }
}

struct EquipmentRnav1Validator;

impl FlightValidator for EquipmentRnav1Validator {
    fn validate_plan(flight: &Flight) -> impl IntoIterator<Item = WarningMessage> {
        (!flight.equipment.contains('R') && (29000..=41100).contains(&flight.cruising_level))
            .then(|| message(WarningMessageField::Equipment, WarningMessageCode::NoRnav1))
    }
}

struct NavigationPerformanceRnav1Validator;

impl FlightValidator for NavigationPerformanceRnav1Validator {
    fn validate_plan(flight: &Flight) -> impl IntoIterator<Item = WarningMessage> {
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

struct RnpArWithoutRfValidator;

impl FlightValidator for RnpArWithoutRfValidator {
    fn validate_plan(flight: &Flight) -> impl IntoIterator<Item = WarningMessage> {
        flight.navigation_performance.contains("T2").then(|| {
            message(
                WarningMessageField::NavigationPerformance,
                WarningMessageCode::RnpArWithoutRf,
            )
        })
    }
}

struct RnpArValidator;

impl FlightValidator for RnpArValidator {
    fn validate_plan(flight: &Flight) -> impl IntoIterator<Item = WarningMessage> {
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
