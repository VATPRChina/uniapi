use sqlx::{PgPool, Row};

use crate::flight_plan::{Fix, FixKind};

pub async fn find(db: &PgPool, ident: &str) -> Result<Vec<Fix>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT icao_code, identifier, latitude, longitude
        FROM navdata.waypoint
        WHERE identifier = $1
        "#,
    )
    .bind(ident)
    .fetch_all(db)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|row| {
                Fix::identified(
                    FixKind::Waypoint,
                    row.get::<String, _>("icao_code"),
                    row.get::<String, _>("identifier"),
                    row.get("latitude"),
                    row.get("longitude"),
                )
            })
            .collect()
    })
}
