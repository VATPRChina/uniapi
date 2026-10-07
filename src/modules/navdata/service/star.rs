use crate::modules::navdata::models::Star;
use crate::modules::navdata::service::{FindProcedureMode, NavdataResult, NavdataService};

impl NavdataService {
    /// List all valid STAR procedures for an airport.
    pub async fn find_stars_by_airport(&self, airport: &str) -> NavdataResult<Vec<Star>> {
        Ok(self
            .find_procedure_by_airport(airport, FindProcedureMode::Star)
            .await?
            .into_iter()
            .map(|proc| Star { proc })
            .collect())
    }

    pub async fn find_stars_by_ident(&self, ident: &str) -> NavdataResult<Vec<Star>> {
        Ok(self
            .find_procedure_by_ident(ident, FindProcedureMode::Star)
            .await?
            .into_iter()
            .map(|proc| Star { proc })
            .collect())
    }
}
