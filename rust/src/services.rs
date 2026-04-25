use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct Services {
    db: PgPool,
}

impl Services {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let db = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { db })
    }

    pub fn db(&self) -> &PgPool {
        &self.db
    }
}
