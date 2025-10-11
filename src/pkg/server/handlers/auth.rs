use std::sync::Arc;

use axum::{
    Extension, Form,
    extract::State,
    http::{HeaderMap, HeaderValue, header::SET_COOKIE},
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use validator::Validate;

use crate::{
    pkg::{
        internal::auth::{AuthToken, TokenStatus, User},
        server::state::AppState,
    },
    prelude::Result,
};

#[derive(Deserialize)]
pub struct SignupInput {
    pub email: String,
    pub name: String,
}

#[derive(Deserialize, Validate)]
pub struct VerifyInput {
    #[validate(length(equal = 6))]
    pub code: String,
}

pub async fn signup(
    State(state): State<AppState>,
    Form(input): Form<SignupInput>,
) -> Result<impl IntoResponse> {
    let user = AuthToken::issue_user_token(&state, &input.email, &input.name).await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&format!("_Host_lws_email={}", &user.email))?,
    );
    headers.insert("HX-Redirect", HeaderValue::from_str("/otp")?);
    Ok(headers)
}

pub async fn logout(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<User>>,
) -> Result<Html<String>> {
    sqlx::query!(
        "update tokens set status = $2 where user_id = $3 and status = $1",
        TokenStatus::Verified as _,
        TokenStatus::Expired as _,
        user.user_id
    )
    .execute(&*state.db_pool)
    .await?;
    tracing::info!("User {} logged out successfully", &user.name);
    Ok(Html(r#"
        <div class="text-center py-12">
          <div class="w-16 h-16 bg-teal-100 dark:bg-teal-900 rounded-full flex items-center justify-center mx-auto mb-4">
            <i class="fas fa-sign-out-alt text-2xl text-teal-600 dark:text-teal-400"></i>
          </div>
          <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-4">See you soon!</h2>
          <p class="text-gray-600 dark:text-gray-400 mb-6">You have been successfully logged out.</p>
          <a href="/" class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-teal-600 hover:bg-teal-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-teal-500">
            <i class="fas fa-sign-in-alt mr-2"></i>Login Again
          </a>
        </div>
    "#.into()))
}

//TODO: update queries to only update latest token entry
pub async fn verify(
    headers: HeaderMap,
    State(state): State<AppState>,
    Form(input): Form<VerifyInput>,
) -> Result<(HeaderMap, Html<String>)> {
    let pool = &*state.db_pool;
    let jar = CookieJar::from_headers(&headers);
    let mut headers = HeaderMap::new();
    if let Some(email) = jar.get("_Host_lws_email").filter(|c| !c.value().is_empty()) {
        let user = sqlx::query_as!(
            User,
            "select user_id, email, name from users where email = $1",
            email.value()
        )
        .fetch_one(pool)
        .await?;
        let token = sqlx::query_as!(
            AuthToken,
            r#"select token, user_id, code, expiry, status as "status:_" from tokens where user_id = $1 and status = $2"#,
            user.user_id,
            TokenStatus::Pending as _
        ).fetch_optional(pool).await?;
        tracing::debug!("verifying token: {:?}", &token);
        if let Some(token) = token {
            if input.code != token.code {
                sqlx::query!(
                    "update tokens set status = $2 where user_id = $3 and status = $1",
                    TokenStatus::Pending as _,
                    TokenStatus::Rejected as _,
                    user.user_id
                )
                .execute(pool)
                .await?;
                return Ok((headers, Html(
                r#"<div id='code-error' class='text-red-500 text-center text-sm mt-2'>Invalid code, please try again.</div>"#.to_string()
            )));
            } else {
                sqlx::query!(
                    "UPDATE tokens SET status = $2 WHERE user_id = $3 AND status = $1",
                    TokenStatus::Pending as _,
                    TokenStatus::Verified as _,
                    user.user_id
                )
                .execute(pool)
                .await?;
                headers.insert(
                    SET_COOKIE,
                    HeaderValue::from_str(&format!("_Host_lws_token={}", &token.token))?,
                );
                Ok((
                    headers,
                    Html(
                        "<div class='text-green-600 text-center text-lg'>Verification successful!</div>"
                            .to_string(),
                    ),
                ))
            }
        } else {
            user.issue_token(&state).await?;
            Ok((
                headers,
                Html(
                    "<div class='text-green-600 text-center text-lg'>No active token found, sent new one!</div>"
                        .to_string(),
                ),
            ))
        }
    } else {
        return Ok((headers, Html(
            r#"<div id='code-error' class='text-red-500 text-center text-sm mt-2'>Verification failed, please try again</div>"#.to_string()
        )));
    }
}
