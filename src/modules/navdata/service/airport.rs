use arrayvec::ArrayString;
use sqlx::prelude::FromRow;
use tracing::instrument;

use crate::modules::navdata::models::Airport;
use crate::modules::navdata::service::{NavdataResult, NavdataService};

impl NavdataService {
    #[instrument(skip(self), fields(ident = %ident))]
    pub async fn find_airport(&self, ident: &str) -> NavdataResult<Option<Airport>> {
        let result: Option<AirportRecord> = sqlx::query_as(
            r#"
            SELECT airport_identifier, airport_ref_latitude, airport_ref_longitude
            FROM tbl_pa_airports
            WHERE airport_identifier = $1;
            "#,
        )
        .bind(ident)
        .fetch_optional(&self.db)
        .await?;
        let airport = result
            .map(|record| -> NavdataResult<Airport> {
                Ok(Airport {
                    identifier: ArrayString::from(&record.airport_identifier)?,
                    latitude: record.airport_ref_latitude,
                    longitude: record.airport_ref_longitude,
                })
            })
            .transpose()?;

        Ok(airport)
    }
}

#[derive(Debug, Clone, FromRow)]
struct AirportRecord {
    airport_identifier: String,
    airport_ref_latitude: f64,
    airport_ref_longitude: f64,
}
