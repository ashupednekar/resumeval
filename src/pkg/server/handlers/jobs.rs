use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use standard_error::{Interpolate, StandardError};

use crate::{
    pkg::{
        internal::{
            adaptors::jobs::{mutators::JobMutator, selectors::JobSelector, spec::JobEntry},
            ai::{fetch::process, generate::GenerateOps},
            auth::User,
        },
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
    let job = JobMutator::new(&mut tx)
        .create(&user.user_id, input)
        .await?;
    //TODO: pubsub maybe...
    tokio::spawn(async move{
    });
    tx.commit().await?;
    Ok(Json(job))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub title: String,
    pub department: String,
    pub description: String,
    pub requirements: String,
}

pub async fn generate_from_url(
    State(state): State<AppState>,
    Json(input): Json<GenerateJobInput>,
) -> Result<Json<Position>> {
    let jd = process(&input.url).await;
    let prompt = format!(
        r#"
        You are a senior recruiter with immense technical background
        Here's a job description from a typical job board like linkedin\n
        {}\n
        go through it and respond with following json format
        {{
             "title": "the job title",
             "department": "the department in the company",
             "description": "detailed job description",
             "requirements": "job requirements"
         }}

        Note: the extra curly brace is not your concern, ignore that
        NOTE: thee values here are for you to fill, don't just keep them the same
        DO NOT DEVIATE THE FORMAT or break JSON
        "#,
        &jd
    );
    let res = state.ai_client.direct_query(&prompt, None).await?;
    let cleaned_json = res.trim_start_matches("```json").trim_end_matches("```");
    tracing::debug!("AI Result: \n {}", &cleaned_json);
    let position: Position = serde_json::from_str(cleaned_json)?;
    Ok(Json(position))
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<JobEntry>>> {
    let mut tx = state.db_pool.begin().await?;
    let jobs = JobSelector::new(&mut *tx).get_all().await?;
    Ok(Json(jobs))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(_user): Extension<Arc<User>>,
    Json(input): Json<PatchJobInput>,
) -> Result<Json<JobEntry>> {
    let mut tx = state.db_pool.begin().await?;
    let job = JobMutator::new(&mut tx)
        .update(input.id as i32, input)
        .await?;
    tx.commit().await?;

    match job {
        Some(updated_job) => Ok(Json(updated_job)),
        None => Err(StandardError::new("ERR-JOB-001")),
    }
}
