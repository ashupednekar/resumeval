use std::sync::Arc;

use axum::{
    Extension, Form, Json,
    extract::{Query, State},
    http::HeaderMap,
    response::{Html, Redirect},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use serde_json::{Value, json};
use standard_error::StandardError;
use validator::Validate;

use crate::{
    pkg::{
        internal::{
            auth::User,
            email::SendEmail,
            project::{AccessInvite, Project},
        },
        server::state::AppState,
    },
    prelude::Result,
};

#[derive(Deserialize, Validate)]
pub struct ProjectInput {
    #[validate(length(min = 1, message = "Field cannot be empty"))]
    pub name: String,
    #[validate(length(min = 1, message = "Field cannot be empty"))]
    pub description: String,
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<User>>,
    Form(input): Form<ProjectInput>,
) -> Result<Redirect> {
    Project::create(&state, &input.name, &input.description, &user.user_id).await?;
    Ok(Redirect::permanent("/"))
}

#[derive(Deserialize, Validate)]
pub struct InviteInput {
    #[validate(length(min = 1, message = "Field cannot be empty"))]
    pub email: String,
}

pub async fn invite(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(me): Extension<Arc<User>>,
    Json(input): Json<InviteInput>,
) -> Result<Json<Value>> {
    let jar = CookieJar::from_headers(&headers);
    let project_id = match jar.get("current_project").filter(|c| !c.value().is_empty()) {
        Some(p) => p.value(),
        None => {
            return Err(StandardError::new("ERR-PROJ-001"));
        }
    };
    let project = Project::retrieve(&state, project_id).await?;
    let user = match User::retrieve(&state, &input.email).await? {
        Some(u) => u,
        None => {
            let (name, _) = input.email.split_once("@").unwrap_or(("unknown", ""));
            User::create(&state, &input.email, &name).await?
        }
    };
    tracing::info!("inviting {} to {}", &user.name, &project.name);
    let mut txn = state.db_pool.begin().await?;
    let invite = project.invite(&mut *txn, &user.user_id, &me.user_id).await?;
    invite.details(&state).await?.send(&user.email)?;
    Ok(Json(json!({
        "code": invite.invite_id
    })))
}


#[derive(Deserialize)]
pub struct AcceptQuery{
    pub invite_code: String
}

pub async fn accept(
    State(state): State<AppState>,
    Query(params): Query<AcceptQuery>,
    Extension(user): Extension<Arc<User>>,
) -> Result<Redirect> {
    let mut txn = state.db_pool.begin().await?;
    let invite = AccessInvite::new(&state, &params.invite_code).await?;
    invite.accept(&mut *txn).await?;
    tracing::info!("{} accepted invite code - {}", &user.name, &params.invite_code);
    Ok(Redirect::permanent("/"))
}
