use std::sync::Arc;

use crate::{conf::settings, pkg::server::state::GetTxn};
use sqlx::{PgPool, migrate::Migrator};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn apply() -> Result<(), sqlx::Error> {
    let pool = Arc::new(PgPool::connect(&settings.database_url).await?);
    let mut tx = pool.begin_txn().await?;
    MIGRATOR.run(&mut tx).await?;
    println!("Migrations applied successfully");
    Ok(())
}
