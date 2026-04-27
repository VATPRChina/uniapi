use sqlx::{PgPool, Row};

use crate::flight_plan::{Fix, FixKind};

pub async fn find(db: &PgPool, ident: &str) -> Result<Vec<Fix>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT icao_code, vor_identifier, vor_latitude, vor_longitude
        FROM navdata.vhf_navaid
        WHERE vor_identifier = $1
        "#,
    )
    .bind(ident)
    .fetch_all(db)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|row| {
                Fix::identified(
                    FixKind::VhfNavaid,
                    row.get::<String, _>("icao_code"),
                    row.get::<String, _>("vor_identifier"),
                    row.get("vor_latitude"),
                    row.get("vor_longitude"),
                )
            })
            .collect()
    })
}
