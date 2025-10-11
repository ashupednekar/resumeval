use axum::middleware::from_fn_with_state;
use axum::routing::post;
use axum::{Router, routing::get};

use super::handlers;
use super::handlers::auth::{logout, signup, verify};
use super::handlers::probes::{healthz, livez};
use super::handlers::ui::{buckets, containers, functions, home};
use super::middlewares::authn;
use super::state::AppState;
use crate::prelude::Result;

pub async fn build_routes() -> Result<Router> {
    let state = AppState::new().await?;
    let app = Router::new()
        .route("/", get(home))
        .route("/logout", post(logout))
        .route("/project", post(handlers::project::create))
        .route("/project/invite", post(handlers::project::invite))
        .route("/project/accept", get(handlers::project::accept))
        .layer(from_fn_with_state(state.clone(), authn::authenticate))
        .route("/signup", post(signup))
        .route("/verify", post(verify))
        .route("/healthz", get(healthz))
        .route("/livez", get(livez))
        .with_state(state);

    Ok(app)
}
