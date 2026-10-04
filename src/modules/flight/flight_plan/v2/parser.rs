//! Flight route parser
//!
//! ```text
//! route = dep seg* arr
//! dep = IDENTIFIER SPEED_AND_ALTITUDE?
//! arr = IDENTIFIER
//! seg = (IDENTIFIER | IDENTIFIER_REFERENCE | GEO) (VFR | IFR)? | DIRECT
//! ```

use crate::modules::flight::flight_plan::v2::{CruisingLevel, LexerToken, LexerTokenValue, Speed};

pub struct Parser<'s> {
    tokens: Vec<LexerToken<'s>>,
}

#[derive(Clone, PartialEq)]
pub struct Ident<'s> {
    pub ident: &'s str,
    pub amendments: Vec<IdentAmend>,
    pub errors: Vec<ParserIdentError>,
}

impl<'s> Ident<'s> {
    pub fn identifier(&self) -> &'s str {
        self.ident
    }
}

impl<'s> std::fmt::Debug for Ident<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}/{:?}/E={:?}",
            self.ident, self.amendments, self.errors
        ))
    }
}

#[derive(Clone, PartialEq)]
pub enum IdentAmend {
    SpeedAndAltitude {
        speed: Speed,
        altitude: CruisingLevel,
    },
    FlightRuleVfr,
    FlightRuleIfr,
}

impl std::fmt::Debug for IdentAmend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::SpeedAndAltitude { .. } => "SpeedAndAltitude",
                Self::FlightRuleVfr => "FlightRuleVfr",
                Self::FlightRuleIfr => "FlightRuleIfr",
            }
        )?;
        if let Self::SpeedAndAltitude { speed, altitude } = self {
            write!(f, "({:?}, {:?})", speed, altitude)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserIdentError {
    ExpectedDepartureIdentifier,
    ExpectedArrivalIdentifier,
    UnexpectedSpeedAndAltitude,
    UnexpectedFlightRule,
    MissingArrival,
}

impl<'s> Parser<'s> {
    pub fn new(tokens: Vec<LexerToken<'s>>) -> Self {
        Self { tokens }
    }

    /// Parse `dep ident* arr`, retaining invalid tokens with diagnostics.
    /// Empty input yields no idents.
    pub fn parse(self) -> impl Iterator<Item = Ident<'s>> {
        Input {
            tokens: &self.tokens,
        }
        .parse_route()
        .into_iter()
    }
}

/// Immutable input cursor for the recursive-descent parser.
///
/// Two-token lookahead distinguishes the final arrival token from an enroute
/// entry. Each production returns its value and the remaining input.
#[derive(Clone, Copy)]
struct Input<'t, 's> {
    tokens: &'t [LexerToken<'s>],
}

impl<'t, 's> Input<'t, 's> {
    fn peek(self, offset: usize) -> Option<&'t LexerTokenValue<'s>> {
        self.tokens.get(offset).map(|token| &token.value)
    }

    fn advance(self) -> Option<(&'t LexerToken<'s>, Self)> {
        let (token, tokens) = self.tokens.split_first()?;
        Some((token, Self { tokens }))
    }

