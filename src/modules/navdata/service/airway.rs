use crate::modules::navdata::models::{
    Airway, AnyFix, DirectionRestriction, Ndb, NdbKind, ResolvedLeg, Vhf, Waypoint, WaypointKind,
};
use crate::modules::navdata::service::{InvalidNavdataError, NavdataResult, NavdataService};
use arrayvec::ArrayString;
use itertools::Itertools;
use sqlx::prelude::FromRow;

impl NavdataService {
    pub async fn find_airway(&self, ident: &str) -> NavdataResult<Option<Airway>> {
        let records: Vec<EnrouteAirwayRecord> = sqlx::query_as(
            "SELECT area_code, COALESCE(direction_restriction, '') AS direction_restriction,
                    icao_code, route_identifier, seqno, waypoint_description_code,
                    waypoint_identifier, waypoint_latitude, waypoint_longitude, waypoint_ref_table
             FROM tbl_er_enroute_airways WHERE route_identifier = $1 ORDER BY area_code, seqno",
        )
        .bind(ident)
        .fetch_all(&self.db)
        .await?;
        if records.is_empty() {
            return Ok(None);
        }
        let legs = records
            .iter()
            .tuple_windows()
            .filter(|(from, to)| from.area_code == to.area_code)
            .map(|(from, to)| to.to_leg(from))
            .collect::<NavdataResult<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        Ok(Some(Airway {
            identifier: ident.try_into()?,
            legs,
        }))
    }
}

#[derive(Debug, Clone, FromRow)]
struct EnrouteAirwayRecord {
    #[allow(unused)]
    area_code: String,
    direction_restriction: String,
    icao_code: String,
    route_identifier: String,
    #[allow(unused)]
    seqno: u32,
    waypoint_description_code: String,
    waypoint_identifier: String,
    waypoint_latitude: f64,
    waypoint_longitude: f64,
    waypoint_ref_table: String,
}

impl EnrouteAirwayRecord {
    fn to_leg(&self, prev: &Self) -> NavdataResult<Option<ResolvedLeg>> {
        if prev.waypoint_description_code.chars().nth(1) == Some('E') {
            return Ok(None);
        }

        let leg = ResolvedLeg {
            identifier: Some(self.route_identifier.clone()),
            from: prev.to_fix()?,
            to: self.to_fix()?,
            is_unknown: false,
            is_sid: false,
            is_star: false,
            direction_restriction: match self.direction_restriction.as_str() {
                "F" => DirectionRestriction::Forward,
                "B" => DirectionRestriction::Backward,
                _ => DirectionRestriction::None,
            },
        };
        Ok(Some(leg))
    }

    fn to_fix(&self) -> NavdataResult<AnyFix> {
        let fix = match self.waypoint_ref_table.trim() {
            "EA" => AnyFix::Waypoint(Waypoint {
                icao_code: ArrayString::from(&self.icao_code)?,
                identifier: ArrayString::from(&self.waypoint_identifier)?,
                latitude: self.waypoint_latitude,
                longitude: self.waypoint_longitude,
                kind: WaypointKind::Enroute,
            }),
            "DB" => AnyFix::Ndb(Ndb {
                icao_code: ArrayString::from(&self.icao_code)?,
                identifier: ArrayString::from(&self.waypoint_identifier)?,
                latitude: self.waypoint_latitude,
                longitude: self.waypoint_longitude,
                kind: NdbKind::Enroute,
            }),
            "D" => AnyFix::Vhf(Vhf {
                icao_code: ArrayString::from(&self.icao_code)?,
                identifier: ArrayString::from(&self.waypoint_identifier)?,
                latitude: self.waypoint_latitude,
                longitude: self.waypoint_longitude,
            }),
            _ => {
                return Err(InvalidNavdataError::InternalError(
                    "unsupported airway waypoint reference table",
                ));
            }
        };
        Ok(fix)
    }
}
