use super::fix_record::FixRecord;
use crate::modules::navdata::models::Vhf;
use crate::modules::navdata::service::{NavdataResult, NavdataService};

impl NavdataService {
    pub async fn find_vhf(&self, ident: &str) -> NavdataResult<Vec<Vhf>> {
        let records: Vec<FixRecord> = sqlx::query_as(
            "SELECT icao_code,
                    COALESCE(NULLIF(TRIM(navaid_identifier), ''), dme_ident) AS identifier,
                    COALESCE(navaid_latitude, dme_latitude) AS latitude,
                    COALESCE(navaid_longitude, dme_longitude) AS longitude
             FROM tbl_d_vhfnavaids WHERE navaid_identifier = $1 OR dme_ident = $1
             ORDER BY icao_code, airport_identifier, latitude, longitude",
        )
        .bind(ident)
        .fetch_all(&self.db)
        .await?;
        records.into_iter().map(FixRecord::into_vhf).collect()
    }
}
