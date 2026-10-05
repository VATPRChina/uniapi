use super::fix_record::FixRecord;
use crate::modules::navdata::models::{Ndb, NdbKind};
use crate::modules::navdata::service::{NavdataResult, NavdataService};

impl NavdataService {
    pub async fn find_enroute_ndb(&self, ident: &str) -> NavdataResult<Vec<Ndb>> {
        let records: Vec<FixRecord> = sqlx::query_as(
            "SELECT icao_code, navaid_identifier AS identifier,
                    navaid_latitude AS latitude, navaid_longitude AS longitude
             FROM tbl_db_enroute_ndbnavaids WHERE navaid_identifier = $1
             ORDER BY icao_code, navaid_latitude, navaid_longitude",
        )
        .bind(ident)
        .fetch_all(&self.db)
        .await?;
        records
            .into_iter()
            .map(|record| record.into_ndb(NdbKind::Enroute))
            .collect()
    }

    pub async fn find_terminal_ndb(&self, ident: &str) -> NavdataResult<Vec<Ndb>> {
        let records: Vec<FixRecord> = sqlx::query_as(
            "SELECT airport_identifier AS icao_code, navaid_identifier AS identifier,
                    navaid_latitude AS latitude, navaid_longitude AS longitude
             FROM tbl_pn_terminal_ndbnavaids WHERE navaid_identifier = $1
             ORDER BY airport_identifier, navaid_latitude, navaid_longitude",
        )
        .bind(ident)
        .fetch_all(&self.db)
        .await?;
        records
            .into_iter()
            .map(|record| record.into_ndb(NdbKind::Terminal))
            .collect()
    }
}