    // route = dep ident* arr
    fn parse_route(self) -> Vec<Ident<'s>> {
        let Some((departure, input)) = self.parse_departure() else {
            return Vec::new();
        };
        if input.peek(0).is_none() {
            return vec![Ident {
                errors: departure
                    .errors
                    .into_iter()
                    .chain([ParserIdentError::MissingArrival])
                    .collect(),
                ..departure
            }];
        }
        std::iter::once(departure)
            .chain(input.parse_tail())
            .collect()
    }

    // dep = IDENTIFIER SPEED_AND_ALTITUDE?
    fn parse_departure(self) -> Option<(Ident<'s>, Self)> {
        let (token, input) = self.advance()?;
        let valid = matches!(token.value, LexerTokenValue::Identifier);
        let (amendment, input) = if valid {
            input.parse_speed_and_altitude()
        } else {
            (None, input)
        };
        Some((
            Ident {
                ident: token.str,
                amendments: amendment.into_iter().collect(),
                errors: (!valid)
                    .then_some(ParserIdentError::ExpectedDepartureIdentifier)
                    .into_iter()
                    .collect(),
            },
            input,
        ))
    }

    // tail = arr | ident tail
    fn parse_tail(self) -> Vec<Ident<'s>> {
        if self.peek(1).is_none() {
            return self.parse_arrival().into_iter().collect();
        }
        let (ident, input) = self.parse_ident().expect("lookahead is not EOF");
        if input.peek(0).is_none() {
            return vec![Ident {
                errors: ident
                    .errors
                    .into_iter()
                    .chain([ParserIdentError::MissingArrival])
                    .collect(),
                ..ident
            }];
        }
        std::iter::once(ident).chain(input.parse_tail()).collect()
    }

    // arr = IDENTIFIER
    fn parse_arrival(self) -> Option<Ident<'s>> {
        let (token, _) = self.advance()?;
        Some(Ident {
            ident: token.str,
            amendments: Vec::new(),
            errors: (!matches!(token.value, LexerTokenValue::Identifier))
                .then_some(ParserIdentError::ExpectedArrivalIdentifier)
                .into_iter()
                .collect(),
        })
    }

    // ident = (IDENTIFIER | IDENTIFIER_REFERENCE | GEO) (VFR | IFR)? | DIRECT
    // Recovery consumes any other token as an entry so parsing always advances.
    fn parse_ident(self) -> Option<(Ident<'s>, Self)> {
        let (token, input) = self.advance()?;
        let (amendment, input) = match token.value {
            LexerTokenValue::Identifier
            | LexerTokenValue::IdentifierReference { .. }
            | LexerTokenValue::Geo { .. } => input.parse_flight_rule(),
            _ => (None, input),
        };
        let error = match token.value {
            LexerTokenValue::SpeedAndAltitude { .. } => {
                Some(ParserIdentError::UnexpectedSpeedAndAltitude)
            }
            LexerTokenValue::Vfr | LexerTokenValue::Ifr => {
                Some(ParserIdentError::UnexpectedFlightRule)
            }
            _ => None,
        };
        Some((
            Ident {
                ident: token.str,
                amendments: amendment.into_iter().collect(),
                errors: error.into_iter().collect(),
            },
            input,
        ))
    }

    fn parse_speed_and_altitude(self) -> (Option<IdentAmend>, Self) {
        let Some(LexerTokenValue::SpeedAndAltitude { speed, altitude }) = self.peek(0) else {
            return (None, self);
        };
        let (_, input) = self.advance().expect("lookahead is not EOF");
        (
            Some(IdentAmend::SpeedAndAltitude {
                speed: speed.clone(),
                altitude: altitude.clone(),
            }),
            input,
        )
    }

    fn parse_flight_rule(self) -> (Option<IdentAmend>, Self) {
        let amendment = match self.peek(0) {
            Some(LexerTokenValue::Vfr) => IdentAmend::FlightRuleVfr,
            Some(LexerTokenValue::Ifr) => IdentAmend::FlightRuleIfr,
            _ => return (None, self),
        };
        let (_, input) = self.advance().expect("lookahead is not EOF");
        (Some(amendment), input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::flight::flight_plan::v2::lexer::Lexer;

    fn parse(route: &str) -> Vec<Ident<'_>> {
        Parser::new(Lexer::new(route).parse_all().collect())
            .parse()
            .collect()
    }

    #[test]
    fn parses_all_enroute_token_types_in_order() {
        let route = parse("ZBAA VYK VYK180040 38N054E 3806N16730W DCT ZSPD");
        assert_eq!(
            route.iter().map(|ident| ident.ident).collect::<Vec<_>>(),
            [
                "ZBAA",
                "VYK",
                "VYK180040",
                "38N054E",
                "3806N16730W",
                "DCT",
                "ZSPD"
            ]
        );
        assert!(route.iter().all(|ident| ident.errors.is_empty()));
        assert!(route.iter().all(|ident| ident.amendments.is_empty()));
    }

    #[test]
    fn attaches_speed_and_altitude_only_to_departure() {
        let route = parse("ZBAA K0830M0840 DCT ZSPD");
        assert_eq!(route.len(), 3);
        assert_eq!(route[0].ident, "ZBAA");
        assert_eq!(
            route[0].amendments,
            [IdentAmend::SpeedAndAltitude {
                speed: Speed::KmH(830),
                altitude: CruisingLevel::MeterAltitude(840),
            }]
        );
        assert!(route.iter().all(|ident| ident.errors.is_empty()));
        assert!(route[1..].iter().all(|ident| ident.amendments.is_empty()));
    }

    #[test]
    fn requires_identifier_endpoints() {
        for token in ["DCT", "38N054E", "VYK180040", "K0830M0840", "VFR", "IFR"] {
            let input = format!("{token} DCT ZSPD");
            let route = parse(&input);
            assert_eq!(
                route[0].errors,
                [ParserIdentError::ExpectedDepartureIdentifier]
            );
            let input = format!("ZBAA DCT {token}");
            let route = parse(&input);
            assert_eq!(
                route[2].errors,
                [ParserIdentError::ExpectedArrivalIdentifier]
            );
        }
    }

    #[test]
    fn retains_misplaced_and_repeated_speed_and_altitude_with_errors() {
        for input in [
            "ZBAA DCT K0830M0840 ZSPD",
            "ZBAA K0830M0840 DCT K0830M0840 ZSPD",
            "ZBAA K0830M0840 K0830M0840 DCT ZSPD",
        ] {
            let route = parse(input);
            let invalid = route
                .iter()
                .find(|ident| ident.ident == "K0830M0840")
                .unwrap();
            assert_eq!(
                invalid.errors,
                [ParserIdentError::UnexpectedSpeedAndAltitude]
            );
            assert!(invalid.amendments.is_empty());
            assert_eq!(route.last().unwrap().ident, "ZSPD");
        }
    }

    #[test]
    fn attaches_flight_rules_to_points() {
        let route = parse("ZBAA VYK VFR VYK180040 IFR 38N054E VFR ZSPD");
        assert_eq!(
            route.iter().map(|ident| ident.ident).collect::<Vec<_>>(),
            ["ZBAA", "VYK", "VYK180040", "38N054E", "ZSPD"]
        );
        assert_eq!(route[1].amendments, [IdentAmend::FlightRuleVfr]);
        assert_eq!(route[2].amendments, [IdentAmend::FlightRuleIfr]);
        assert_eq!(route[3].amendments, [IdentAmend::FlightRuleVfr]);
        assert!(route.iter().all(|ident| ident.errors.is_empty()));
    }

    #[test]
    fn departure_does_not_accept_flight_rules() {
        for rule in ["VFR", "IFR"] {
            for prefix in ["ZBAA", "ZBAA K0830M0840"] {
                let input = format!("{prefix} {rule} ZSPD");
                let route = parse(&input);
                assert_eq!(route.len(), 3);
                assert_eq!(route[1].ident, rule);
                assert_eq!(route[1].errors, [ParserIdentError::UnexpectedFlightRule]);
                assert!(
                    route[0]
                        .amendments
                        .iter()
                        .all(|amendment| matches!(amendment, IdentAmend::SpeedAndAltitude { .. }))
                );
            }
        }
    }

    #[test]
    fn accepts_at_most_one_flight_rule_per_point() {
        for rules in ["VFR IFR", "IFR VFR", "VFR VFR", "IFR IFR"] {
            let input = format!("ZBAA VYK {rules} ZSPD");
            let route = parse(&input);
            assert_eq!(route.len(), 4);
            assert_eq!(route[1].amendments.len(), 1);
            assert_eq!(route[2].errors, [ParserIdentError::UnexpectedFlightRule]);
            assert!(route[2].amendments.is_empty());
        }
    }

    #[test]
    fn retains_flight_rules_without_a_preceding_point() {
        for rule in ["VFR", "IFR"] {
            let input = format!("ZBAA DCT {rule} ZSPD");
            let route = parse(&input);
            assert_eq!(route.len(), 4);
            assert_eq!(route[2].ident, rule);
            assert_eq!(route[2].errors, [ParserIdentError::UnexpectedFlightRule]);
            assert!(route.iter().all(|ident| ident.amendments.is_empty()));
        }
    }

    #[test]
    fn requires_arrival_after_amended_enroute_point() {
        let route = parse("ZBAA VYK IFR ZSPD");
        assert_eq!(route.len(), 3);
        assert_eq!(route[1].amendments, [IdentAmend::FlightRuleIfr]);
        assert!(route[2].amendments.is_empty());
        assert!(route.iter().all(|ident| ident.errors.is_empty()));

        for point in ["ZSPD", "38N054E", "VYK180040"] {
            for rule in ["VFR", "IFR"] {
                let input = format!("ZBAA {point} {rule}");
                let route = parse(&input);
                assert_eq!(route.len(), 2);
                assert_eq!(route[1].amendments.len(), 1);
                assert_eq!(route[1].errors, [ParserIdentError::MissingArrival]);
            }
        }
    }

    #[test]
    fn accepts_zero_enroute_idents() {
        for input in ["ZBAA ZSPD", "ZBAA K0830M0840 ZSPD"] {
            let route = parse(input);
            assert_eq!(route.len(), 2);
            assert_eq!(route[0].ident, "ZBAA");
            assert_eq!(route[1].ident, "ZSPD");
            assert!(route[1].amendments.is_empty());
            assert!(route.iter().all(|ident| ident.errors.is_empty()));
        }
    }

    #[test]
    fn handles_empty_and_incomplete_routes() {
        assert!(parse("").is_empty());
        for input in ["ZBAA", "ZBAA K0830M0840"] {
            let route = parse(input);
            assert_eq!(route.len(), 1);
            assert_eq!(route[0].errors, [ParserIdentError::MissingArrival]);
        }
    }
}
