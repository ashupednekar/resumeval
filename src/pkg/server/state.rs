use ai::clients::openai::Client as AIClient;
use sqlx::PgPool;
use std::sync::Arc;
use standard_error::StandardError;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::{conf::settings, prelude::Result};

pub fn db_pool() -> Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .max_connections(settings.database_pool_max_connections)
        .connect_lazy(&settings.database_url)?;
    Ok(pool)
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_pool: Arc<PgPool>,
    pub ai_client: Arc<AIClient>
}

impl AppState {
    pub async fn new() -> Result<AppState> {
        let ai = ai::clients::openai::Client::from_url(&settings.ai_key, &settings.ai_endpoint).map_err(|_| StandardError::new("AI-000"))?;
        Ok(AppState {
            db_pool: Arc::new(db_pool()?),
            ai_client: Arc::new(ai)
        })
    }
}
