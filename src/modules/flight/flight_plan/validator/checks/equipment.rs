use super::Validator;

/// Equipment must permit RVSM when cruising at a China metric RVSM flight level.
pub struct EquipmentRvsmValidator;

impl Validator for EquipmentRvsmValidator {
    const IDENT: &'static str = "equipment-rvsm";
}

/// Equipment must support RNAV 1.
pub struct EquipmentRnav1Validator;

impl Validator for EquipmentRnav1Validator {
    const IDENT: &'static str = "equipment-rnav1";
}

/// Equipment must support RNP with authorization required (RNP AR).
pub struct EquipmentRnpArValidator;

impl Validator for EquipmentRnpArValidator {
    const IDENT: &'static str = "equipment-rnp-ar";
}

/// Equipment must support RNP without authorization required.
pub struct EquipmentRnpValidator;

impl Validator for EquipmentRnpValidator {
    const IDENT: &'static str = "equipment-rnp";
}
