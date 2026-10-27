use itertools::Itertools;

use super::{CheckResult, ValidationContext, ValidationLevel, Validator};
use crate::modules::flight::models::{
    PreferredRouteDescription, ValidationField, ValidationProblem, ValidatorIdentifier,
    ValidatorStatus,
};
use crate::modules::navdata::models::{AnyFix, DirectionRestriction, Fix, LegKind, ResolvedLeg};
use crate::modules::navdata::service::NavdataService;

/// Route must match a predefined preferred route.
pub struct PreferredRouteValidator;

impl Validator for PreferredRouteValidator {
    const LEVEL: ValidationLevel = ValidationLevel::Segment;
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::PreferredRoute;
    const FIELD: ValidationField = ValidationField::Route;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        let (status, problems) = if context.route.legs.is_empty() {
            (ValidatorStatus::Unavailable, Vec::new())
        } else if let Some(route) = context.preferred_route {
            (
                ValidatorStatus::Pass,
                vec![ValidationProblem::RouteMatchPreferred {
                    from: route.departure.clone(),
                    to: route.arrival.clone(),
                    raw_route: route.raw_route.clone(),
                }],
            )
        } else if context.preferred_routes.is_empty() {
            (ValidatorStatus::Suppressed, Vec::new())
        } else {
            (
                ValidatorStatus::Rejected,
                vec![ValidationProblem::NotPreferredRoute {
                    routes: context
                        .preferred_routes
                        .iter()
                        .sorted_by(|left, right| left.name.cmp(&right.name))
                        .map(|route| PreferredRouteDescription {
                            from: route.departure.clone(),
                            to: route.arrival.clone(),
                            raw_route: route.raw_route.clone(),
                        })
                        .collect(),
                }],
            )
        };
        CheckResult {
            id: Self::IDENT,
            field: Self::FIELD,
            status,
            problems,
        }
    }
}

pub struct RouteDirectLegValidator;

impl Validator for RouteDirectLegValidator {
    const LEVEL: ValidationLevel = ValidationLevel::Leg;
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::RouteDirectLeg;
    const FIELD: ValidationField = ValidationField::Route;

    fn validate(context: &ValidationContext<'_>, navdata: &NavdataService) -> CheckResult {
        validate_legs::<Self>(context, |leg| {
            (leg.kind == LegKind::Direct
                && !matches!(leg.from, AnyFix::Airport(_))
                && !matches!(leg.to, AnyFix::Airport(_))
                && leg.from.is_china_or_unknown(navdata.china_boundaries())
                && leg.to.is_china_or_unknown(navdata.china_boundaries()))
            .then(|| ValidationProblem::RouteDirectSegment {
                from: ident(&leg.from),
                to: ident(&leg.to),
            })
        })
    }
}

pub struct RouteUnknownLegOrFixValidator;

impl Validator for RouteUnknownLegOrFixValidator {
    const LEVEL: ValidationLevel = ValidationLevel::Leg;
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::RouteUnknownLegOrFix;
    const FIELD: ValidationField = ValidationField::Route;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        validate_legs::<Self>(context, |leg| {
            if leg.is_unknown {
                Some(ValidationProblem::RouteUnknownLegOrFix {
                    ident: leg.identifier.clone().unwrap_or_else(|| "DCT".into()),
                })
            } else if leg.from.is_unknown() {
                Some(ValidationProblem::RouteUnknownLegOrFix {
                    ident: ident(&leg.from),
                })
            } else if leg.to.is_unknown() {
                Some(ValidationProblem::RouteUnknownLegOrFix {
                    ident: ident(&leg.to),
                })
            } else {
                None
            }
        })
    }
}

pub struct AirwayApprovalValidator;

impl Validator for AirwayApprovalValidator {
    const LEVEL: ValidationLevel = ValidationLevel::Leg;
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::AirwayApproval;
    const FIELD: ValidationField = ValidationField::Route;

    fn validate(context: &ValidationContext<'_>, navdata: &NavdataService) -> CheckResult {
        validate_legs::<Self>(context, |leg| {
            (leg.kind == LegKind::Airway
                && leg.identifier.as_deref().is_some_and(|ident| {
                    let ident = ident.to_ascii_uppercase();
                    (ident.starts_with('V') || ident.starts_with('X'))
                        && (leg.from.is_china_or_unknown(navdata.china_boundaries())
                            || leg.to.is_china_or_unknown(navdata.china_boundaries()))
                }))
            .then(|| ValidationProblem::AirwayRequireApproval {
                from: ident(&leg.from),
                ident: leg.identifier.clone().unwrap_or_default(),
                to: ident(&leg.to),
            })
        })
    }
}

pub struct AirwayDirectionValidator;

impl Validator for AirwayDirectionValidator {
    const LEVEL: ValidationLevel = ValidationLevel::Leg;
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::AirwayDirection;
    const FIELD: ValidationField = ValidationField::Route;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        validate_legs::<Self>(context, |leg| {
            (leg.kind == LegKind::Airway
                && leg.identifier.is_some()
                && leg.direction_restriction == DirectionRestriction::Backward)
                .then(|| ValidationProblem::RouteLegDirection {
                    from: ident(&leg.from),
                    ident: leg.identifier.clone().unwrap_or_default(),
                    to: ident(&leg.to),
                })
        })
    }
}

fn ident(fix: &AnyFix) -> String {
    fix.identifier().map(str::to_owned).unwrap_or_else(|| {
        fix.position()
            .map(|(lat, lon)| format!("{lat},{lon}"))
            .unwrap_or_default()
    })
}

fn validate_legs<V: Validator>(
    context: &ValidationContext<'_>,
    problem: impl Fn(&ResolvedLeg) -> Option<ValidationProblem>,
) -> CheckResult {
    let (status, problems) = if context.route.legs.is_empty() {
        (ValidatorStatus::Unavailable, Vec::new())
    } else if context.preferred_route.is_some() {
        (ValidatorStatus::Suppressed, Vec::new())
    } else {
        let problems = context
            .route
            .legs
            .iter()
            .filter_map(problem)
            .collect::<Vec<_>>();
        (
            if problems.is_empty() {
                ValidatorStatus::Pass
            } else {
                ValidatorStatus::Rejected
            },
            problems,
        )
    };
    CheckResult {
        id: V::IDENT,
        field: V::FIELD,
        status,
        problems,
    }
}
