use super::{Validator, WarningMessageField};

/// Cruising level must be a China metric RVSM flight level.
pub struct ChinaRvsmLevelValidator;

impl Validator for ChinaRvsmLevelValidator {
    const IDENT: &'static str = "cruising-level-china-rvsm";
    const FIELD: WarningMessageField = WarningMessageField::CruisingLevel;
}

/// Cruising level must meet the departure/arrival pair's restrictions.
pub struct CruisingLevelRestrictionValidator;

impl Validator for CruisingLevelRestrictionValidator {
    const IDENT: &'static str = "cruising-level-restriction";
    const FIELD: WarningMessageField = WarningMessageField::CruisingLevel;
}
