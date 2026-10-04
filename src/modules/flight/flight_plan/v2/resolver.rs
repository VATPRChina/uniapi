use arrayvec::ArrayString;

use futures::{StreamExt, TryStreamExt, stream};
use sqlx::FromRow;

use crate::modules::{
    flight::flight_plan::v2::{Ident, Lexer, LexerTokenValue},
    navdata::{
        models::GeoPoint,
        service::{InvalidNavdataError, NavdataResult, NavdataService},
    },
};

pub struct CandidateResolver<'s> {
    idents: Vec<Ident<'s>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct IdentWithCandidate<'s> {
    pub(super) ident: Ident<'s>,
    pub(super) candidates: Vec<IdentCandidate>,
}

pub type IcaoCode = ArrayString<2>;
pub type AirportIdentifier = ArrayString<4>;

/// An identifier can denote either a physical fix or a connecting leg.
#[derive(Debug, Clone, PartialEq)]
pub enum IdentCandidate {
    Fix(FixCandidate),
    Leg(LegCandidate),
}

impl IdentCandidate {
    pub fn is_unknown(&self) -> bool {
        match self {
            IdentCandidate::Fix(fix_candidate) => fix_candidate.is_unknown(),
            IdentCandidate::Leg(leg_candidate) => leg_candidate.is_unknown(),
        }
    }
}

/// Physical-point interpretations, retaining their coordinates and scope.
#[derive(Debug, Clone, PartialEq)]
pub enum FixCandidate {
    Airport {
        airport: AirportIdentifier,
        lat: f64,
        lon: f64,
    },
    EnrouteWaypoint {
        icao_code: IcaoCode,
        lat: f64,
        lon: f64,
    },
    TerminalWaypoint {
        airport: AirportIdentifier,
        lat: f64,
        lon: f64,
    },
    EnrouteVor {
        icao_code: IcaoCode,
        lat: f64,
        lon: f64,
    },
    TerminalVor {
        airport: AirportIdentifier,
        lat: f64,
        lon: f64,
    },
    EnrouteNdb {
        icao_code: IcaoCode,
        lat: f64,
        lon: f64,
    },
    TerminalNdb {
        airport: AirportIdentifier,
        lat: f64,
        lon: f64,
    },
    Geo {
        lat: f64,
        lon: f64,
    },
    UnknownWaypoint,
}

impl FixCandidate {
    pub fn latitude(&self) -> Option<f64> {
        match self {
            FixCandidate::Airport { lat, .. }
            | FixCandidate::EnrouteWaypoint { lat, .. }
            | FixCandidate::TerminalWaypoint { lat, .. }
            | FixCandidate::EnrouteVor { lat, .. }
            | FixCandidate::TerminalVor { lat, .. }
            | FixCandidate::EnrouteNdb { lat, .. }
            | FixCandidate::TerminalNdb { lat, .. }
            | FixCandidate::Geo { lat, .. } => Some(*lat),
            FixCandidate::UnknownWaypoint => None,
        }
    }

    pub fn longitude(&self) -> Option<f64> {
        match self {
            FixCandidate::Airport { lon, .. }
            | FixCandidate::EnrouteWaypoint { lon, .. }
            | FixCandidate::TerminalWaypoint { lon, .. }
            | FixCandidate::EnrouteVor { lon, .. }
            | FixCandidate::TerminalVor { lon, .. }
            | FixCandidate::EnrouteNdb { lon, .. }
            | FixCandidate::TerminalNdb { lon, .. }
            | FixCandidate::Geo { lon, .. } => Some(*lon),
            FixCandidate::UnknownWaypoint => None,
        }
    }

    pub fn position(&self) -> Option<(f64, f64)> {
        match self {
            FixCandidate::Airport { lat, lon, .. }
            | FixCandidate::EnrouteWaypoint { lat, lon, .. }
            | FixCandidate::TerminalWaypoint { lat, lon, .. }
            | FixCandidate::EnrouteVor { lat, lon, .. }
            | FixCandidate::TerminalVor { lat, lon, .. }
            | FixCandidate::EnrouteNdb { lat, lon, .. }
            | FixCandidate::TerminalNdb { lat, lon, .. }
            | FixCandidate::Geo { lat, lon, .. } => Some((*lat, *lon)),
            FixCandidate::UnknownWaypoint => None,
        }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, FixCandidate::UnknownWaypoint)
    }
}

