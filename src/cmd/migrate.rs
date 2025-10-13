use std::sync::Arc;

use crate::{conf::settings, prelude::Result, pkg::server::state::GetTxn};
use sqlx::{migrate::Migrator, postgres::PgPoolOptions, PgPool};
use standard_error::{Interpolate, StandardError};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");


pub async fn apply() -> Result<()> {
    let pool = Arc::new(
        PgPoolOptions::new()
            .connect(&settings.database_url)
            .await
            .map_err(|e| StandardError::new("ERR-DB-000").interpolate_err(e.to_string()))?
    );
    
    tracing::debug!("connected to db");
    let mut tx = pool.begin_txn().await?;
    MIGRATOR.run(&mut *tx).await
        .map_err(|e| StandardError::new("ERR-DB-000").interpolate_err(e.to_string()))?;
    
    tx.commit().await
        .map_err(|e| StandardError::new("ERR-DB-000").interpolate_err(e.to_string()))?;
    
    println!("Migrations applied successfully");
    Ok(())
}
