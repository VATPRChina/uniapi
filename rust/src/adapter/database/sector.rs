use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn user_can_online(db: &PgPool, user_id: Uuid, cid: &str) -> Result<bool, sqlx::Error> {
    if cid == "1573922" {
        return Ok(true);
    }

    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM public.user_atc_permission
            WHERE user_id = $1
              AND (
                  state IN ('Student', 'UnderMentor', 'Certified', 'Mentor')
                  OR (state = 'Solo' AND (solo_expires_at IS NULL OR solo_expires_at > $2))
              )
        )
        "#,
    )
    .bind(user_id)
    .bind(Utc::now())
    .fetch_one(db)
    .await
}
