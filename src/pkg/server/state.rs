use ai::clients::openai::Client as AIClient;
use aws_sdk_s3::{config::{Credentials, Region}, Client as S3Client};
use sqlx::PgPool;
use std::sync::Arc;
use standard_error::{Interpolate, StandardError};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use crate::{conf::settings, pkg::internal::minio::create_bucket, prelude::Result};

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
    pub s3_client: Arc<S3Client>
}

impl AppState {
    pub async fn new() -> Result<AppState> { 
        let ai = if &settings.ai_provider == "gemini"{
            ai::clients::openai::ClientBuilder::default()
              .http_client(
                  reqwest::Client::builder()
                      .http1_title_case_headers()
                      .build()?,
              )
              .api_key(settings.ai_key.clone().into())
              .base_url("https://generativelanguage.googleapis.com/v1beta/openai".into())
              .build()
              .map_err(|e| StandardError::new("ERR-AI-000").interpolate_err(e.to_string()))
              ?
        }else{
            ai::clients::openai::Client::from_url(
                &settings.ai_key, &settings.ai_endpoint
            ).map_err(|e| StandardError::new("ERR-AI-000").interpolate_err(e.to_string()))?
        };
        let s3_config = aws_sdk_s3::config::Builder::new()
            .credentials_provider(Credentials::new(
                &settings.s3_access_key, &settings.s3_secret_key, None, None, "")
            )
            .endpoint_url(&settings.s3_endpoint)
            .region(Region::new(settings.s3_region.clone()))
            .force_path_style(true)
            .build();
        let s3_client = aws_sdk_s3::Client::from_conf(s3_config);
        create_bucket(&s3_client, &settings.s3_bucket_name).await?;
        Ok(AppState {
            db_pool: Arc::new(db_pool()?),
            ai_client: Arc::new(ai),
            s3_client: Arc::new(s3_client)
        })
    }
}
