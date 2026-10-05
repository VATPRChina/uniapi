use crate::modules::navdata::models::Sid;
use crate::modules::navdata::service::{NavdataResult, NavdataService};

impl NavdataService {
    pub async fn find_sids(&self, ident: &str) -> NavdataResult<Vec<Sid>> {
        super::procedure::find_common_procedures(&self.db, ident, false)
            .await?
            .into_iter()
            .map(|(airport, legs)| {
                Ok(Sid {
                    airport: airport.as_str().try_into()?,
                    identifier: ident.try_into()?,
                    legs,
                })
            })
            .collect()
    }
}
