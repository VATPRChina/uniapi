use sqlx::postgres::PgPoolOptions;

pub async fn migrate(database_url: &str) -> Result<(), sqlx::migrate::MigrateError> {
    let db = PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await?;

    sqlx::migrate!().run(&db).await?;
    db.close().await;

    Ok(())
}
