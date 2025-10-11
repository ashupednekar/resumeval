use std::sync::Arc;
use std::path::Path;
use uuid::Uuid;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use axum::{Extension, extract::{Multipart, State}, Json};
use serde::Serialize;
use standard_error::StandardError;

use crate::{
    pkg::{
        internal::{
            auth::User,
            adaptors::evaluations::{
                selectors::EvaluationSelector,
                mutators::{EvaluationMutator, ResumeMutator},
                spec::EvaluationWithJob,
            },
        },
        server::state::AppState,
    },
    prelude::Result,
};

#[derive(Serialize)]
pub struct EvaluationTask {
    pub id: i32,
    pub name: String,
    pub job_title: String,
    pub status: String,
    pub total_resumes: i32,
    pub processed: i32,
    pub accepted: i32,
    pub rejected: i32,
    pub pending: i32,
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<User>>,
    mut multipart: Multipart,
) -> Result<Json<EvaluationTask>> {
    let mut name = String::new();
    let mut job_id_str = String::new();
    let mut resume_files = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| StandardError::new(&format!("EVAL-001: {}", e)))? {
        let field_name = field.name().unwrap_or("");
        match field_name {
            "name" => {
                name = field.text().await.map_err(|e| StandardError::new(&format!("EVAL-002: {}", e)))?;
            }
            "jobId" => {
                job_id_str = field.text().await.map_err(|e| StandardError::new(&format!("EVAL-003: {}", e)))?;
            }
            "resumes" => {
                let file_name = field.file_name().unwrap_or("unknown").to_string();
                let data = field.bytes().await.map_err(|e| StandardError::new(&format!("EVAL-004: {}", e)))?;
                let file_extension = Path::new(&file_name)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                
                if !["pdf", "doc", "docx"].contains(&file_extension.as_str()) {
                    return Err(StandardError::new("EVAL-006: Invalid file type. Only PDF, DOC, DOCX files are allowed").into());
                }
                
                if data.len() > 10 * 1024 * 1024 { // 10MB limit
                    return Err(StandardError::new("EVAL-007: File too large. Maximum size is 10MB").into());
                }
                
                resume_files.push((file_name, data));
            }
            _ => {
                let _ = field.bytes().await.map_err(|e| StandardError::new(&format!("EVAL-005: {}", e)))?;
            }
        }
    }

    let job_id: i32 = job_id_str.parse().map_err(|_| StandardError::new("EVAL-008: Invalid job ID"))?;

    let mut tx = state.db_pool.begin().await?;
    
    let evaluation = EvaluationMutator::new(&mut tx).create(&name, job_id, &user.user_id).await?;
    
    let upload_dir = "uploads/resumes";
    fs::create_dir_all(upload_dir).await.map_err(|e| StandardError::new(&format!("EVAL-009: {}", e)))?;
    
    for (original_filename, data) in resume_files {
        let file_id = Uuid::new_v4();
        let file_extension = Path::new(&original_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");
        let filename = format!("{}.{}", file_id, file_extension);
        let file_path = format!("{}/{}", upload_dir, filename);
        
        let mut file = fs::File::create(&file_path).await.map_err(|e| StandardError::new(&format!("EVAL-010: {}", e)))?;
        file.write_all(&data).await.map_err(|e| StandardError::new(&format!("EVAL-011: {}", e)))?;
        
        let mime_type = match file_extension {
            "pdf" => "application/pdf",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            _ => "application/octet-stream",
        };
        
        ResumeMutator::new(&mut tx).create(
            evaluation.id,
            &filename,
            &original_filename,
            &file_path,
            data.len() as i64,
            mime_type,
        ).await?;
    }
    
    let updated_evaluation = EvaluationMutator::new(&mut tx).update_counts(evaluation.id).await?;
    
    tx.commit().await?;

    let task = EvaluationTask {
        id: updated_evaluation.id,
        name: updated_evaluation.name,
        job_title: "Selected Job Position".to_string(), 
        status: updated_evaluation.status,
        total_resumes: updated_evaluation.total_resumes,
        processed: updated_evaluation.processed,
        accepted: updated_evaluation.accepted,
        rejected: updated_evaluation.rejected,
        pending: updated_evaluation.pending,
    };

    Ok(Json(task))
}

pub async fn list(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<User>>,
) -> Result<Json<Vec<EvaluationTask>>> {
    let mut tx = state.db_pool.begin().await?;
    let evaluations = EvaluationSelector::new(&mut tx).get_evaluations_for_user(&user.user_id).await?;
    
    let tasks: Vec<EvaluationTask> = evaluations.into_iter().map(|eval| EvaluationTask {
        id: eval.id,
        name: eval.name,
        job_title: "".into(),//eval.job_title, TODO: join to get job id
        status: eval.status,
        total_resumes: eval.total_resumes,
        processed: eval.processed,
        accepted: eval.accepted,
        rejected: eval.rejected,
        pending: eval.pending,
    }).collect();

    Ok(Json(tasks))
}
