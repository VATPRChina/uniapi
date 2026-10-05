use super::fix_record::FixRecord;
use crate::modules::navdata::models::{Waypoint, WaypointKind};
use crate::modules::navdata::service::{NavdataResult, NavdataService};

impl NavdataService {
    pub async fn find_enroute_waypoint(&self, ident: &str) -> NavdataResult<Vec<Waypoint>> {
        let records: Vec<FixRecord> = sqlx::query_as(
            "SELECT icao_code, waypoint_identifier AS identifier,
                    waypoint_latitude AS latitude, waypoint_longitude AS longitude
             FROM tbl_ea_enroute_waypoints WHERE waypoint_identifier = $1
             ORDER BY icao_code, waypoint_latitude, waypoint_longitude",
        )
        .bind(ident)
        .fetch_all(&self.db)
        .await?;
        records
            .into_iter()
            .map(|record| record.into_waypoint(WaypointKind::Enroute))
            .collect()
    }

    pub async fn find_terminal_waypoint(&self, ident: &str) -> NavdataResult<Vec<Waypoint>> {
        let records: Vec<FixRecord> = sqlx::query_as(
            "SELECT region_code AS icao_code, waypoint_identifier AS identifier,
                    waypoint_latitude AS latitude, waypoint_longitude AS longitude
             FROM tbl_pc_terminal_waypoints WHERE waypoint_identifier = $1
             ORDER BY region_code, waypoint_latitude, waypoint_longitude",
        )
        .bind(ident)
        .fetch_all(&self.db)
        .await?;
        records
            .into_iter()
            .map(|record| record.into_waypoint(WaypointKind::Terminal))
            .collect()
    }
}
