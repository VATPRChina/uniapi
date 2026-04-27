use sqlx::{PgPool, Row};

use crate::flight_plan::{Fix, FixKind};

pub async fn find(db: &PgPool, ident: &str) -> Result<Option<Fix>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT identifier, latitude, longitude
        FROM navdata.airport
        WHERE identifier = $1
        "#,
    )
    .bind(ident)
    .fetch_optional(db)
    .await
    .map(|row| {
        row.map(|row| {
            Fix::identified(
                FixKind::Airport,
                "",
                row.get::<String, _>("identifier"),
                row.get("latitude"),
                row.get("longitude"),
            )
        })
    })
}
