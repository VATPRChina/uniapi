use sqlx::PgPool;
use uuid::Uuid;

pub async fn has_any_by_user_id(db: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM public.user_atc_permission
            WHERE user_id = $1
        )
        "#,
    )
    .bind(user_id)
    .fetch_one(db)
    .await
}

pub async fn has_mentor_by_user_id(db: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM public.user_atc_permission
            WHERE user_id = $1 AND state = 'Mentor'
        )
        "#,
    )
    .bind(user_id)
    .fetch_one(db)
    .await
}
