use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::flight_plan::{
    AirwayDirection, AirwayLeg, Fix, FixKind, LevelRestrictionType, PreferredRoute,
};

pub async fn find_airport(db: &PgPool, ident: &str) -> Result<Option<Fix>, sqlx::Error> {
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

pub async fn find_fix(
    db: &PgPool,
    ident: &str,
    lat: f64,
    lon: f64,
) -> Result<Option<Fix>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT kind, icao_code, identifier, latitude, longitude
        FROM (
            SELECT 'Waypoint' AS kind, icao_code, identifier, latitude, longitude
            FROM navdata.waypoint
            WHERE identifier = $1
            UNION ALL
            SELECT 'VhfNavaid' AS kind, icao_code, vor_identifier AS identifier,
                   vor_latitude AS latitude, vor_longitude AS longitude
            FROM navdata.vhf_navaid
            WHERE vor_identifier = $1
            UNION ALL
            SELECT 'NdbNavaid' AS kind, icao_code, identifier, latitude, longitude
            FROM navdata.ndb_navaid
            WHERE identifier = $1
        ) fixes
        ORDER BY (($2 - latitude) * ($2 - latitude)) + (($3 - longitude) * ($3 - longitude))
        LIMIT 1
        "#,
    )
    .bind(ident)
    .bind(lat)
    .bind(lon)
    .fetch_optional(db)
    .await
    .map(|row| {
        row.map(|row| {
            let kind = match row.get::<String, _>("kind").as_str() {
                "VhfNavaid" => FixKind::VhfNavaid,
                "NdbNavaid" => FixKind::NdbNavaid,
                _ => FixKind::Waypoint,
            };
            Fix::identified(
                kind,
                row.get::<String, _>("icao_code"),
                row.get::<String, _>("identifier"),
                row.get("latitude"),
                row.get("longitude"),
            )
        })
    })
}

pub async fn find_sid(
    db: &PgPool,
    ident: &str,
    airport_ident: &str,
) -> Result<Option<String>, sqlx::Error> {
    find_procedure(db, ident, airport_ident, "D").await
}

pub async fn find_star(
    db: &PgPool,
    ident: &str,
    airport_ident: &str,
) -> Result<Option<String>, sqlx::Error> {
    find_procedure(db, ident, airport_ident, "A").await
}

async fn find_procedure(
    db: &PgPool,
    ident: &str,
    airport_ident: &str,
    subsection_code: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT procedure.identifier
        FROM navdata.procedure
        JOIN navdata.airport ON airport.id = procedure.airport_id
        WHERE procedure.identifier = $1
          AND airport.identifier = $2
          AND procedure.subsection_code = $3
        LIMIT 1
        "#,
    )
    .bind(ident)
    .bind(airport_ident)
    .bind(subsection_code)
    .fetch_optional(db)
    .await
}

pub async fn exists_airway_with_fix(
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

pub async fn airway_legs(db: &PgPool, airway_ident: &str) -> Result<Vec<AirwayLeg>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        WITH ordered AS (
            SELECT
                airway.identifier AS airway_identifier,
                airway_fix.fix_identifier,
                airway_fix.fix_icao_code,
                airway_fix.directional_restriction,
                airway_fix.sequence_number,
                lead(airway_fix.fix_identifier) OVER (
                    PARTITION BY airway.id ORDER BY airway_fix.sequence_number
                ) AS next_fix_identifier,
                lead(airway_fix.fix_icao_code) OVER (
                    PARTITION BY airway.id ORDER BY airway_fix.sequence_number
                ) AS next_fix_icao_code
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
        let Some(from) = find_fix(db, &from_ident, 0.0, 0.0).await? else {
            continue;
        };
        let Some(to) = find_fix(db, &to_ident, 0.0, 0.0).await? else {
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

pub async fn recommended_routes(
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
