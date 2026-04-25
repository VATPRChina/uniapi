use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{adapter::smms::SmmsClient, settings::Settings};

#[derive(Clone)]
pub struct Services {
    db: PgPool,
    smms: SmmsClient,
}

impl Services {
    pub async fn connect(settings: &Settings) -> Result<Self, sqlx::Error> {
        let db = PgPoolOptions::new()
            .max_connections(10)
            .connect(&settings.database.url)
            .await?;

        Ok(Self {
            db,
            smms: SmmsClient::new(settings.storage.image.smms.secret_token.clone()),
        })
    }

    pub fn db(&self) -> &PgPool {
        &self.db
    }

    pub fn smms(&self) -> &SmmsClient {
        &self.smms
    }
}
