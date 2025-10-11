use crate::{
    pkg::{
        internal::email::{SendEmail, authtoken::AuthnCodeTemplate},
        server::state::AppState,
    },
    prelude::Result,
};
use axum::http::StatusCode;
use rand::{Rng, distr::Alphanumeric};
use sqlx::{
    prelude::{FromRow, Type},
    types::time::OffsetDateTime,
};
use standard_error::{StandardError, Status};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Type)]
#[sqlx(type_name = "token_status", rename_all = "lowercase")]
pub enum TokenStatus {
    Pending,
    Verified,
    Rejected,
    Expired,
}

#[derive(FromRow, Debug)]
pub struct AuthToken {
    pub token: Uuid,
    pub user_id: String,
    pub code: String,
    pub expiry: OffsetDateTime,
    pub status: TokenStatus,
}

#[derive(FromRow, Debug)]
pub struct User {
    pub user_id: String,
    pub email: String,
    pub name: String,
}

impl User {
    pub async fn create(state: &AppState, email: &str, name: &str) -> Result<Self> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, name, user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (email) DO update
            set name = $2
            RETURNING user_id, email, name 
            "#,
            email,
            name,
            Uuid::new_v4().to_string()
        )
        .fetch_one(&*state.db_pool)
        .await?;
        Ok(user)
    }

    pub async fn retrieve(state: &AppState, email: &str) -> Result<Option<Self>> {
        Ok(sqlx::query_as!(
            User,
            r#"
            select user_id, email, name from users
            where email = $1
            "#,
            &email
        )
        .fetch_optional(&*state.db_pool)
        .await?)
    }

    pub async fn issue_token(&self, state: &AppState) -> Result<()> {
        let pool = &*state.db_pool;
        let code = AuthToken::generate_code();
        tracing::debug!("issued code: {}", &code);
        sqlx::query!(
            r#"
            INSERT INTO tokens (user_id, code, expiry, status)
            VALUES ($1, $2, NOW() + interval '1 hour', $3)
            "#,
            self.user_id,
            code,
            TokenStatus::Pending as _
        )
        .execute(pool)
        .await?;
        AuthnCodeTemplate {
            name: &self.name,
            code: &code,
        }
        .send(&self.email)?;
        Ok(())
    }
}

impl AuthToken {
    fn generate_code() -> String {
    //    rand::rng()
    //        .sample_iter(&Alphanumeric)
    //        .take(6)
    //        .map(char::from)
    //        .collect()
        let mut rng = rand::rng();
        (0..6).map(|_| rng.random_range(0..10).to_string())
            .collect()
    }

    pub async fn issue_user_token(state: &AppState, email: &str, name: &str) -> Result<User> {
        let user = User::create(&state, email, name).await?;
        user.issue_token(&state).await?;
        Ok(user)
    }

    pub async fn check_code(token: AuthToken, code: &str) -> Result<()> {
        if token.code == code {
            Ok(())
        } else {
            Err(StandardError::new("ERR-AUTH-003"))
        }
    }

    pub async fn check_token_validity(state: &AppState, token_str: &str) -> Result<User> {
        let pool = &*state.db_pool;
        let token_str = token_str
            .parse::<Uuid>()
            .map_err(|_| StandardError::new("ERR-AUTH-002"))?;

        tracing::info!("Verifying token: {}", token_str);
        let result = sqlx::query_as!(
            AuthToken,
            r#"
            SELECT token, user_id, code, expiry, status as "status: _"
            FROM tokens
            WHERE token = $1
            AND status = $2
            AND expiry > now()
            "#,
            &token_str,
            &TokenStatus::Verified as _
        )
        .fetch_optional(pool)
        .await;
        if let Ok(Some(token)) = result {
            let user = sqlx::query_as!(
                User,
                r#"SELECT user_id, email, name FROM users WHERE user_id = $1"#,
                token.user_id
            )
            .fetch_one(pool)
            .await?;
            Ok(user)
        } else {
            Err(StandardError::new("ERR-AUTH-001"))
        }
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;
    use uuid::Uuid;

    use crate::{
        pkg::{internal::auth::AuthToken, server::state::AppState},
        prelude::Result,
    };

    #[traced_test]
    #[tokio::test]
    async fn test_verify() -> Result<()> {
        let state = AppState::new().await?;
        let token = Uuid::new_v4();
        let _ = AuthToken::check_token_validity(&state, &token.to_string()).await;
        Ok(())
    }
}
