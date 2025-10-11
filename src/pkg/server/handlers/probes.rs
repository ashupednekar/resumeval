use axum::extract::State;
use sqlx::query;

use crate::{pkg::server::state::AppState, prelude::Result};

pub async fn livez() -> Result<()> {
    tracing::debug!("service is live");
    Ok(())
}

pub async fn healthz(State(state): State<AppState>) -> Result<()> {
    query("select 1").execute(&*state.db_pool).await?;
    tracing::debug!("service is healthy");
    Ok(())
}
