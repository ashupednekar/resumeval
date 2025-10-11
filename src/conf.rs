use config::{Config, ConfigError, Environment};
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub base_url: String,
    pub service_name: String,
    pub listen_port: String,
    pub database_url: String,
    pub database_schema: String,
    pub database_pool_max_connections: u32,
    //otel
    //pub otlp_host: Option<String>,
    //pub otlp_port: Option<String>,
    //pub use_telemetry: bool,
    //email
    pub from_email: String,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub ai_endpoint: String,
    pub ai_provider: String,
    pub ai_model: String,
    pub ai_key: String
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let conf = Config::builder()
            .add_source(Environment::default())
            .build()?;
        let mut s: Settings = conf.try_deserialize()?;
        match s.ai_provider.as_str(){
        "ollama" => {
            s.ai_key = "ollama".into();
            s.ai_endpoint = "http://localhost:11434/v1".into();
            if s.ai_model.is_empty(){
                s.ai_model = "gemma3:12b".into();
            }
        },
        "openai" => {
            s.ai_endpoint = "https://api.openai.com/v1".into();
            if s.ai_model.is_empty(){
                s.ai_model = "gpt-4o-mini".into();
            }
        },
        "gemini" => {
            s.ai_endpoint = "https://generativelanguage.googleapis.com/v1beta/openai".into();
            if s.ai_model.is_empty(){
                s.ai_model = "gemini-2.5-flash".into();
            }
        },
        _ => {}
    }
        Ok(s)
    }
}

lazy_static! {
    pub static ref settings: Settings = Settings::new().expect("improperly configured");
}
