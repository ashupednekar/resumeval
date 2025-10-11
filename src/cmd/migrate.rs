use crate::conf::settings;
use sqlx::{PgPool, migrate::Migrator};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn apply() -> Result<(), sqlx::Error> {
    let pool = PgPool::connect(&settings.database_url).await?;
    //sqlx::query(&format!("set search_path to {}", &settings.database_schema)).execute(&pool).await?;
    MIGRATOR.run(&pool).await?;
    println!("Migrations applied successfully");
    Ok(())
}
