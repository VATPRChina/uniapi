use std::str::FromStr;

pub struct Lexer<'r> {
    route: &'r str,
}

impl<'r> Lexer<'r> {
    pub fn new(route: &'r str) -> Self {
        Self { route }
    }

    pub fn parse_all(&self) -> impl Iterator<Item = LexerToken<'r>> {
        self.route
            .split([' ', '\t', '\n', '\r'])
            .filter(|seg| !seg.is_empty())
            .map(|seg| {
                let seg = seg.split('/').next().unwrap_or_default();
                SpeedAndAltitudeTokenHandler::handle_segment(seg)
                    .or_else(|| DctTokenHandler::handle_segment(seg))
                    .or_else(|| VfrTokenHandler::handle_segment(seg))
                    .or_else(|| IfrTokenHandler::handle_segment(seg))
                    .or_else(|| Geo11TokenHandler::handle_segment(seg))
                    .or_else(|| Geo7TokenHandler::handle_segment(seg))
                    .or_else(|| IdentifierReferenceTokenHandler::handle_segment(seg))
                    .unwrap_or_else(|| LexerToken::new(seg, LexerTokenValue::Identifier))
            })
    }
}

#[derive(Debug, PartialEq)]
pub struct LexerToken<'r> {
    pub(super) str: &'r str,
    pub(super) value: LexerTokenValue<'r>,
    amend: Option<LexerTokenAmend>,
}

impl<'r> LexerToken<'r> {
    pub fn new(str: &'r str, value: LexerTokenValue<'r>) -> Self {
        Self {
            str,
            value,
            amend: None,
        }
    }

    pub fn new_amend(str: &'r str, value: LexerTokenValue<'r>, amend: LexerTokenAmend) -> Self {
        Self {
            str,
            value,
            amend: Some(amend),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LexerTokenValue<'s> {
    SpeedAndAltitude {
        speed: Speed,
        altitude: CruisingLevel,
    },
    Direct,
    Vfr,
    Ifr,
    Identifier,
    Geo {
        lat: f64,
        lon: f64,
    },
    IdentifierReference {
        ident: &'s str,
        heading: u16,
        distance: u16,
    },
}

#[derive(Debug, PartialEq)]
pub enum LexerTokenAmend {}

trait TokenHandler {
    fn handle_segment<'r>(token: &'r str) -> Option<LexerToken<'r>>;
}

struct SpeedAndAltitudeTokenHandler;

impl TokenHandler for SpeedAndAltitudeTokenHandler {
    fn handle_segment<'r>(token: &'r str) -> Option<LexerToken<'r>> {
        let speed_len = Speed::predict_len(token);
        if speed_len == 0 {
            return None;
        }
        let Ok(speed) = Speed::from_str(token.get(..speed_len)?) else {
            return None;
        };

        let altitude_text = token.get(speed_len..)?;
        let altitude_len = CruisingLevel::predict_len(altitude_text);
        if altitude_len == 0 {
            return None;
        }
        let Ok(altitude) = CruisingLevel::from_str(altitude_text) else {
            return None;
        };

        Some(LexerToken::new(
            token,
            LexerTokenValue::SpeedAndAltitude { speed, altitude },
        ))
    }
}

#[cfg(test)]
#[test]
fn test_speed_and_altitude_token_handler() {
    assert_eq!(
        SpeedAndAltitudeTokenHandler::handle_segment("K0830M0840"),
        Some(LexerToken {
            str: "K0830M0840",
            value: LexerTokenValue::SpeedAndAltitude {
                speed: Speed::KmH(830),
                altitude: CruisingLevel::MeterAltitude(840),
            },
            amend: None,
        })
    );
    assert_eq!(SpeedAndAltitudeTokenHandler::handle_segment("P123"), None);
    for identifier in ["NLG", "ML", "KWE", "KBOS", "K", "N", "M", "M中"] {
        assert_eq!(
            SpeedAndAltitudeTokenHandler::handle_segment(identifier),
            None
        );
        assert!(matches!(
            Lexer::new(identifier).parse_all().next().unwrap().value,
            LexerTokenValue::Identifier
        ));
    }
}

struct DctTokenHandler;

impl TokenHandler for DctTokenHandler {
    fn handle_segment<'r>(token: &'r str) -> Option<LexerToken<'r>> {
        if token != "DCT" {
            return None;
        }
        Some(LexerToken::new(token, LexerTokenValue::Direct))
    }
}

#[cfg(test)]
#[test]
fn test_dct_token_handler() {
    assert_eq!(
        DctTokenHandler::handle_segment("DCT"),
        Some(LexerToken {
            str: "DCT",
            value: LexerTokenValue::Direct,
            amend: None,
        })
    );
    assert_eq!(DctTokenHandler::handle_segment("K0830M0840"), None);
}

