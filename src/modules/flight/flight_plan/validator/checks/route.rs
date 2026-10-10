use super::{ValidationContext, Validator, WarningMessageField};
use crate::modules::flight::flight_plan::validator::{WarningMessage, WarningMessageCode};
use crate::modules::flight::models::{ValidatorResult, ValidatorStatus};
use crate::modules::navdata::models::{AnyFix, DirectionRestriction, LegKind, ResolvedLeg};
use crate::modules::navdata::service::NavdataService;

/// Route must match a predefined preferred route.
pub struct PreferredRouteValidator;

impl Validator for PreferredRouteValidator {
    const IDENT: &'static str = "route-preferred";
    const FIELD: WarningMessageField = WarningMessageField::Route;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        let (status, warnings) = if context.route.legs.is_empty() {
            (ValidatorStatus::Unavailable, Vec::new())
        } else if let Some(preferred) = context.preferred_route {
            (
                ValidatorStatus::Pass,
                vec![WarningMessage::with_parameter(
                    Self::FIELD,
                    WarningMessageCode::RouteMatchPreferred,
                    if preferred.is_public {
                        &preferred.raw_route
                    } else {
                        &context.flight.raw_route
                    },
                )],
            )
        } else if context.preferred_routes.is_empty() {
            (ValidatorStatus::Suppressed, Vec::new())
        } else {
            let mut public_routes = context
                .preferred_routes
                .iter()
                .filter(|route| route.is_public)
                .collect::<Vec<_>>();
            public_routes.sort_by(|left, right| left.name.cmp(&right.name));
            (
                ValidatorStatus::Rejected,
                vec![WarningMessage::with_parameter(
                    Self::FIELD,
                    WarningMessageCode::NotPreferredRoute,
                    public_routes
                        .iter()
                        .map(|route| route.raw_route.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                )],
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

/// Route must not contain direct legs.
pub struct RouteDirectLegValidator;

impl Validator for RouteDirectLegValidator {
    const IDENT: &'static str = "route-direct-leg";
    const FIELD: WarningMessageField = WarningMessageField::Route;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        validate_legs::<Self>(
            context,
            true,
            WarningMessageCode::RouteDirectSegment,
            |leg| {
                leg.kind == LegKind::Direct
                    && !matches!(leg.from, AnyFix::Airport(_))
                    && !matches!(leg.to, AnyFix::Airport(_))
                    && (leg.from.is_china() && leg.to.is_china())
            },
        )
    }
}

/// Route must not contain unknown legs or fixes.
pub struct RouteUnknownLegOrFixValidator;

impl Validator for RouteUnknownLegOrFixValidator {
    const IDENT: &'static str = "route-unknown-leg-or-fix";
    const FIELD: WarningMessageField = WarningMessageField::Route;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        validate_legs::<Self>(
            context,
            true,
            WarningMessageCode::RouteUnknownLegOrFix,
            |leg| leg.is_unknown || leg.from.is_unknown() || leg.to.is_unknown(),
        )
    }
}

/// Route must not contain airways requiring approval.
pub struct AirwayApprovalValidator;

impl Validator for AirwayApprovalValidator {
    const IDENT: &'static str = "route-airway-approval";
    const FIELD: WarningMessageField = WarningMessageField::Route;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        validate_legs::<Self>(
            context,
            true,
            WarningMessageCode::AirwayRequireApproval,
            |leg| {
                leg.kind == LegKind::Airway
                    && leg.identifier.as_deref().is_some_and(|ident| {
                        let ident = ident.to_ascii_uppercase();
                        (ident.starts_with('V') || ident.starts_with('X'))
                            && (leg.from.is_china() || leg.to.is_china())
                    })
            },
        )
    }
}

/// Route must comply with each airway's direction restriction.
pub struct AirwayDirectionValidator;

impl Validator for AirwayDirectionValidator {
    const IDENT: &'static str = "route-airway-direction";
    const FIELD: WarningMessageField = WarningMessageField::Route;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> ValidatorResult {
        validate_legs::<Self>(
            context,
            true,
            WarningMessageCode::RouteLegDirection,
            |leg| {
                leg.kind == LegKind::Airway
                    && leg.identifier.is_some()
                    && leg.direction_restriction == DirectionRestriction::Backward
            },
        )
    }
}

fn validate_legs<V: Validator>(
    context: &ValidationContext<'_>,
    suppress_preferred: bool,
    code: WarningMessageCode,
    is_invalid: impl Fn(&ResolvedLeg) -> bool,
) -> ValidatorResult {
    let (status, warnings) = if context.route.legs.is_empty() {
        (ValidatorStatus::Unavailable, Vec::new())
    } else if suppress_preferred && context.preferred_route.is_some() {
        (ValidatorStatus::Suppressed, Vec::new())
    } else {
        let warnings = context
            .route
            .legs
            .iter()
            .enumerate()
            .filter(|(_, leg)| is_invalid(leg))
            .map(|(index, leg)| {
                WarningMessage::route_indexed_with_param(index, code, format!("{:?}", leg))
            })
            .collect::<Vec<_>>();
        (
            if warnings.is_empty() {
                ValidatorStatus::Pass
            } else {
                ValidatorStatus::Rejected
            },
            warnings,
        )
    };
    ValidatorResult {
        validator_ident: V::IDENT.to_owned(),
        field: V::FIELD,
        status,
        warnings,
    }
}
