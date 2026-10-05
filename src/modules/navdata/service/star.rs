use crate::modules::navdata::models::Star;
use crate::modules::navdata::service::{NavdataResult, NavdataService};

impl NavdataService {
    pub async fn find_stars(&self, ident: &str) -> NavdataResult<Vec<Star>> {
        super::procedure::find_common_procedures(&self.db, ident, true)
            .await?
            .into_iter()
            .map(|(airport, legs)| {
                Ok(Star {
                    airport: airport.as_str().try_into()?,
                    identifier: ident.try_into()?,
                    legs,
                })
            })
            .collect()
    }
}
