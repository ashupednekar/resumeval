use std::sync::Arc;

use axum::{Extension, extract::{Multipart, State}, Json};
use serde::Serialize;
use standard_error::StandardError;

use crate::{
    pkg::{
        internal::auth::User,
        server::state::AppState,
    },
    prelude::Result,
};

#[derive(Serialize)]
pub struct EvaluationTask {
    pub id: u32,
    pub name: String,
    pub job_title: String,
    pub status: String,
    pub total_resumes: u32,
    pub processed: u32,
    pub accepted: u32,
    pub rejected: u32,
    pub pending: u32,
}

pub async fn create(
    State(_state): State<AppState>,
    Extension(_user): Extension<Arc<User>>,
    mut multipart: Multipart,
) -> Result<Json<EvaluationTask>> {
    let mut name = String::new();
    let mut _job_id = String::new();
    let mut resume_count = 0;

    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| StandardError::new(&format!("EVAL-001: {}", e)))? {
        let field_name = field.name().unwrap_or("");
        
        match field_name {
            "name" => {
                name = field.text().await.map_err(|e| StandardError::new(&format!("EVAL-002: {}", e)))?;
            }
            "jobId" => {
                _job_id = field.text().await.map_err(|e| StandardError::new(&format!("EVAL-003: {}", e)))?;
            }
            "resumes" => {
                let _file_name = field.file_name().unwrap_or("unknown");
                let _data = field.bytes().await.map_err(|e| StandardError::new(&format!("EVAL-004: {}", e)))?;
                resume_count += 1;
                // In real implementation, would process and store the resume files
            }
            _ => {
                // Skip unknown fields
                let _ = field.bytes().await.map_err(|e| StandardError::new(&format!("EVAL-005: {}", e)))?;
            }
        }
    }

    // Placeholder implementation
    let task = EvaluationTask {
        id: rand::random::<u32>(),
        name,
        job_title: "Selected Job Position".to_string(), // In real implementation, would lookup job by ID
        status: "pending".to_string(),
        total_resumes: resume_count,
        processed: 0,
        accepted: 0,
        rejected: 0,
        pending: resume_count,
    };

    Ok(Json(task))
}

pub async fn list(
    State(_state): State<AppState>,
    Extension(_user): Extension<Arc<User>>,
) -> Result<Json<Vec<EvaluationTask>>> {
    // Placeholder hardcoded evaluation tasks
    let tasks = vec![
        EvaluationTask {
            id: 1,
            name: "Frontend Dev Batch #1".to_string(),
            job_title: "Senior Frontend Developer".to_string(),
            status: "completed".to_string(),
            total_resumes: 25,
            processed: 25,
            accepted: 8,
            rejected: 12,
            pending: 5,
        },
        EvaluationTask {
            id: 2,
            name: "PM Candidates Q1".to_string(),
            job_title: "Product Manager".to_string(),
            status: "processing".to_string(),
            total_resumes: 15,
            processed: 10,
            accepted: 3,
            rejected: 4,
            pending: 8,
        },
    ];

    Ok(Json(tasks))
}