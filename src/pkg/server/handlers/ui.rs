use std::sync::Arc;

use askama::Template;
use axum::{Extension, extract::State, response::Html};

use crate::{
    pkg::{
        internal::{auth::User, project::Project},
        server::{
            state::AppState,
            uispec::{Buckets, Containers, Functions, Home, Metrics},
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
    let metrics = Metrics {
        containers: 2,
        functions: 5,
        buckets: 3,
        total_requests: 1200000,
    };

    let template = Home {
        username: &user.name,
        projects,
        metrics,
    };

    Ok(Html(template.render()?))
}
