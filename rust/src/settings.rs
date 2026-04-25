use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub bind_address: String,
    pub authentication: Authentication,
    pub database: Database,
    pub storage: Storage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Authentication {
    pub jwt: JwtAuthentication,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtAuthentication {
    pub public_key: String,
    pub issuer: String,
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
    pub secret_token: String,
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
