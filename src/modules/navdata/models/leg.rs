use crate::modules::navdata::models::AnyFix;

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLeg {
    pub from: AnyFix,
    pub to: AnyFix,
    pub identifier: Option<String>,
    /// The connection was recovered as an unknown leg rather than navdata.
    pub is_unknown: bool,
    pub kind: LegKind,
    pub direction_restriction: DirectionRestriction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectionRestriction {
    None,
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegKind {
    Airway,
    Sid,
    Star,
    Direct,
}

impl ResolvedLeg {
    pub fn into_reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            identifier: self.identifier,
            is_unknown: self.is_unknown,
            kind: self.kind,
            direction_restriction: match self.direction_restriction {
                DirectionRestriction::None => DirectionRestriction::None,
                DirectionRestriction::Forward => DirectionRestriction::Backward,
                DirectionRestriction::Backward => DirectionRestriction::Forward,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::navdata::models::GeoPoint;

    #[test]
    fn reversal_preserves_kind_and_recovery_status() {
        for kind in [
            LegKind::Airway,
            LegKind::Sid,
            LegKind::Star,
            LegKind::Direct,
        ] {
            for is_unknown in [false, true] {
                let leg = ResolvedLeg {
                    from: GeoPoint::new(30., 110.).into(),
                    to: GeoPoint::new(30., 111.).into(),
                    identifier: Some("A1".to_owned()),
                    is_unknown,
                    kind,
                    direction_restriction: DirectionRestriction::Forward,
                };
                let reversed = leg.clone().into_reversed();
                assert_eq!(reversed.from, leg.to);
                assert_eq!(reversed.to, leg.from);
                assert_eq!(reversed.identifier, leg.identifier);
                assert_eq!(reversed.kind, kind);
                assert_eq!(reversed.is_unknown, is_unknown);
                assert_eq!(
                    reversed.direction_restriction,
                    DirectionRestriction::Backward
                );
                assert_eq!(reversed.into_reversed(), leg);
            }
        }
    }
}
