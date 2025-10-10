use async_trait::async_trait;
use humantime::parse_duration;
use redis::Commands;
use standard_error::StandardError;
use uuid::Uuid;

use crate::{conf::settings, pkg::{email::send_email, users::models::User}, prelude::Result, state::AppState};

#[async_trait]
pub trait RegisterActions {
    async fn initiate_registration(&self, state: AppState) -> Result<()>;
    async fn verify_registration(state: AppState, code: &str) -> Result<()>;
}

#[async_trait]
impl RegisterActions for User {
    async fn initiate_registration(&self, state: AppState) -> Result<()> {
        let code = Uuid::new_v4().to_string();
        let mut conn = state.redis_client.get_connection().unwrap();
        let timeout = parse_duration(&settings.registration_timeout).expect("invalid time format").as_secs(); 
        let _: () = redis::cmd("SET")
            .arg(&code)
            .arg(&self.username)
            .arg("EX")
            .arg(timeout)
            .query(&mut conn)
            .map_err(|_|StandardError::new("ERR-REG-001"))?;
        send_email(&self.email, "registration", &format!("here's your code: {}", &code)).await?;
        Ok(())
    }


    async fn verify_registration(state: AppState, code: &str) -> Result<()> {
        let mut conn = state.redis_client.get_connection().unwrap();
        let username: Option<String> = redis::cmd("GET")
            .arg(code)
            .query(&mut conn)
            .map_err(|_| StandardError::new("ERR-REG-002"))?; // expired code
        if let Some(username) = username{
            let user = User::find_by_username(&state.db_pool, &username).await?;
            if let Some(user) = user{
                user.mark_as_verified(&state.db_pool).await?; 
            }else{
                return Err(StandardError::new("ERR-REG-003")); //invalid code
            }
        }
        Ok(())
    }
}
