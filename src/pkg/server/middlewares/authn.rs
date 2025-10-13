use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use standard_error::{HtmlRes, StandardError, Status};

use crate::{
    pkg::{
        internal::auth::{AuthToken, User},
        server::{state::{AppState, GetTxn}, uispec::Verify},
    },
    prelude::Result,
};

pub async fn authenticate(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response> {
    let mut tx = state.db_pool.begin_txn().await?;
    let jar = CookieJar::from_headers(&headers);
    let maybe_cookie = jar.get("_Host_token").filter(|c| !c.value().is_empty());
    if let Some(cookie) = maybe_cookie {
        match AuthToken::check_token_validity(&state, cookie.value()).await {
            Ok(user) => {
                request.extensions_mut().insert(Arc::new(user));
                return Ok(next.run(request).await);
            }
            Err(_) => {}
        }
    }
    tracing::warn!("token missing, authentication denied");
    if let Some(email) = jar.get("_Host_email").filter(|c| !c.value().is_empty()) {
        if let Some(user) = sqlx::query_as!(
            User,
            "select user_id, email, name from users where email = $1",
            email.value()
        )
        .fetch_optional(&mut *tx)
        .await?
        {
            user.issue_token(&state).await?;
        };
    }
    Err(StandardError::new("ERR-AUTH-001")
        .code(StatusCode::UNAUTHORIZED)
        .template(Verify {}.render()?))
}
