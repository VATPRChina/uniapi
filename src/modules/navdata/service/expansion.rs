use super::*;
use crate::modules::navdata::models::{GeoPoint, ProcedureKind, ProcedureSegment};

impl NavdataService {
    /// Load published adjacent airway segments without joining separate areas
    /// or crossing a published end marker. Invalid records propagate to callers.
    pub async fn list_airway_segments(&self, identifier: &str) -> NavdataResult<Vec<ResolvedLeg>> {
        let records: Vec<EnrouteAirwayRecord> = sqlx::query_as(
            "SELECT area_code, COALESCE(direction_restriction, '') AS direction_restriction,
                    icao_code, route_identifier, seqno, waypoint_description_code,
                    waypoint_identifier, waypoint_latitude, waypoint_longitude, waypoint_ref_table
             FROM tbl_er_enroute_airways WHERE route_identifier = $1 ORDER BY area_code, seqno",
        )
        .bind(identifier)
        .fetch_all(&self.db)
        .await?;
        Ok(records
            .iter()
            .tuple_windows()
            .filter(|(from, to)| from.area_code == to.area_code)
            .map(|(from, to)| to.to_leg(from))
            .collect::<NavdataResult<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect())
    }

    pub async fn list_procedure_segments(
        &self,
        kind: ProcedureKind,
        airport: &str,
        identifier: &str,
    ) -> NavdataResult<Vec<ProcedureSegment>> {
        let table = match kind {
            ProcedureKind::Sid => "tbl_pd_sids",
            ProcedureKind::Star => "tbl_pe_stars",
        };
        let records: Vec<ProcedureRecord> = sqlx::query_as(&format!(
            "SELECT route_type, COALESCE(transition_identifier, '') AS transition,
                    waypoint_identifier AS identifier, waypoint_icao_code AS icao_code,
                    waypoint_ref_table AS ref_table, waypoint_latitude AS latitude,
                    waypoint_longitude AS longitude
             FROM {table} WHERE airport_identifier = $1 AND procedure_identifier = $2
             ORDER BY route_type, transition_identifier, seqno"
        ))
        .bind(airport)
        .bind(identifier)
        .fetch_all(&self.db)
        .await?;
        records
            .into_iter()
            .chunk_by(|r| (r.route_type.clone(), r.transition.clone()))
            .into_iter()
            .map(|((route_type, transition), records)| {
                let fixes = records
                    .map(ProcedureRecord::into_fix)
                    .collect::<NavdataResult<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect();
                Ok(ProcedureSegment {
                    route_type,
                    transition,
                    fixes,
                })
            })
            .collect()
    }
}

#[derive(FromRow)]
struct ProcedureRecord {
    route_type: String,
    transition: String,
    identifier: Option<String>,
    icao_code: Option<String>,
    ref_table: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

impl ProcedureRecord {
    fn into_fix(self) -> NavdataResult<Option<AnyFix>> {
        // Vector/altitude terminations have no published fixed endpoint.
        let Some(identifier) = self.identifier.filter(|s| !s.trim().is_empty()) else {
            return Ok(None);
        };
        let latitude = self
            .latitude
            .ok_or(InvalidNavdataError::InvalidNavaidNullLatLong)?;
        let longitude = self
            .longitude
            .ok_or(InvalidNavdataError::InvalidNavaidNullLatLong)?;
        let icao_code = self.icao_code.as_deref().unwrap_or("").try_into()?;
        let fix = match self.ref_table.as_deref().unwrap_or("").trim() {
            "D" => AnyFix::Vhf(Vhf {
                icao_code,
                identifier: identifier.as_str().try_into()?,
                latitude,
                longitude,
            }),
            "DB" | "PN" => AnyFix::Ndb(Ndb {
                icao_code,
                identifier: identifier.as_str().try_into()?,
                latitude,
                longitude,
                kind: if self.ref_table.as_deref().unwrap_or("").trim() == "DB" {
                    NdbKind::Enroute
                } else {
                    NdbKind::Terminal
                },
            }),
            "EA" | "PC" => AnyFix::Waypoint(Waypoint {
                icao_code,
                identifier: identifier.as_str().try_into()?,
                latitude,
                longitude,
                kind: if self.ref_table.as_deref().unwrap_or("").trim() == "EA" {
                    WaypointKind::Enroute
                } else {
                    WaypointKind::Terminal
                },
            }),
            "PG" => GeoPoint::new(latitude, longitude).into(),
            _ => {
                return Err(InvalidNavdataError::InternalError(
                    "unsupported procedure waypoint reference table",
                ));
            }
        };
        Ok(Some(fix))
    }
}