/// Connection interpretations, completed by subsequent fix entries.
#[derive(Debug, Clone, PartialEq)]
pub enum LegCandidate {
    Direct,
    Airway,
    Sid { airport: AirportIdentifier },
    Star { airport: AirportIdentifier },
    UnknownAirway,
    UnknownSid,
    UnknownStar,
}

impl LegCandidate {
    pub fn is_unknown(&self) -> bool {
        matches!(
            self,
            LegCandidate::UnknownAirway | LegCandidate::UnknownSid | LegCandidate::UnknownStar
        )
    }
}

    }
}

impl<'s> CandidateResolver<'s> {
    pub fn new(idents: Vec<Ident<'s>>) -> Self {
        Self { idents }
    }

    /// Resolve all candidates in route order without choosing between ambiguous matches.
    /// Plain identifiers always include unknown route/point fallbacks.
    /// Database failures propagate instead of being treated as unknown identifiers.
    ///
    /// # Panics
    /// Panics if a standalone speed/altitude or flight-rule token reaches resolution.
    pub async fn resolve_candidates(
        self,
        navdata: &NavdataService,
    ) -> NavdataResult<impl Iterator<Item = IdentWithCandidate<'s>>> {
        let resolved: Vec<_> = stream::iter(self.idents)
            .then(|ident| async move {
                let candidates = resolve_ident(navdata, ident.identifier()).await?;
                Ok::<_, InvalidNavdataError>(IdentWithCandidate { ident, candidates })
            })
            .try_collect()
            .await?;
        Ok(resolved.into_iter())
    }
}

async fn resolve_ident(
    navdata: &NavdataService,
    ident: &str,
) -> NavdataResult<Vec<IdentCandidate>> {
    let Some(token) = Lexer::new(ident).parse_all().next() else {
        return Ok(Vec::new());
    };
    match token.value {
        LexerTokenValue::Direct => Ok(vec![IdentCandidate::Leg(LegCandidate::Direct)]),
        LexerTokenValue::Geo { lat, lon } => {
            Ok(vec![IdentCandidate::Fix(FixCandidate::Geo { lat, lon })])
        }
        LexerTokenValue::IdentifierReference {
            ident,
            heading,
            distance,
        } => {
            let fixes = find_fix_candidates(navdata, ident, Some((heading, distance))).await?;
            if fixes.is_empty() {
                Ok(vec![IdentCandidate::Fix(FixCandidate::UnknownWaypoint)])
            } else {
                Ok(fixes.into_iter().map(IdentCandidate::Fix).collect())
            }
        }

        LexerTokenValue::Identifier => {
            let (routes, fixes) = tokio::try_join!(
                find_route_candidates(navdata, ident),
                find_fix_candidates(navdata, ident, None),
            )?;
            Ok(routes
                .into_iter()
                .map(IdentCandidate::Leg)
                .chain(fixes.into_iter().map(IdentCandidate::Fix))
                .chain([
                    IdentCandidate::Leg(LegCandidate::UnknownSid),
                    IdentCandidate::Leg(LegCandidate::UnknownStar),
                    IdentCandidate::Leg(LegCandidate::UnknownAirway),
                    IdentCandidate::Fix(FixCandidate::UnknownWaypoint),
                ])
                .collect())
        }
        LexerTokenValue::SpeedAndAltitude { .. } | LexerTokenValue::Vfr | LexerTokenValue::Ifr => {
            panic!("standalone amendment token reached candidate resolution: {ident}")
        }
    }
}

async fn find_route_candidates(
    navdata: &NavdataService,
    ident: &str,
) -> NavdataResult<Vec<LegCandidate>> {
    let (airway, sids, stars) = tokio::try_join!(
        navdata.exists_airway(ident),
        navdata.list_sid_airports(ident),
        navdata.list_star_airports(ident),
    )?;
    airway
        .then_some(Ok(LegCandidate::Airway))
        .into_iter()
        .chain(sids.into_iter().map(|airport| -> NavdataResult<_> {
            Ok(LegCandidate::Sid {
                airport: airport.as_str().try_into()?,
            })
        }))
        .chain(stars.into_iter().map(|airport| -> NavdataResult<_> {
            Ok(LegCandidate::Star {
                airport: airport.as_str().try_into()?,
            })
        }))
        .collect()
}

