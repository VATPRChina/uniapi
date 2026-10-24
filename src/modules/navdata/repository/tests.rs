use sqlx::{Connection, SqliteConnection};

use super::*;

#[tokio::test]
async fn queries_committed_changes_without_reloading_the_repository() {
    let path = std::env::temp_dir().join(format!("routes-{}.db", ulid::Ulid::new()));
    let mut writer = SqliteConnection::connect_with(
        &SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true),
    )
    .await
    .unwrap();
    sqlx::raw_sql(include_str!("../../../../assets/test/routes.sql"))
        .execute(&mut writer)
        .await
        .unwrap();
    // Unrelated invalid data must not be read at startup or when querying another pair.
    sqlx::query("UPDATE Routes SET MinAlt = 'invalid' WHERE Dep = 'ZSPD'")
        .execute(&mut writer)
        .await
        .unwrap();

    let repository = PreferredRouteRepository::from_db_path(&path).await.unwrap();
    let clone = repository.clone();
    let original = repository
        .list_preferred_routes("zbaa", "zspd")
        .await
        .unwrap();
    assert_eq!(original.len(), 1);
    assert_eq!(original[0].name, "ZBAA-ZSPD-01");

    sqlx::query("UPDATE Routes SET Name = 'updated', Route = 'NEW DCT ROUTE' WHERE Dep = 'ZBAA'")
        .execute(&mut writer)
        .await
        .unwrap();
    let updated = clone.list_preferred_routes("ZBAA", "ZSPD").await.unwrap();
    assert_eq!(updated[0].name, "updated");
    assert_eq!(updated[0].raw_route, "NEW DCT ROUTE");
    assert_eq!(original[0].name, "ZBAA-ZSPD-01");

    sqlx::query(
        "INSERT INTO Routes VALUES
         ('ZBAA', 'ZSPD', '', '', 'inserted', 'FE', 'F310/S101', '', 'DCT', 'AiP RoUtE')",
    )
    .execute(&mut writer)
    .await
    .unwrap();
    let inserted = repository
        .list_preferred_routes("ZBAA", "ZSPD")
        .await
        .unwrap();
    assert_eq!(inserted.len(), 2);
    assert_eq!(inserted[0].name, "updated");
    assert_eq!(inserted[1].name, "inserted");
    assert_eq!(inserted[1].allowed_altitudes, [31000, 33100]);
    assert_eq!(
        inserted[1].cruising_level_restriction,
        LevelRestrictionType::FlightLevelEven
    );
    assert!(!inserted[1].is_public);
    assert!(matches!(
        repository.list_preferred_routes("ZSPD", "ZBAA").await,
        Err(PreferredRouteRepositoryError::InvalidAltitude(_))
    ));

    sqlx::query("DELETE FROM Routes WHERE Dep = 'ZBAA'")
        .execute(&mut writer)
        .await
        .unwrap();
    assert!(
        clone
            .list_preferred_routes("ZBAA", "ZSPD")
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        sqlx::query("DELETE FROM Routes")
            .execute(&repository.db)
            .await
            .is_err()
    );

    repository.db.close().await;
    writer.close().await.unwrap();
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn queries_airport_and_waypoint_endpoints_case_insensitively() {
    let db = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::raw_sql(include_str!("../../../../assets/test/routes.sql"))
        .execute(&db)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO Routes VALUES
         (' zbaa ', '', '', ' exit ', 'outbound', '', '', '', 'DCT', ''),
         ('', ' zspd ', ' entry ', '', 'inbound', '', '', '', 'DCT', ''),
         ('', '', ' entry ', ' exit ', 'transit', '', '', '', 'DCT', '')",
    )
    .execute(&db)
    .await
    .unwrap();
    let repository = PreferredRouteRepository { db };
    for (departure, arrival, name) in [
        ("ZBAA", "ZSPD", "ZBAA-ZSPD-01"),
        ("ZBAA", "EXIT", "outbound"),
        ("ENTRY", "ZSPD", "inbound"),
        ("ENTRY", "EXIT", "transit"),
    ] {
        let routes = repository
            .list_preferred_routes(departure, arrival)
            .await
            .unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].name, name);
        assert_eq!(routes[0].departure, departure);
        assert_eq!(routes[0].arrival, arrival);
    }
}

#[tokio::test]
async fn reports_database_errors_on_lookup() {
    let repository = PreferredRouteRepository {
        db: SqlitePool::connect("sqlite::memory:").await.unwrap(),
    };
    assert!(matches!(
        repository.list_preferred_routes("ZBAA", "ZSPD").await,
        Err(PreferredRouteRepositoryError::Database(_))
    ));
}
