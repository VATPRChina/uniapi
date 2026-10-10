use super::Validator;

/// Cruising level must be a China metric RVSM flight level.
pub struct ChinaRvsmLevelValidator;

impl Validator for ChinaRvsmLevelValidator {
    const IDENT: &'static str = "cruising-level-china-rvsm";
}

/// Cruising level must meet the departure/arrival pair's restrictions.
pub struct CruisingLevelRestrictionValidator;

impl Validator for CruisingLevelRestrictionValidator {
    const IDENT: &'static str = "cruising-level-restriction";
}
