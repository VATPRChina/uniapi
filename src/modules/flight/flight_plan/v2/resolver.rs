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
#[derive(Clone, PartialEq)]
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

impl std::fmt::Debug for FixCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Airport { airport, lat, lon } => {
                write!(f, "Airport({} @ {:.3}, {:.3})", airport, lat, lon)
            }
            Self::EnrouteWaypoint {
                icao_code,
                lat,
                lon,
            } => write!(f, "EnrouteWaypoint({} @ {:.3}, {:.3})", icao_code, lat, lon),
            Self::TerminalWaypoint { airport, lat, lon } => {
                write!(f, "TerminalWaypoint({} @ {:.3}, {:.3})", airport, lat, lon)
            }
            Self::EnrouteVor {
                icao_code,
                lat,
                lon,
            } => write!(f, "EnrouteVor({} @ {:.3}, {:.3})", icao_code, lat, lon),
            Self::TerminalVor { airport, lat, lon } => {
                write!(f, "TerminalVor({} @ {:.3}, {:.3})", airport, lat, lon)
            }
            Self::EnrouteNdb {
                icao_code,
                lat,
                lon,
            } => write!(f, "EnrouteNdb({} @ {:.3}, {:.3})", icao_code, lat, lon),
            Self::TerminalNdb { airport, lat, lon } => {
                write!(f, "TerminalNdb({} @ {:.3}, {:.3})", airport, lat, lon)
            }
            Self::Geo { lat, lon } => write!(f, "Geo( @ {:.3}, {:.3})", lat, lon),
            Self::UnknownWaypoint => write!(f, "UnknownWaypoint"),
        }
    }
}

/// Connection interpretations, completed by subsequent fix entries.
#[derive(Clone, PartialEq)]
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

impl std::fmt::Debug for LegCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(f, "Direct"),
            Self::Airway => write!(f, "Airway"),
            Self::Sid { airport } => write!(f, "SID({})", airport),
            Self::Star { airport } => write!(f, "STAR({})", airport),
            Self::UnknownAirway => write!(f, "UnknownAirway"),
            Self::UnknownSid => write!(f, "UnknownSid"),
            Self::UnknownStar => write!(f, "UnknownStar"),
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
