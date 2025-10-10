use config::{Config, ConfigError, Environment};
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub listen_port: String,
    pub database_url: String,
    pub redis_url: String,
    //otel
    pub otlp_host: Option<String>,
    pub otlp_port: Option<String>,
    pub use_telemetry: bool,
    //email
    pub from_email: String,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    //timeouts
    pub registration_timeout: String 
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let conf = Config::builder()
            .add_source(Environment::default())
            .build()?;
        conf.try_deserialize()
    }
}

lazy_static! {
    pub static ref settings: Settings = Settings::new().expect("improperly configured");
}
