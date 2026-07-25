use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::adapter::compat::CompatClient;
use crate::adapter::discourse::DiscourseClient;
use crate::adapter::email::EmailClient;
use crate::adapter::moodle::MoodleClient;
use crate::adapter::navdata::NavdataAdapter;
use crate::adapter::smms::SmmsClient;
use crate::adapter::vatsim_auth::VatsimAuthClient;
use crate::modules::atc_application::service::AtcApplicationService;
use crate::modules::training::service::{TrainingApplicationService, TrainingService};
use crate::modules::user::service::access_token::AccessTokenService;
use crate::modules::user::service::device_authorization::DeviceAuthorizationService;
use crate::modules::user::service::refresh_token::RefreshTokenService;
use crate::modules::user::service::user::UserService;
use crate::settings::Settings;

pub mod controller_info;

use crate::modules::audit_log::service::AuditLogService;
use controller_info::ControllerInfoService;

#[derive(Clone)]
pub struct Services {
    db: PgPool,
    access_token: AccessTokenService,
    refresh_token: RefreshTokenService,
    device_authorization: DeviceAuthorizationService,
    smms: SmmsClient,
    compat: CompatClient,
    #[allow(dead_code)]
    discourse: DiscourseClient,
    email: EmailClient,
    moodle: MoodleClient,
    vatsim_auth: VatsimAuthClient,
    navdata: NavdataAdapter,
    audit_log: AuditLogService,
    atc_application: AtcApplicationService,
    training: TrainingService,
    training_application: TrainingApplicationService,
    user: UserService,
    controller_info: ControllerInfoService,
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
        let audit_log = AuditLogService::new(db.clone());
        let moodle = MoodleClient::new(settings.moodle.api_key.clone());
        let user = UserService::new(db.clone(), moodle.clone(), audit_log.clone());
        let atc_application =
            AtcApplicationService::new(db.clone(), audit_log.clone(), user.clone());
        let access_token = AccessTokenService::new(&settings.authentication.jwt);
        let refresh_token =
            RefreshTokenService::new(db.clone(), settings.authentication.jwt.refresh_expires_days);
        let device_authorization = DeviceAuthorizationService::new(
            db.clone(),
            settings.authentication.jwt.device_authz_expires_seconds,
        );
        let email = EmailClient::new(&settings.email)?;
        let training = TrainingService::new(db.clone(), user.clone());
        let training_application =
            TrainingApplicationService::new(db.clone(), email.clone(), user.clone());

        Ok(Self {
            controller_info: ControllerInfoService::new(db.clone()),
            user,
            audit_log,
            atc_application,
            training,
            training_application,
            db,
            access_token,
            refresh_token,
            device_authorization,
            smms: SmmsClient::new(
                settings.storage.image.smms.base_url.clone(),
                settings.storage.image.smms.secret_token.clone(),
            ),
            compat: CompatClient::new(settings.utils.metar.endpoint.clone()),
            discourse: DiscourseClient::new(
                settings.discourse.endpoint.clone(),
                settings.discourse.api_key.clone(),
            ),
            email,
            moodle,
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

    pub fn access_token(&self) -> &AccessTokenService {
        &self.access_token
    }

    pub fn refresh_token(&self) -> &RefreshTokenService {
        &self.refresh_token
    }

    pub fn device_authorization(&self) -> &DeviceAuthorizationService {
        &self.device_authorization
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

    pub fn atc_application(&self) -> &AtcApplicationService {
        &self.atc_application
    }

    pub fn training(&self) -> &TrainingService {
        &self.training
    }

    pub fn training_application(&self) -> &TrainingApplicationService {
        &self.training_application
    }

    pub fn user(&self) -> &UserService {
        &self.user
    }

    pub fn controller_info(&self) -> &ControllerInfoService {
        &self.controller_info
    }
}
