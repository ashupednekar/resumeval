use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    pkg::{
        internal::{adaptors::jobs::{mutators::JobMutator, spec::JobEntry}, auth::User},
        server::state::AppState,
    },
    prelude::Result,
};

#[derive(Deserialize)]
pub struct CreateJobInput {
    pub title: String,
    pub department: String,
    pub description: String,
    pub requirements: String,
    pub url: Option<String>,
}

#[derive(Deserialize)]
pub struct PatchJobInput {
    pub id: u32,
    pub title: Option<String>,
    pub department: Option<String>,
    pub description: Option<String>,
    pub requirements: Option<String>,
    pub url: Option<String>,
}


#[derive(Deserialize)]
pub struct GenerateJobInput {
    pub url: String,
}


pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<User>>,
    Json(input): Json<CreateJobInput>,
) -> Result<Json<JobEntry>> {
    let mut tx = state.db_pool.begin().await?;
    let job = JobMutator::new(&mut tx).create(&user.user_id, input).await?;
    Ok(Json(job))
}

pub async fn generate_from_url(
    State(_state): State<AppState>,
    Extension(_user): Extension<Arc<User>>,
    Json(_input): Json<GenerateJobInput>,
) -> Result<Json<Value>> {
    // Placeholder AI generation - in reality this would call an AI service
    let generated = json!({
        "title": "AI Generated Position",
        "department": "Technology",
        "description": "This is an AI-generated job description based on the provided URL. In a real implementation, this would analyze the URL content and generate appropriate job details using AI/ML services.",
        "requirements": "• Bachelor's degree in relevant field\n• 3+ years of experience\n• Strong communication skills\n• Experience with modern technologies"
    });

    Ok(Json(generated))
}

pub async fn list(
    State(_state): State<AppState>,
    Extension(_user): Extension<Arc<User>>,
) -> Result<Json<Vec<JobEntry>>> {
    // Placeholder hardcoded jobs
    Ok(Json(vec![]))
}
