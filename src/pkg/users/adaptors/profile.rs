use async_trait::async_trait;

use crate::{pkg::users::models::User, prelude::Result};

#[async_trait]
trait ProfileMutator {
    async fn update_dp(&mut self, display_pic: &str) -> Result<()>;
}

#[async_trait]
impl ProfileMutator for User {
    async fn update_dp(&mut self, display_pic: &str) -> Result<()> {
        Ok(())
    }
}
