use super::{Validator, WarningMessageField};

/// Route must match a predefined preferred route.
pub struct PreferredRouteValidator;

impl Validator for PreferredRouteValidator {
    const IDENT: &'static str = "route-preferred";
    const FIELD: WarningMessageField = WarningMessageField::Route;
}

/// Route must not contain direct legs.
pub struct RouteDirectLegValidator;

impl Validator for RouteDirectLegValidator {
    const IDENT: &'static str = "route-direct-leg";
    const FIELD: WarningMessageField = WarningMessageField::Route;
}

/// Route must not contain unknown legs or fixes.
pub struct RouteUnknownLegOrFixValidator;

impl Validator for RouteUnknownLegOrFixValidator {
    const IDENT: &'static str = "route-unknown-leg-or-fix";
    const FIELD: WarningMessageField = WarningMessageField::Route;
}

/// Route must not contain airways requiring approval.
pub struct AirwayApprovalValidator;

impl Validator for AirwayApprovalValidator {
    const IDENT: &'static str = "route-airway-approval";
    const FIELD: WarningMessageField = WarningMessageField::Route;
}

/// Route must comply with each airway's direction restriction.
pub struct AirwayDirectionValidator;

impl Validator for AirwayDirectionValidator {
    const IDENT: &'static str = "route-airway-direction";
    const FIELD: WarningMessageField = WarningMessageField::Route;
}
