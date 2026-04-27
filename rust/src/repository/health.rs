use sqlx::PgPool;

pub async fn is_healthy(db: &PgPool) -> bool {
    matches!(
        sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(db).await,
        Ok(1)
    )
}
