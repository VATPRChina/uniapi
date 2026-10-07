use crate::modules::navdata::models::Star;
use crate::modules::navdata::service::{FindProcedureMode, NavdataResult, NavdataService};

impl NavdataService {
    pub async fn find_stars_by_ident(&self, ident: &str) -> NavdataResult<Vec<Star>> {
        Ok(self
            .find_procedure_by_ident(ident, FindProcedureMode::Star)
            .await?
            .into_iter()
            .map(|proc| Star { proc })
            .collect())
    }
}