struct VfrTokenHandler;

impl TokenHandler for VfrTokenHandler {
    fn handle_segment<'r>(token: &'r str) -> Option<LexerToken<'r>> {
        if token != "VFR" {
            return None;
        }
        Some(LexerToken::new(token, LexerTokenValue::Vfr))
    }
}

#[cfg(test)]
#[test]
fn test_vfr_token_handler() {
    assert_eq!(
        VfrTokenHandler::handle_segment("VFR"),
        Some(LexerToken {
            str: "VFR",
            value: LexerTokenValue::Vfr,
            amend: None,
        })
    );
    assert_eq!(VfrTokenHandler::handle_segment("K0830M0840"), None);
}

struct IfrTokenHandler;

impl TokenHandler for IfrTokenHandler {
    fn handle_segment<'r>(token: &'r str) -> Option<LexerToken<'r>> {
        if token != "IFR" {
            return None;
        }
        Some(LexerToken::new(token, LexerTokenValue::Ifr))
    }
}

#[cfg(test)]
#[test]
fn test_ifr_token_handler() {
    assert_eq!(
        IfrTokenHandler::handle_segment("IFR"),
        Some(LexerToken {
            str: "IFR",
            value: LexerTokenValue::Ifr,
            amend: None,
        })
    );
    assert_eq!(DctTokenHandler::handle_segment("K0830M0840"), None);
}

struct Geo7TokenHandler;

impl TokenHandler for Geo7TokenHandler {
    fn handle_segment<'r>(token: &'r str) -> Option<LexerToken<'r>> {
        if token.len() != 7 {
            return None;
        }
        let bytes = token.as_bytes();
        if !matches!(bytes[2], b'N' | b'S') || !matches!(bytes[6], b'E' | b'W') {
            return None;
        }
        let Ok(lat) = token[..2].parse::<f64>() else {
            return None;
        };
        let Ok(lon) = token[3..6].parse::<f64>() else {
            return None;
        };
        let lat = lat * if bytes[2] == b'N' { 1.0 } else { -1.0 };
        let lon = lon * if bytes[6] == b'E' { 1.0 } else { -1.0 };
        Some(LexerToken::new(token, LexerTokenValue::Geo { lat, lon }))
    }
}

#[cfg(test)]
#[test]
fn test_geo7_token_handler() {
    assert_eq!(
        Geo7TokenHandler::handle_segment("38N054E"),
        Some(LexerToken {
            str: "38N054E",
            value: LexerTokenValue::Geo { lat: 38., lon: 54. },
            amend: None,
        })
    );
    assert_eq!(Geo7TokenHandler::handle_segment("K0830M0840"), None);
}

struct Geo11TokenHandler;

impl TokenHandler for Geo11TokenHandler {
    fn handle_segment<'r>(token: &'r str) -> Option<LexerToken<'r>> {
        if token.len() != 11 {
            return None;
        }
        let bytes = token.as_bytes();
        if !matches!(bytes[4], b'N' | b'S') || !matches!(bytes[10], b'E' | b'W') {
            return None;
        }
        let Ok(lat_deg) = token[..2].parse::<f64>() else {
            return None;
        };
        let Ok(lat_min) = token[2..4].parse::<f64>() else {
            return None;
        };
        let Ok(lon_deg) = token[5..8].parse::<f64>() else {
            return None;
        };
        let Ok(lon_min) = token[8..10].parse::<f64>() else {
            return None;
        };
        let lat = (lat_deg + lat_min / 60.0) * if bytes[4] == b'N' { 1.0 } else { -1.0 };
        let lon = (lon_deg + lon_min / 60.0) * if bytes[10] == b'E' { 1.0 } else { -1.0 };
        Some(LexerToken::new(token, LexerTokenValue::Geo { lat, lon }))
    }
}

#[cfg(test)]
#[test]
fn test_geo11_token_handler() {
    assert_eq!(
        Geo11TokenHandler::handle_segment("3806N16730W"),
        Some(LexerToken {
            str: "3806N16730W",
            value: LexerTokenValue::Geo {
                lat: 38.1,
                lon: -167.5
            },
            amend: None,
        })
    );
    assert_eq!(Geo11TokenHandler::handle_segment("K0830M0840"), None);
}

struct IdentifierReferenceTokenHandler;

impl TokenHandler for IdentifierReferenceTokenHandler {
    fn handle_segment<'r>(token: &'r str) -> Option<LexerToken<'r>> {
        if token.len() < 8 {
            return None;
        }
        let Ok(heading) = u16::from_str(&token[(token.len() - 6)..(token.len() - 3)]) else {
            return None;
        };
        let Ok(distance) = u16::from_str(&token[(token.len() - 3)..]) else {
            return None;
        };
        Some(LexerToken::new(
            token,
            LexerTokenValue::IdentifierReference {
                ident: &token[..(token.len() - 6)],
                heading,
                distance,
            },
        ))
    }
}