#[derive(FromRow)]
struct FixCandidateRecord {
    kind: String,
    icao_code: Option<String>,
    airport_identifier: Option<String>,
    identifier: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

impl FixCandidateRecord {
    fn into_candidate(self, reference: Option<(u16, u16)>) -> NavdataResult<FixCandidate> {
        let lat = self
            .latitude
            .ok_or(InvalidNavdataError::InvalidNavaidNullLatLong)?;
        let lon = self
            .longitude
            .ok_or(InvalidNavdataError::InvalidNavaidNullLatLong)?;
        // Keep the base candidate's type and scope, but locate it at the referenced fix.
        let point = GeoPoint::new(lat, lon);
        let point = match reference {
            Some((heading, distance)) => point.destination(heading, distance),
            None => point,
        };
        let lat = point.latitude;
        let lon = point.longitude;
        let airport = self
            .airport_identifier
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let icao_code = || -> NavdataResult<IcaoCode> {
            Ok(self
                .icao_code
                .as_deref()
                .ok_or(InvalidNavdataError::InternalError("missing ICAO code"))?
                .try_into()?)
        };
        let terminal_airport = || -> NavdataResult<AirportIdentifier> {
            Ok(airport
                .ok_or(InvalidNavdataError::InternalError(
                    "missing terminal airport",
                ))?
                .try_into()?)
        };
        Ok(match self.kind.as_str() {
            "tbl_pa_airports" => FixCandidate::Airport {
                airport: self.identifier.as_str().try_into()?,
                lat,
                lon,
            },
            "tbl_ea_enroute_waypoints" => FixCandidate::EnrouteWaypoint {
                icao_code: icao_code()?,
                lat,
                lon,
            },
            "tbl_pc_terminal_waypoints" => FixCandidate::TerminalWaypoint {
                airport: terminal_airport()?,
                lat,
                lon,
            },
            "tbl_d_vhfnavaids" => match airport {
                Some(_) => FixCandidate::TerminalVor {
                    airport: terminal_airport()?,
                    lat,
                    lon,
                },
                None => FixCandidate::EnrouteVor {
                    icao_code: icao_code()?,
                    lat,
                    lon,
                },
            },
            "tbl_db_enroute_ndbnavaids" => FixCandidate::EnrouteNdb {
                icao_code: icao_code()?,
                lat,
                lon,
            },
            "tbl_pn_terminal_ndbnavaids" => FixCandidate::TerminalNdb {
                airport: terminal_airport()?,
                lat,
                lon,
            },
            _ => return Err(InvalidNavdataError::InternalError("unknown candidate kind")),
        })
    }
}

// Preserve terminal airport scope, which the shared AnyFix VHF model cannot carry.
async fn find_fix_candidates(
    navdata: &NavdataService,
    ident: &str,
    reference: Option<(u16, u16)>,
) -> NavdataResult<Vec<FixCandidate>> {
    let records: Vec<FixCandidateRecord> = sqlx::query_as(
        r#"
        SELECT * FROM (
            SELECT
                'tbl_pa_airports' AS kind,
                icao_code AS icao_code,
                NULL AS airport_identifier,
                airport_identifier AS identifier,
                airport_ref_latitude AS latitude,
                airport_ref_longitude AS longitude
            FROM tbl_pa_airports
            UNION
            SELECT
                'tbl_d_vhfnavaids' AS kind,
                icao_code AS icao_code,
                airport_identifier AS airport_identifier,
                coalesce(navaid_identifier, dme_ident) AS identifier,
                coalesce(navaid_latitude, dme_latitude) AS latitude,
                coalesce(navaid_longitude, dme_longitude) AS longitude
            FROM tbl_d_vhfnavaids
            UNION
            SELECT
                'tbl_db_enroute_ndbnavaids' AS kind,
                icao_code AS icao_code,
                NULL AS airport_identifier,
                navaid_identifier AS identifier,
                navaid_latitude AS latitude,
                navaid_longitude AS longitude
            FROM tbl_db_enroute_ndbnavaids
            UNION
            SELECT
                'tbl_pn_terminal_ndbnavaids' AS kind,
                icao_code AS icao_code,
                airport_identifier AS airport_identifier,
                navaid_identifier AS identifier,
                navaid_latitude AS latitude,
                navaid_longitude AS longitude
            FROM tbl_pn_terminal_ndbnavaids
            UNION
            SELECT
                'tbl_ea_enroute_waypoints' AS kind,
                icao_code AS icao_code,
                NULL AS airport_identifier,
                waypoint_identifier AS identifier,
                waypoint_latitude AS latitude,
                waypoint_longitude AS longitude
            FROM tbl_ea_enroute_waypoints
            UNION
            SELECT
                'tbl_pc_terminal_waypoints' AS kind,
                icao_code AS icao_code,
                region_code AS airport_identifier,
                waypoint_identifier AS identifier,
                waypoint_latitude AS latitude,
                waypoint_longitude AS longitude
            FROM tbl_pc_terminal_waypoints
        ) WHERE identifier = $1
        ORDER BY kind, icao_code, airport_identifier, latitude, longitude
    "#,
    )
    .bind(ident)
    .fetch_all(&navdata.db)
    .await?;
    records
        .into_iter()
        .map(|record| record.into_candidate(reference))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::flight::flight_plan::v2::Parser;

    async fn navdata() -> NavdataService {
        NavdataService::with_preferred_routes_path(
            "data/navdata.db?mode=ro",
            "assets/test/routes.csv",
        )
        .await
        .unwrap()
    }

    async fn resolve<'s>(navdata: &NavdataService, route: &'s str) -> Vec<IdentWithCandidate<'s>> {
        CandidateResolver::new(
            Parser::new(Lexer::new(route).parse_all().collect())
                .parse()
                .collect(),
        )
        .resolve_candidates(navdata)
        .await
        .unwrap()
        .collect()
    }

    #[tokio::test]
    async fn resolves_fix_types_and_preserves_terminal_airports() {
        let navdata = navdata().await;
        let route = resolve(&navdata, "MBAC DOM AC07F ICDO GD AGNOD MBAC").await;
        assert!(
            route[0].candidates.iter().any(|candidate| matches!(candidate, IdentCandidate::Fix(FixCandidate::Airport { airport, .. }) if airport == "MBAC"))
        );
        assert_eq!(route[1].candidates.len(), 4);
        assert!(route[1].candidates.iter().any(|candidate| matches!(candidate, IdentCandidate::Fix(FixCandidate::EnrouteVor { icao_code, lat, lon }) if icao_code == "TD" && lat.is_finite() && lon.is_finite())));
        assert!(route[1].candidates.iter().any(|candidate| matches!(
            candidate,
            IdentCandidate::Fix(FixCandidate::EnrouteNdb { .. })
        )));
        assert!(route[2].candidates.iter().any(|candidate| matches!(candidate, IdentCandidate::Fix(FixCandidate::TerminalWaypoint { airport, .. }) if airport == "MBAC")));
        assert!(route[3].candidates.iter().any(|candidate| matches!(candidate, IdentCandidate::Fix(FixCandidate::TerminalVor { airport, .. }) if airport == "MDSD")));
        assert!(route[4].candidates.iter().any(|candidate| matches!(candidate, IdentCandidate::Fix(FixCandidate::TerminalNdb { airport, .. }) if airport == "MMGL")));
        assert!(route[5].candidates.iter().any(|candidate| matches!(candidate, IdentCandidate::Fix(FixCandidate::EnrouteWaypoint { icao_code, .. }) if icao_code == "MB")));
    }

    #[tokio::test]
    async fn preserves_ambiguous_procedures_and_airways() {
        let navdata = navdata().await;
        let route = resolve(&navdata, "MBAC L453 GTK2A ANTE2D MBAC").await;
        assert!(
            route[1]
                .candidates
                .contains(&IdentCandidate::Leg(LegCandidate::Airway))
        );
        assert!(
            route[2]
                .candidates
                .contains(&IdentCandidate::Leg(LegCandidate::Sid {
                    airport: "MBAC".try_into().unwrap()
                }))
        );
        assert!(
            route[3]
                .candidates
                .contains(&IdentCandidate::Leg(LegCandidate::Sid {
                    airport: "MMUN".try_into().unwrap()
                }))
        );
        assert!(
            route[3]
                .candidates
                .contains(&IdentCandidate::Leg(LegCandidate::Star {
                    airport: "MDLR".try_into().unwrap()
                }))
        );
    }

    #[tokio::test]
    async fn resolves_geo_direct_and_all_reference_bases() {
        let navdata = navdata().await;
        let route = resolve(&navdata, "MBAC DCT 38N054E 3806N16730W DOM180040 MBAC").await;
        assert_eq!(
            route[1].candidates,
            [IdentCandidate::Leg(LegCandidate::Direct)]
        );
        assert_eq!(
            route[2].candidates,
            [IdentCandidate::Fix(FixCandidate::Geo {
                lat: 38.0,
                lon: 54.0
            })]
        );
        assert_eq!(
            route[3].candidates,
            [IdentCandidate::Fix(FixCandidate::Geo {
                lat: 38.1,
                lon: -167.5
            })]
        );
        assert_eq!(route[4].candidates.len(), 2);
        assert!(route[4].candidates.iter().all(|candidate| matches!(
            candidate,
            IdentCandidate::Fix(FixCandidate::EnrouteVor { .. })
                | IdentCandidate::Fix(FixCandidate::EnrouteNdb { .. })
        )));
        assert_eq!(route[4].ident.identifier(), "DOM180040");
    }

    #[tokio::test]
    async fn reference_offsets_preserve_candidate_types_and_scopes() {
        let navdata = navdata().await;
        for name in ["MBAC", "DOM", "AC07F", "ICDO", "GD", "AGNOD"] {
            let input = format!("MBAC {name} {name}180040 {name}090000 MBAC");
            let route = resolve(&navdata, &input).await;
            let bases: Vec<_> = route[1]
                .candidates
                .iter()
                .filter(|candidate| {
                    !matches!(
                        candidate,
                        IdentCandidate::Leg(LegCandidate::UnknownAirway)
                            | IdentCandidate::Fix(FixCandidate::UnknownWaypoint)
                    )
                })
                .collect();
            assert!(!bases.is_empty());
            assert_eq!(bases, route[3].candidates.iter().collect::<Vec<_>>());
            assert_eq!(bases.len(), route[2].candidates.len());
            for (base, offset) in bases.into_iter().zip(&route[2].candidates) {
                let IdentCandidate::Fix(base_fix) = base else {
                    panic!("expected a base fix");
                };
                let IdentCandidate::Fix(offset_fix) = offset else {
                    panic!("expected an offset fix");
                };
                assert_eq!(
                    std::mem::discriminant(base_fix),
                    std::mem::discriminant(offset_fix)
                );
                let (scope, lat, lon) = fix_fields(base);
                let (offset_scope, offset_lat, offset_lon) = fix_fields(offset);
                assert_eq!(scope, offset_scope);
                // Due south: latitude changes by angular distance, longitude stays fixed.
                approx::assert_abs_diff_eq!(
                    offset_lat,
                    lat - (40.0_f64 * 1852.0 / 6_371_008.8).to_degrees(),
                    epsilon = 1e-9
                );
                approx::assert_abs_diff_eq!(offset_lon, lon, epsilon = 1e-9);
            }
        }
    }

    fn fix_fields(candidate: &IdentCandidate) -> (&str, f64, f64) {
        match candidate {
            IdentCandidate::Fix(FixCandidate::Airport { airport, lat, lon })
            | IdentCandidate::Fix(FixCandidate::TerminalWaypoint { airport, lat, lon })
            | IdentCandidate::Fix(FixCandidate::TerminalVor { airport, lat, lon })
            | IdentCandidate::Fix(FixCandidate::TerminalNdb { airport, lat, lon }) => {
                (airport.as_str(), *lat, *lon)
            }
            IdentCandidate::Fix(FixCandidate::EnrouteWaypoint {
                icao_code,
                lat,
                lon,
            })
            | IdentCandidate::Fix(FixCandidate::EnrouteVor {
                icao_code,
                lat,
                lon,
            })
            | IdentCandidate::Fix(FixCandidate::EnrouteNdb {
                icao_code,
                lat,
                lon,
            }) => (icao_code.as_str(), *lat, *lon),
            _ => panic!("expected a navigation fix"),
        }
    }

    #[tokio::test]
    async fn retains_unknowns_and_parser_output() {
        let navdata = navdata().await;
        let input = "ZZZZ K0830M0840 UNKNOWN VFR DCT UNKNOWN180040 ZZZZ";
        let expected: Vec<_> = Parser::new(Lexer::new(input).parse_all().collect())
            .parse()
            .collect();
        let route = resolve(&navdata, input).await;
        assert_eq!(
            route.iter().map(|entry| &entry.ident).collect::<Vec<_>>(),
            expected.iter().collect::<Vec<_>>()
        );
        assert_eq!(
            route[0].candidates,
            [
                IdentCandidate::Leg(LegCandidate::UnknownAirway),
                IdentCandidate::Fix(FixCandidate::UnknownWaypoint)
            ]
        );
        assert_eq!(
            route[1].candidates,
            [
                IdentCandidate::Leg(LegCandidate::UnknownAirway),
                IdentCandidate::Fix(FixCandidate::UnknownWaypoint)
            ]
        );
        assert_eq!(
            route[3].candidates,
            [IdentCandidate::Fix(FixCandidate::UnknownWaypoint)]
        );
    }

    #[tokio::test]
    async fn resolves_identifiers_uniformly_and_always_keeps_fallbacks() {
        let navdata = navdata().await;
        let route = resolve(&navdata, "DOM MBAC DOM L453 MBAC DOM").await;
        assert_eq!(route[0].candidates, route[2].candidates);
        assert_eq!(route[0].candidates, route[5].candidates);
        assert_eq!(route[1].candidates, route[4].candidates);
        for entry in route {
            assert!(entry.candidates.ends_with(&[
                IdentCandidate::Leg(LegCandidate::UnknownAirway),
                IdentCandidate::Fix(FixCandidate::UnknownWaypoint),
            ]));
        }
    }

    #[tokio::test]
    #[should_panic(
        expected = "standalone amendment token reached candidate resolution: K0830M0840"
    )]
    async fn rejects_standalone_speed_and_altitude() {
        let navdata = navdata().await;
        resolve(&navdata, "MBAC DCT K0830M0840 MBAC").await;
    }

    #[tokio::test]
    #[should_panic(expected = "standalone amendment token reached candidate resolution: VFR")]
    async fn rejects_standalone_vfr() {
        let navdata = navdata().await;
        resolve(&navdata, "MBAC DCT VFR MBAC").await;
    }

    #[tokio::test]
    #[should_panic(expected = "standalone amendment token reached candidate resolution: IFR")]
    async fn rejects_standalone_ifr() {
        let navdata = navdata().await;
        resolve(&navdata, "MBAC DCT IFR MBAC").await;
    }

    #[tokio::test]
    async fn propagates_database_errors_but_empty_input_needs_no_database() {
        let navdata = navdata().await;
        navdata.db.close().await;
        assert!(resolve(&navdata, "").await.is_empty());
        let idents = Parser::new(Lexer::new("MBAC MBAC").parse_all().collect())
            .parse()
            .collect();
        assert!(matches!(
            CandidateResolver::new(idents)
                .resolve_candidates(&navdata)
                .await,
            Err(InvalidNavdataError::DatabaseError(_))
        ));
    }

    #[test]
    fn rejects_invalid_fix_records_without_panicking() {
        let record = |icao: &str, latitude| FixCandidateRecord {
            kind: "tbl_ea_enroute_waypoints".into(),
            icao_code: Some(icao.into()),
            airport_identifier: None,
            identifier: "FIX".into(),
            latitude,
            longitude: Some(1.0),
        };
        assert!(matches!(
            record("TOOLONG", Some(1.0)).into_candidate(None),
            Err(InvalidNavdataError::StringTooLong)
        ));
        assert!(matches!(
            record("MB", None).into_candidate(None),
            Err(InvalidNavdataError::InvalidNavaidNullLatLong)
        ));
    }
}
