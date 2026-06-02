use crate::flight_plan::validator::{
    MessageContainer, WarningMessage, WarningMessageCode, WarningMessageField,
};
use crate::model::navdata::{AnyFix, DirectionRestriction, ResolvedLeg};

pub fn validate_leg(leg: &ResolvedLeg, index: usize) -> impl IntoIterator<Item = WarningMessage> {
    MessageContainer::new()
        .validate_leg::<RouteLegDirectionValidator>(leg, index)
        .validate_leg::<AirwayRequireApprovalValidator>(leg, index)
        .validate_leg::<RouteDirectSegmentValidator>(leg, index)
        .build()
}

impl<T: IntoIterator<Item = WarningMessage>> MessageContainer<T> {
    fn validate_leg<V: LegValidator>(
        self,
        leg: &ResolvedLeg,
        index: usize,
    ) -> MessageContainer<impl IntoIterator<Item = WarningMessage>> {
        MessageContainer(self.0.into_iter().chain(V::validate_leg(leg, index)))
    }
}

trait LegValidator {
    #[must_use]
    fn validate_leg(leg: &ResolvedLeg, index: usize) -> impl IntoIterator<Item = WarningMessage>;
}

struct RouteLegDirectionValidator;

impl LegValidator for RouteLegDirectionValidator {
    fn validate_leg(leg: &ResolvedLeg, index: usize) -> impl IntoIterator<Item = WarningMessage> {
        (leg.identifier.is_some() && leg.direction_restriction == DirectionRestriction::Backward)
            .then(|| route_indexed_message(index, WarningMessageCode::RouteLegDirection))
    }
}

struct AirwayRequireApprovalValidator;

impl LegValidator for AirwayRequireApprovalValidator {
    fn validate_leg(leg: &ResolvedLeg, index: usize) -> impl IntoIterator<Item = WarningMessage> {
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
            .map(|_| route_indexed_message(index, WarningMessageCode::AirwayRequireApproval))
    }
}

struct RouteDirectSegmentValidator;

impl LegValidator for RouteDirectSegmentValidator {
    fn validate_leg(leg: &ResolvedLeg, index: usize) -> impl IntoIterator<Item = WarningMessage> {
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
            .then(|| route_indexed_message(index, WarningMessageCode::RouteDirectSegment))
    }
}

fn route_indexed_message(index: usize, code: WarningMessageCode) -> WarningMessage {
    WarningMessage {
        message_code: code,
        parameter: None,
        field: WarningMessageField::Route,
        field_index: Some(index),
    }
}
