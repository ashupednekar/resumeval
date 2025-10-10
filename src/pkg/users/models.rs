use crate::{pkg::server::handlers::user_mgmt::RegistrationInput, prelude::Result};
use chrono::NaiveDateTime;
use serde::Deserialize;
use sqlx::{prelude::FromRow, PgPool};

#[derive(Debug, Deserialize, FromRow)]
pub struct User {
    pub email: String,
    pub username: String,
    pub password: String,
    pub display_pic: Option<String>,
    pub verified: bool,
}

impl User {

    pub fn new(payload: RegistrationInput) -> Self{
        Self {
            username: payload.username,
            email: payload.email,
            password: payload.password,
            display_pic: None,
            verified: false,
        }
    }

    pub async fn save(&self, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO users (username, email, password, display_pic, verified) 
            VALUES ($1, $2, $3, $4, $5)
            "#,
            self.username,
            self.email,
            self.password,
            self.display_pic,
            self.verified
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn mark_as_verified(&self, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users 
            SET verified = true 
            WHERE username = $1
            "#,
            self.username
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn update_display_pic(&self, pool: &PgPool, new_dp: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users 
            SET display_pic = $1 
            WHERE username = $2
            "#,
            new_dp,
            self.username
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT username, email, password, display_pic, verified 
            FROM users 
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

}

#[derive(Debug, Deserialize, FromRow)]
pub struct Session {
    pub username: String,
    pub token: String,
    pub create_dt: NaiveDateTime,
}
