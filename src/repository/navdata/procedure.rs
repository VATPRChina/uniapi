use sqlx::PgPool;

pub async fn find_sid(
    db: &PgPool,
    ident: &str,
    airport_ident: &str,
) -> Result<Option<String>, sqlx::Error> {
    find(db, ident, airport_ident, "D").await
}

pub async fn find_star(
    db: &PgPool,
    ident: &str,
    airport_ident: &str,
) -> Result<Option<String>, sqlx::Error> {
    find(db, ident, airport_ident, "A").await
}

async fn find(
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
