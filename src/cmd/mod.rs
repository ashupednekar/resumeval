use crate::{conf::settings, pkg::server, prelude::Result};
use clap::{Parser, Subcommand};
use sqlx::{Executor, PgPool};

#[derive(Parser)]
#[command(about = "lets you run auth-svc commands")]
struct Cmd {
    #[command(subcommand)]
    command: Option<SubCommandType>,
}

#[derive(Subcommand)]
enum SubCommandType {
    Listen,
    Migrate,
}

pub async fn run() -> Result<()> {
    let args = Cmd::parse();
    match args.command {
        Some(SubCommandType::Listen) => {
            server::listen().await?;
        }
        Some(SubCommandType::Migrate) => {
            let pool = PgPool::connect(&settings.database_url).await?;
            pool.execute(include_str!("../migrations/init.sql")).await?;
            tracing::info!("init migrations applied successfully")
        }
        None => {
            tracing::error!("no subcommand passed")
        }
    }
    Ok(())
}
