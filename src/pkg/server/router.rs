use axum::{middleware::from_fn_with_state, routing::{get, post}, Router};

use crate::{pkg::middlewares::auth::auth_middleware, prelude::Result, state::AppState};

use super::handlers::{probes::livez, user_mgmt::{initiate_user_registration, verify_user_registration}};

pub async fn build_routes() -> Result<Router> {
    let state = AppState::new().await?;
    Ok(Router::new()
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .route("/livez/", get(livez))
        .route("/register/initiate/", post(initiate_user_registration))
        .route("/register/verify/", get(verify_user_registration))
        .with_state(state))
}
