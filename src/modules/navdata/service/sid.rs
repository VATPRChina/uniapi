use crate::modules::navdata::models::Sid;
use crate::modules::navdata::service::{FindProcedureMode, NavdataResult, NavdataService};

impl NavdataService {
    pub async fn find_sids_by_ident(&self, ident: &str) -> NavdataResult<Vec<Sid>> {
        Ok(self
            .find_procedure_by_ident(ident, FindProcedureMode::Sid)
            .await?
            .into_iter()
            .map(|proc| Sid { proc })
            .collect())
    }
}
