mod cmd;
mod conf;
pub mod pkg;
mod prelude;
pub mod state;

use crate::prelude::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    cmd::run().await?;
    Ok(())
}
