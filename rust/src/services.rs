use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{
    adapter::{compat::CompatClient, smms::SmmsClient},
    jwt::JwtService,
    settings::Settings,
};

#[derive(Clone)]
pub struct Services {
    db: PgPool,
    jwt: JwtService,
    smms: SmmsClient,
    compat: CompatClient,
}

impl Services {
    pub async fn connect(settings: &Settings) -> Result<Self, sqlx::Error> {
        let db = PgPoolOptions::new()
            .max_connections(10)
            .connect(&settings.database.url)
            .await?;

        Ok(Self {
            db,
            jwt: JwtService::new(&settings.authentication.jwt),
            smms: SmmsClient::new(
                settings.storage.image.smms.base_url.clone(),
                settings.storage.image.smms.secret_token.clone(),
            ),
            compat: CompatClient::new(settings.utils.metar.endpoint.clone()),
        })
    }

    pub fn db(&self) -> &PgPool {
        &self.db
    }

    pub fn smms(&self) -> &SmmsClient {
        &self.smms
    }

    pub fn jwt(&self) -> &JwtService {
        &self.jwt
    }

    pub fn compat(&self) -> &CompatClient {
        &self.compat
    }
}
