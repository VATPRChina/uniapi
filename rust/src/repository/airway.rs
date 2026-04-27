use sqlx::{PgPool, Row};

use crate::{
    flight_plan::{AirwayDirection, AirwayLeg, Fix},
    repository::{ndb_navaid, vhf_navaid, waypoint},
};

pub async fn exists_with_fix(
    db: &PgPool,
    airway_ident: &str,
    fix_ident: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM navdata.airway
            JOIN navdata.airway_fix ON airway_fix.airway_id = airway.id
            WHERE airway.identifier = $1
              AND airway_fix.fix_identifier = $2
        )
        "#,
    )
    .bind(airway_ident)
    .bind(fix_ident)
    .fetch_one(db)
    .await
}

pub async fn legs(db: &PgPool, airway_ident: &str) -> Result<Vec<AirwayLeg>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        WITH ordered AS (
            SELECT
                airway.identifier AS airway_identifier,
                airway_fix.fix_identifier,
                airway_fix.directional_restriction,
                airway_fix.sequence_number,
                lead(airway_fix.fix_identifier) OVER (
                    PARTITION BY airway.id ORDER BY airway_fix.sequence_number
                ) AS next_fix_identifier
            FROM navdata.airway
            JOIN navdata.airway_fix ON airway_fix.airway_id = airway.id
            WHERE airway.identifier = $1
        )
        SELECT *
        FROM ordered
        WHERE next_fix_identifier IS NOT NULL
        ORDER BY sequence_number
        "#,
    )
    .bind(airway_ident)
    .fetch_all(db)
    .await?;

    let mut legs = Vec::with_capacity(rows.len());
    for row in rows {
        let from_ident: String = row.get("fix_identifier");
        let to_ident: String = row.get("next_fix_identifier");
        let Some(from) = find_fix(db, &from_ident).await? else {
            continue;
        };
        let Some(to) = find_fix(db, &to_ident).await? else {
            continue;
        };
        let direction = match row.get::<String, _>("directional_restriction").trim() {
            "F" => AirwayDirection::Forward,
            "B" => AirwayDirection::Backward,
            _ => AirwayDirection::Both,
        };
        legs.push(AirwayLeg {
            from,
            to,
            identifier: row.get("airway_identifier"),
            direction,
        });
    }
    Ok(legs)
}

async fn find_fix(db: &PgPool, ident: &str) -> Result<Option<Fix>, sqlx::Error> {
    let mut fixes = waypoint::find(db, ident).await?;
    fixes.extend(vhf_navaid::find(db, ident).await?);
    fixes.extend(ndb_navaid::find(db, ident).await?);
    Ok(fixes.into_iter().next())
}