#[cfg(test)]
#[test]
fn test_ident_ref_token_handler() {
    assert_eq!(
        IdentifierReferenceTokenHandler::handle_segment("VYK180040"),
        Some(LexerToken {
            str: "VYK180040",
            value: LexerTokenValue::IdentifierReference {
                ident: "VYK",
                heading: 180,
                distance: 40
            },
            amend: None,
        })
    );
    assert_eq!(IdentifierReferenceTokenHandler::handle_segment("VYK"), None);
}

#[derive(Debug, Clone, PartialEq)]
pub enum Speed {
    KmH(u16),
    NMileH(u16),
    Mach(u16),
}

impl Speed {
    pub fn predict_len(s: &str) -> usize {
        let Some(first_ch) = s.chars().nth(0) else {
            return 0;
        };
        if first_ch == 'N' || first_ch == 'K' {
            5
        } else if first_ch == 'M' {
            4
        } else {
            0
        }
    }
}

impl FromStr for Speed {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().nth(0) {
            Some('K') => {
                if s.len() == 5 {
                    u16::from_str(&s[1..]).map(Speed::KmH).map_err(|_| ())
                } else {
                    Err(())
                }
            }
            Some('N') => {
                if s.len() == 5 {
                    u16::from_str(&s[1..]).map(Speed::NMileH).map_err(|_| ())
                } else {
                    Err(())
                }
            }
            Some('M') => {
                if s.len() == 4 {
                    u16::from_str(&s[1..]).map(Speed::Mach).map_err(|_| ())
                } else {
                    Err(())
                }
            }
            Some(_) => Err(()),
            None => Err(()),
        }
    }
}

#[cfg(test)]
#[test]
fn test_speed_from_str() {
    assert_eq!(Speed::from_str("K0830"), Ok(Speed::KmH(830)));
    assert_eq!(Speed::from_str("N0485"), Ok(Speed::NMileH(485)));
    assert_eq!(Speed::from_str("M082"), Ok(Speed::Mach(82)));
    assert_eq!(Speed::from_str("A1"), Err(()));
    assert_eq!(Speed::from_str("M1"), Err(()));
    assert_eq!(Speed::from_str(""), Err(()));
}

#[derive(Debug, Clone, PartialEq)]
pub enum CruisingLevel {
    MeterAltitude(u16),
    MeterLevel(u16),
    FeetAltitude(u16),
    FeetLevel(u16),
}

impl CruisingLevel {
    pub fn predict_len(s: &str) -> usize {
        let Some(first_ch) = s.chars().nth(0) else {
            return 0;
        };
        if first_ch == 'M' || first_ch == 'S' {
            5
        } else if first_ch == 'A' || first_ch == 'F' {
            4
        } else {
            0
        }
    }
}

impl FromStr for CruisingLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().nth(0) {
            Some('M') => {
                if s.len() == 5 {
                    u16::from_str(&s[1..])
                        .map(CruisingLevel::MeterAltitude)
                        .map_err(|_| ())
                } else {
                    Err(())
                }
            }
            Some('S') => {
                if s.len() == 5 {
                    u16::from_str(&s[1..])
                        .map(CruisingLevel::MeterLevel)
                        .map_err(|_| ())
                } else {
                    Err(())
                }
            }
            Some('A') => {
                if s.len() == 4 {
                    u16::from_str(&s[1..])
                        .map(CruisingLevel::FeetAltitude)
                        .map_err(|_| ())
                } else {
                    Err(())
                }
            }
            Some('F') => {
                if s.len() == 4 {
                    u16::from_str(&s[1..])
                        .map(CruisingLevel::FeetLevel)
                        .map_err(|_| ())
                } else {
                    Err(())
                }
            }
            Some(_) => Err(()),
            None => Err(()),
        }
    }
}

#[cfg(test)]
#[test]
fn test_cruising_level_from_str() {
    assert_eq!(
        CruisingLevel::from_str("M0840"),
        Ok(CruisingLevel::MeterAltitude(840))
    );
    assert_eq!(
        CruisingLevel::from_str("S1130"),
        Ok(CruisingLevel::MeterLevel(1130))
    );
    assert_eq!(
        CruisingLevel::from_str("A045"),
        Ok(CruisingLevel::FeetAltitude(45))
    );
    assert_eq!(
        CruisingLevel::from_str("F330"),
        Ok(CruisingLevel::FeetLevel(330))
    );
    assert_eq!(CruisingLevel::from_str("A1"), Err(()));
    assert_eq!(CruisingLevel::from_str("N1"), Err(()));
    assert_eq!(CruisingLevel::from_str(""), Err(()));
}
