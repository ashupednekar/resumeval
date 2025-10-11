use std::sync::Arc;

use askama::Template;
use axum::{Extension, extract::State, response::Html};

use crate::{
    pkg::{
        internal::{auth::User, project::Project},
        server::{
            state::AppState,
            uispec::Home,
        },
    },
    prelude::Result,
};

pub async fn home(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<User>>,
) -> Result<Html<String>> {
    let projects = Project::list(&state, &user.user_id).await?;
    tracing::debug!("projects: {:?}", &projects);
    let template = Home {
        username: &user.name,
        projects,
    };

    Ok(Html(template.render()?))
}
