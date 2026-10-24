use super::*;
use crate::modules::flight::dto::*;
use crate::modules::navdata::models::{AnyFix, ChinaMainlandBoundary, NavProc};

impl RouteParseResults {
    pub(crate) fn observe(
        &mut self,
        step: RouteParseStep<'_, '_>,
        boundary: &ChinaMainlandBoundary,
    ) {
        match step {
            RouteParseStep::Lexed(tokens) => {
                self.lexer = Some(
                    tokens
                        .iter()
                        .map(|token| {
                            let mut amendments = token
                                .amend
                                .as_ref()
                                .map(|amend| match amend {
                                    LexerTokenAmend::SpeedAndAltitude { speed, altitude } => {
                                        speed_level(speed, altitude)
                                    }
                                    LexerTokenAmend::Runway(runway) => RouteAmendment {
                                        kind: "runway".into(),
                                        value: runway.clone(),
                                    },
                                    LexerTokenAmend::Invalid(value) => RouteAmendment {
                                        kind: "invalid".into(),
                                        value: value.clone(),
                                    },
                                })
                                .into_iter()
                                .collect::<Vec<_>>();
                            let kind = match token.value {
                                LexerTokenValue::SpeedAndAltitude {
                                    ref speed,
                                    ref altitude,
                                } => {
                                    amendments.push(speed_level(speed, altitude));
                                    "speed-and-altitude"
                                }
                                LexerTokenValue::Direct => "direct",
                                LexerTokenValue::Vfr => "vfr",
                                LexerTokenValue::Ifr => "ifr",
                                LexerTokenValue::Ident => "identifier",
                                LexerTokenValue::Geo { .. } => "coordinate",
                                LexerTokenValue::IdentRef { .. } => "fix-reference",
                            };
                            let (latitude, longitude) =
                                if let LexerTokenValue::Geo { lat, lon } = token.value {
                                    (Some(lat), Some(lon))
                                } else {
                                    (None, None)
                                };
                            let (reference_identifier, heading, distance_nm) =
                                if let LexerTokenValue::IdentRef {
                                    ident,
                                    heading,
                                    distance,
                                } = token.value
                                {
                                    (Some(ident.to_owned()), Some(heading), Some(distance))
                                } else {
                                    (None, None, None)
                                };
                            RouteLexerToken {
                                text: token.str.into(),
                                kind: kind.into(),
                                amendments,
                                latitude,
                                longitude,
                                reference_identifier,
                                heading,
                                distance_nm,
                            }
                        })
                        .collect(),
                )
            }
            RouteParseStep::Parsed(idents) => {
                self.lex_grouper = Some(idents.iter().map(Into::into).collect())
            }
            RouteParseStep::Candidates(idents) => {
                self.resolver = Some(
                    idents
                        .iter()
                        .map(|ident| RouteResolvedIdent {
                            ident: (&ident.ident).into(),
                            candidates: ident.candidates.iter().map(Into::into).collect(),
                        })
                        .collect(),
                )
            }
            RouteParseStep::Solved(idents) => {
                self.solver = Some(
                    idents
                        .iter()
                        .enumerate()
                        .map(|(index, ident)| RouteSolvedIdent {
                            ident: (&ident.ident).into(),
                            candidates: ident
                                .candidates
                                .iter()
                                .map(|candidate| candidate.snapshot(index))
                                .collect(),
                        })
                        .collect(),
                )
            }
            RouteParseStep::Constructed(legs) => {
                self.constructor = Some(
                    legs.iter()
                        .map(|entry| RouteConstructedLeg {
                            leg: FlightV2Leg::new(&entry.leg, boundary),
                            procedure: entry.procedure.as_ref().map(procedure_candidate),
                        })
                        .collect(),
                )
            }
            RouteParseStep::Expanded(legs) => {
                self.expander = Some(
                    legs.iter()
                        .map(|leg| FlightV2Leg::new(leg, boundary))
                        .collect(),
                )
            }
            RouteParseStep::PreferredRoutes(_) => {}
        }
    }
}

impl From<&Ident<'_>> for RouteParsedIdent {
    fn from(ident: &Ident<'_>) -> Self {
        Self {
            identifier: ident.ident.into(),
            amendments: ident
                .amendments
                .iter()
                .map(|amend| match amend {
                    IdentAmend::SpeedAndAltitude { speed, altitude } => {
                        speed_level(speed, altitude)
                    }
                    IdentAmend::Runway(runway) => RouteAmendment {
                        kind: "runway".into(),
                        value: runway.clone(),
                    },
                    IdentAmend::FlightRuleVfr => RouteAmendment {
                        kind: "flight-rules".into(),
                        value: "VFR".into(),
                    },
                    IdentAmend::FlightRuleIfr => RouteAmendment {
                        kind: "flight-rules".into(),
                        value: "IFR".into(),
                    },
                })
                .collect(),
            errors: ident
                .errors
                .iter()
                .map(|error| format!("{error:?}"))
                .collect(),
        }
    }
}

impl From<&IdentCandidate> for RouteCandidate {
    fn from(candidate: &IdentCandidate) -> Self {
        match candidate {
            IdentCandidate::Fix(fix) => {
                let dto = FlightRouteFix::from(fix);
                let kind = match fix {
                    AnyFix::Airport(_) => "airport",
                    AnyFix::GeoPoint(_) => "coordinate",
                    AnyFix::Ndb(_) => "ndb",
                    AnyFix::Vhf(_) => "vhf",
                    AnyFix::Waypoint(_) => "waypoint",
                    AnyFix::FixReference(_) => "fix-reference",
                    AnyFix::Unknown(_) => "unknown-fix",
                };
                Self {
                    kind: kind.into(),
                    identifier: dto.identifier.clone(),
                    is_unknown: fix.is_unknown(),
                    fix: Some(dto),
                }
            }
            IdentCandidate::Leg(proc) => procedure_candidate(proc),
        }
    }
}

fn procedure_candidate(proc: &NavProc) -> RouteCandidate {
    RouteCandidate {
        kind: serde_plain::to_string(&proc.kind()).expect("enum serializes"),
        identifier: proc.identifier().into(),
        is_unknown: proc.is_unknown(),
        fix: None,
    }
}

fn speed_level(speed: &Speed, altitude: &CruisingLevel) -> RouteAmendment {
    let speed = match speed {
        Speed::KmH(value) => format!("K{value:04}"),
        Speed::NMileH(value) => format!("N{value:04}"),
        Speed::Mach(value) => format!("M{value:03}"),
    };
    let altitude = match altitude {
        CruisingLevel::MeterAltitude(value) => format!("M{value:04}"),
        CruisingLevel::MeterLevel(value) => format!("S{value:04}"),
        CruisingLevel::FeetAltitude(value) => format!("A{value:03}"),
        CruisingLevel::FeetLevel(value) => format!("F{value:03}"),
    };
    RouteAmendment {
        kind: "speed-and-altitude".into(),
        value: format!("{speed}{altitude}"),
    }
}
