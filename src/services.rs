use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{
    adapter::{
        compat::CompatClient, discourse::DiscourseClient, moodle::MoodleClient, smms::SmmsClient,
        vatsim_auth::VatsimAuthClient,
    },
    jwt::JwtService,
    settings::Settings,
};

#[derive(Clone)]
pub struct Services {
    db: PgPool,
    jwt: JwtService,
    smms: SmmsClient,
    compat: CompatClient,
    discourse: DiscourseClient,
    moodle: MoodleClient,
    vatsim_auth: VatsimAuthClient,
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
            discourse: DiscourseClient::new(
                settings.discourse.endpoint.clone(),
                settings.discourse.api_key.clone(),
            ),
            moodle: MoodleClient::new(settings.moodle.api_key.clone()),
            vatsim_auth: VatsimAuthClient::new(settings.authentication.vatsim.clone()),
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

    pub fn discourse(&self) -> &DiscourseClient {
        &self.discourse
    }

    pub fn moodle(&self) -> &MoodleClient {
        &self.moodle
    }

    pub fn vatsim_auth(&self) -> &VatsimAuthClient {
        &self.vatsim_auth
    }
}
