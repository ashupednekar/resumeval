use crate::{conf::settings, pkg::internal::minio::S3Ops, prelude::Result};
use ai::clients::openai::Client as AIClient;
use aws_sdk_s3::{
    Client as S3Client,
    config::{Credentials, Region},
};
use axum::async_trait;
use sqlx::{PgConnection, PgPool, Transaction};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use standard_error::{Interpolate, StandardError};
use std::sync::Arc;

pub fn db_pool() -> Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .max_connections(settings.database_pool_max_connections)
        .connect_lazy(&settings.database_url)?;
    Ok(pool)
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_pool: Arc<PgPool>,
    pub ai_client: Arc<AIClient>,
    pub s3_client: Arc<S3Client>,
}

#[async_trait]
pub trait GetTxn{
    async fn begin_txn(&self) -> Result<Transaction<'static, Postgres>>;
}

#[async_trait]
impl GetTxn for Arc<PgPool>{
    async fn begin_txn(&self) -> Result<Transaction<'static, Postgres>>{
        let mut tx = self.begin().await?;
        let set_schema = format!("set search_path to {}", &settings.database_schema);
        tracing::info!("{}", &set_schema);
        sqlx::query(&set_schema).execute(&mut *tx).await?;
        Ok(tx)
    }
}


impl AppState {
    pub async fn new() -> Result<AppState> {
        let ai = if &settings.ai_provider == "gemini" {
            ai::clients::openai::ClientBuilder::default()
                .http_client(
                    reqwest::Client::builder()
                        .http1_title_case_headers()
                        .build()?,
                )
                .api_key(settings.ai_key.clone().into())
                .base_url("https://generativelanguage.googleapis.com/v1beta/openai".into())
                .build()
                .map_err(|e| StandardError::new("ERR-AI-000").interpolate_err(e.to_string()))?
        } else {
            ai::clients::openai::Client::from_url(&settings.ai_key, &settings.ai_endpoint)
                .map_err(|e| StandardError::new("ERR-AI-000").interpolate_err(e.to_string()))?
        };
        let s3_config = aws_sdk_s3::config::Builder::new()
            .credentials_provider(Credentials::new(
                &settings.s3_access_key,
                &settings.s3_secret_key,
                None,
                None,
                "",
            ))
            .endpoint_url(&settings.s3_endpoint)
            .region(Region::new(settings.s3_region.clone()))
            .force_path_style(true)
            .build();
        let s3_client = Arc::new(aws_sdk_s3::Client::from_conf(s3_config));
        s3_client.create_new_bucket(&settings.s3_bucket_name).await?;
        Ok(AppState {
            db_pool: Arc::new(db_pool()?),
            ai_client: Arc::new(ai),
            s3_client
        })
    }
}
