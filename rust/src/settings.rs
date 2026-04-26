use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub bind_address: String,
    pub authentication: Authentication,
    pub database: Database,
    pub moodle: Moodle,
    pub storage: Storage,
    pub utils: Utils,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Authentication {
    pub jwt: JwtAuthentication,
    pub vatsim: VatsimAuthentication,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtAuthentication {
    pub private_key: String,
    pub public_key: String,
    pub issuer: String,
    pub audience_first_party: String,
    pub first_party_expires_seconds: i64,
    pub refresh_expires_days: i64,
    pub device_authz_expires_seconds: i64,
    #[serde(default)]
    pub clients: Vec<JwtClient>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtClient {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VatsimAuthentication {
    pub endpoint: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Storage {
    pub image: ImageStorage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImageStorage {
    pub smms: SmmsStorage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SmmsStorage {
    pub base_url: String,
    pub secret_token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Moodle {
    pub api_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Utils {
    pub metar: Metar,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Metar {
    pub endpoint: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            .add_source(File::from_str(
                include_str!("../assets/config/settings.default.toml"),
                FileFormat::Toml,
            ))
            .add_source(File::with_name(&format!("settings.{}", run_mode)).required(false))
            .add_source(File::with_name("settings.local").required(false))
            .add_source(Environment::with_prefix("app"))
            .build()?;

        s.try_deserialize()
    }
}
