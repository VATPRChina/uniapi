use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::adapter::compat::CompatClient;
use crate::adapter::discourse::DiscourseClient;
use crate::adapter::email::EmailClient;
use crate::adapter::moodle::MoodleClient;
use crate::adapter::navdata::NavdataAdapter;
use crate::adapter::smms::SmmsClient;
use crate::adapter::vatsim_auth::VatsimAuthClient;
use crate::audit_log_service::AuditLogService;
use crate::jwt::JwtService;
use crate::settings::Settings;

#[derive(Clone)]
pub struct Services {
    db: PgPool,
    jwt: JwtService,
    smms: SmmsClient,
    compat: CompatClient,
    #[allow(dead_code)]
    discourse: DiscourseClient,
    email: EmailClient,
    moodle: MoodleClient,
    vatsim_auth: VatsimAuthClient,
    navdata: NavdataAdapter,
    audit_log: AuditLogService,
}

impl Services {
    pub async fn connect(settings: &Settings) -> Result<Self, anyhow::Error> {
        let db = PgPoolOptions::new()
            .max_connections(10)
            .connect(&settings.database.url)
            .await?;
        let navdata = NavdataAdapter::with_preferred_routes_path(
            &settings.navdata.local_data_path,
            &settings.navdata.preferred_routes_path,
        )
        .await?;

        Ok(Self {
            audit_log: AuditLogService::new(db.clone()),
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
            email: EmailClient::new(&settings.email)?,
            moodle: MoodleClient::new(settings.moodle.api_key.clone()),
            vatsim_auth: VatsimAuthClient::new(settings.authentication.vatsim.clone()),
            navdata,
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

    // TODO: Unsuppress when DiscourseClient is used in at least one route
    #[allow(dead_code)]
    pub fn discourse(&self) -> &DiscourseClient {
        &self.discourse
    }

    pub fn moodle(&self) -> &MoodleClient {
        &self.moodle
    }

    pub fn email(&self) -> &EmailClient {
        &self.email
    }

    pub fn vatsim_auth(&self) -> &VatsimAuthClient {
        &self.vatsim_auth
    }

    pub fn navdata(&self) -> &NavdataAdapter {
        &self.navdata
    }

    pub fn audit_log(&self) -> &AuditLogService {
        &self.audit_log
    }
}
