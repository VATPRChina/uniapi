use crate::modules::navdata::models::Sid;
use crate::modules::navdata::service::{FindProcedureMode, NavdataResult, NavdataService};

impl NavdataService {
    /// List all valid SID procedures for an airport.
    pub async fn find_sids_by_airport(&self, airport: &str) -> NavdataResult<Vec<Sid>> {
        Ok(self
            .find_procedure_by_airport(airport, FindProcedureMode::Sid)
            .await?
            .into_iter()
            .map(|proc| Sid { proc })
            .collect())
    }

    pub async fn find_sids_by_ident(&self, ident: &str) -> NavdataResult<Vec<Sid>> {
        Ok(self
            .find_procedure_by_ident(ident, FindProcedureMode::Sid)
            .await?
            .into_iter()
            .map(|proc| Sid { proc })
            .collect())
    }
}
