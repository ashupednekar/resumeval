use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    pkg::{
        internal::auth::User,
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
    pub ai_url: Option<String>,
}

#[derive(Deserialize)]
pub struct GenerateJobInput {
    pub url: String,
}

#[derive(Serialize)]
pub struct Job {
    pub id: u32,
    pub title: String,
    pub department: String,
    pub description: String,
    pub requirements: String,
    pub status: String,
    pub applicants: u32,
    pub created: String,
}

pub async fn create(
    State(_state): State<AppState>,
    Extension(_user): Extension<Arc<User>>,
    Json(input): Json<CreateJobInput>,
) -> Result<Json<Job>> {
    // Placeholder implementation with hardcoded data
    let job = Job {
        id: rand::random::<u32>(),
        title: input.title,
        department: input.department,
        description: input.description,
        requirements: input.requirements,
        status: "active".to_string(),
        applicants: 0,
        created: "just now".to_string(),
    };

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
) -> Result<Json<Vec<Job>>> {
    // Placeholder hardcoded jobs
    let jobs = vec![
        Job {
            id: 1,
            title: "Senior Frontend Developer".to_string(),
            department: "Engineering".to_string(),
            description: "We are looking for a senior frontend developer to join our team...".to_string(),
            requirements: "React, TypeScript, 5+ years experience".to_string(),
            status: "active".to_string(),
            applicants: 12,
            created: "2 days ago".to_string(),
        },
        Job {
            id: 2,
            title: "Product Manager".to_string(),
            department: "Product".to_string(),
            description: "Lead product strategy and development...".to_string(),
            requirements: "MBA, 3+ years PM experience".to_string(),
            status: "active".to_string(),
            applicants: 8,
            created: "1 week ago".to_string(),
        },
    ];

    Ok(Json(jobs))
}
