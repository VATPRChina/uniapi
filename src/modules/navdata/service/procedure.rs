use itertools::Itertools;
use sqlx::{FromRow, SqlitePool};

use super::{InvalidNavdataError, NavdataResult};
use crate::modules::navdata::models::{
    Airport, AnyFix, DirectionRestriction, GeoPoint, LegKind, NdbKind, ResolvedLeg, WaypointKind,
};

/// Return all airport-scoped matches, including procedures with no common legs.
pub(super) async fn find_common_procedures(
    db: &SqlitePool,
    ident: &str,
    is_star: bool,
) -> NavdataResult<Vec<(String, Vec<ResolvedLeg>)>> {
    // DFD v2: conventional/RNAV/FMS common routes, plus STAR profile descent.
    // https://developers.navigraph.com/docs/navigation-data/dfd-data-format-v2
    let table = if is_star {
        "tbl_pe_stars"
    } else {
        "tbl_pd_sids"
    };

    let records: Vec<ProcedureRecord> = sqlx::query_as(&format!(
        "SELECT airport_identifier AS airport, route_type,
                COALESCE(transition_identifier, '') AS transition,
                waypoint_identifier AS identifier, waypoint_icao_code AS icao_code,
                waypoint_ref_table AS ref_table, waypoint_latitude AS latitude,
                waypoint_longitude AS longitude
         FROM {table} WHERE procedure_identifier = $1
         ORDER BY airport_identifier, route_type, transition_identifier, seqno",
    ))
    .bind(ident)
    .fetch_all(db)
    .await?;
    let records = records
        .into_iter()
        .into_group_map_by(|record| record.airport.clone());
    records
        .into_iter()
        .map(|(airport, fixes)| {
            let fixes = fixes
                .iter()
                .filter(|f| {
                    if is_star {
                        ["2", "5", "8", "M"].contains(&f.route_type.as_str())
                    } else {
                        ["2", "5", "M"].contains(&f.route_type.as_str())
                    }
                })
                .map(ProcedureRecord::to_fix)
                .collect::<NavdataResult<Vec<_>>>()?;
            let legs = fixes
                .windows(2)
                .filter_map(|pair| {
                    let [from, to] = dbg!(pair) else {
                        return None;
                    };
                    Some(ResolvedLeg {
                        from: from.clone(),
                        to: to.clone(),
                        identifier: Some(ident.to_owned()),
                        is_unknown: false,
                        kind: if is_star { LegKind::Star } else { LegKind::Sid },
                        direction_restriction: DirectionRestriction::None,
                    })
                })
                .collect();
            Ok((airport, legs))
        })
        .collect()
}

#[derive(FromRow)]
struct ProcedureRecord {
    airport: String,
    route_type: String,
    #[allow(unused)]
    transition: String,
    identifier: Option<String>,
    icao_code: Option<String>,
    ref_table: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

impl ProcedureRecord {
    fn to_fix(&self) -> NavdataResult<AnyFix> {
        let Some(identifier) = self.identifier.as_deref().filter(|s| !s.trim().is_empty()) else {
            return Err(InvalidNavdataError::InternalError(
                "record missing fix identifier",
            ));
        };
        let record = super::fix_record::FixRecord {
            icao_code: self.icao_code.clone().unwrap_or_default(),
            identifier: identifier.to_owned(),
            latitude: self.latitude,
            longitude: self.longitude,
        };
        let fix = match self.ref_table.as_deref().unwrap_or("").trim() {
            "PA" => {
                let (latitude, longitude) = record.position()?;
                AnyFix::Airport(Airport {
                    identifier: identifier.try_into()?,
                    latitude,
                    longitude,
                })
            }
            "D" => AnyFix::Vhf(record.into_vhf()?),
            "DB" => AnyFix::Ndb(record.into_ndb(NdbKind::Enroute)?),
            "PN" => AnyFix::Ndb(record.into_ndb(NdbKind::Terminal)?),
            "EA" => AnyFix::Waypoint(record.into_waypoint(WaypointKind::Enroute)?),
            "PC" => AnyFix::Waypoint(record.into_waypoint(WaypointKind::Terminal)?),
            "PG" => {
                let (latitude, longitude) = record.position()?;
                GeoPoint::new(latitude, longitude).into()
            }
            _ => {
                return Err(InvalidNavdataError::InternalError(
                    "unsupported procedure waypoint reference table",
                ));
            }
        };
        Ok(fix)
    }
}
