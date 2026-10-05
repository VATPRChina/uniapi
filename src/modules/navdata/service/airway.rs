use super::EnrouteAirwayRecord;
use crate::modules::navdata::models::Airway;
use crate::modules::navdata::service::{NavdataResult, NavdataService};
use itertools::Itertools;

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
