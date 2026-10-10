use super::Validator;

/// Route must match a predefined preferred route.
pub struct PreferredRouteValidator;

impl Validator for PreferredRouteValidator {
    const IDENT: &'static str = "route-preferred";
}

/// Route must not contain direct legs.
pub struct RouteDirectLegValidator;

impl Validator for RouteDirectLegValidator {
    const IDENT: &'static str = "route-direct-leg";
}

/// Route must not contain unknown legs or fixes.
pub struct RouteUnknownLegOrFixValidator;

impl Validator for RouteUnknownLegOrFixValidator {
    const IDENT: &'static str = "route-unknown-leg-or-fix";
}

/// Route must not contain airways requiring approval.
pub struct AirwayApprovalValidator;

impl Validator for AirwayApprovalValidator {
    const IDENT: &'static str = "route-airway-approval";
}

/// Route must comply with each airway's direction restriction.
pub struct AirwayDirectionValidator;

impl Validator for AirwayDirectionValidator {
    const IDENT: &'static str = "route-airway-direction";
}
