use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::flight_plan::{LevelRestrictionType, PreferredRoute};

pub async fn recommended(
    db: &PgPool,
    dep: &str,
    arr: &str,
) -> Result<Vec<PreferredRoute>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, departure, arrival, raw_route, cruising_level_restriction,
               allowed_altitudes, minimal_altitude, remarks, valid_from, valid_until
        FROM navdata.preferred_route
        WHERE departure = $1
          AND arrival = $2
          AND (valid_from IS NULL OR valid_from <= now())
          AND (valid_until IS NULL OR valid_until >= now())
        ORDER BY id
        "#,
    )
    .bind(dep)
    .bind(arr)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| PreferredRoute {
            id: row.get::<Uuid, _>("id"),
            departure: row.get("departure"),
            arrival: row.get("arrival"),
            raw_route: row.get("raw_route"),
            cruising_level_restriction: row
                .get::<String, _>("cruising_level_restriction")
                .parse()
                .unwrap_or(LevelRestrictionType::Standard),
            allowed_altitudes: row.get::<Vec<i32>, _>("allowed_altitudes"),
            minimal_altitude: row.get("minimal_altitude"),
            remarks: row.get("remarks"),
            valid_from: row.get::<Option<DateTime<Utc>>, _>("valid_from"),
            valid_until: row.get::<Option<DateTime<Utc>>, _>("valid_until"),
        })
        .collect())
}
