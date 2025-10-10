use async_trait::async_trait;

use crate::{pkg::users::models::User, prelude::Result};

#[async_trait]
pub trait RecoveryActions {
    async fn initiate_recovery(&self) -> Result<()>;
    async fn verify_recovery(&self, code: &str) -> Result<()>;
}

#[async_trait]
impl RecoveryActions for User {
    async fn initiate_recovery(&self) -> Result<()> {
        Ok(())
    }
    async fn verify_recovery(&self, code: &str) -> Result<()> {
        Ok(())
    }
}
