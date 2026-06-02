use crate::flight_plan::validator::{
    MessageContainer, Validator, WarningMessage, WarningMessageCode,
};
use crate::model::navdata::{AnyFix, DirectionRestriction, ResolvedLeg};

type LegContext<'a> = (usize, &'a ResolvedLeg);

pub struct LegValidator;

impl<'a> Validator<LegContext<'a>> for LegValidator {
    fn validate((index, leg): LegContext<'a>) -> impl IntoIterator<Item = WarningMessage> {
        MessageContainer::new()
            .validate::<RouteLegDirectionValidator, _>((index, leg))
            .validate::<AirwayRequireApprovalValidator, _>((index, leg))
            .validate::<RouteDirectSegmentValidator, _>((index, leg))
            .build()
    }
}

pub struct RouteLegDirectionValidator;

impl<'a> Validator<LegContext<'a>> for RouteLegDirectionValidator {
    fn validate((index, leg): LegContext<'a>) -> impl IntoIterator<Item = WarningMessage> {
        (leg.identifier.is_some() && leg.direction_restriction == DirectionRestriction::Backward)
            .then(|| WarningMessage::route_indexed(index, WarningMessageCode::RouteLegDirection))
    }
}

pub struct AirwayRequireApprovalValidator;

impl<'a> Validator<LegContext<'a>> for AirwayRequireApprovalValidator {
    fn validate((index, leg): LegContext<'a>) -> impl IntoIterator<Item = WarningMessage> {
        leg.identifier
            .as_deref()
            .filter(|ident| {
                let from_zh = leg
                    .from
                    .identifier()
                    .is_some_and(|identifier| identifier.starts_with('Z'));
                let to_zh = leg
                    .to
                    .identifier()
                    .is_some_and(|identifier| identifier.starts_with('Z'));

                from_zh && to_zh && (ident.starts_with('V') || ident.starts_with('X'))
            })
            .map(|_| {
                WarningMessage::route_indexed(index, WarningMessageCode::AirwayRequireApproval)
            })
    }
}

pub struct RouteDirectSegmentValidator;

impl<'a> Validator<LegContext<'a>> for RouteDirectSegmentValidator {
    fn validate((index, leg): LegContext<'a>) -> impl IntoIterator<Item = WarningMessage> {
        let is_from_zh = leg
            .from
            .identifier()
            .is_none_or(|identifier| identifier.starts_with('Z'));
        let is_to_zh = leg
            .to
            .identifier()
            .is_none_or(|identifier| identifier.starts_with('Z'));

        (leg.identifier.is_none()
            && !matches!(leg.from, AnyFix::Airport { .. })
            && !matches!(leg.to, AnyFix::Airport { .. })
            && is_from_zh
            && is_to_zh)
            .then(|| WarningMessage::route_indexed(index, WarningMessageCode::RouteDirectSegment))
    }
}
