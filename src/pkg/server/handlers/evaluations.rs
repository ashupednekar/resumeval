use std::sync::Arc;
use std::path::Path;
use uuid::Uuid;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use axum::{Extension, extract::{Multipart, State, Path as AxumPath}, Json, response::Html};
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

#[derive(Serialize)]
pub struct EvaluationDetails {
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

#[derive(Serialize)]
pub struct DocumentInfo {
    pub id: i32,
    pub filename: String,
    pub original_filename: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub evaluation_status: Option<String>,
    pub indexing_status: String,
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

pub async fn details_page(
    AxumPath(evaluation_id): AxumPath<i32>,
) -> Result<Html<String>> {
    let template = tokio::fs::read_to_string("templates/evaluation_details.html").await
        .map_err(|e| StandardError::new(&format!("EVAL-TEMPLATE-001: {}", e)))?;
    
    Ok(Html(template))
}

pub async fn get_details(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<User>>,
    AxumPath(evaluation_id): AxumPath<i32>,
) -> Result<Json<EvaluationDetails>> {
    let mut tx = state.db_pool.begin().await?;
    let evaluations = EvaluationSelector::new(&mut tx).get_evaluations_for_user(&user.user_id).await?;
    
    let evaluation = evaluations.into_iter()
        .find(|eval| eval.id == evaluation_id)
        .ok_or_else(|| StandardError::new("EVAL-404: Evaluation not found"))?;
    
    let details = EvaluationDetails {
        id: evaluation.id,
        name: evaluation.name,
        job_title: "Sample Job Position".to_string(), // Placeholder
        status: evaluation.status,
        total_resumes: evaluation.total_resumes,
        processed: evaluation.processed,
        accepted: evaluation.accepted,
        rejected: evaluation.rejected,
        pending: evaluation.pending,
    };

    Ok(Json(details))
}

pub async fn get_documents(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<User>>,
    AxumPath(evaluation_id): AxumPath<i32>,
) -> Result<Json<Vec<DocumentInfo>>> {
    let mut tx = state.db_pool.begin().await?;
    
    // First verify the evaluation belongs to the user
    let evaluations = EvaluationSelector::new(&mut tx).get_evaluations_for_user(&user.user_id).await?;
    let _evaluation = evaluations.into_iter()
        .find(|eval| eval.id == evaluation_id)
        .ok_or_else(|| StandardError::new("EVAL-404: Evaluation not found"))?;
    
    // Mock document data - replace with actual database query
    let documents = vec![
        DocumentInfo {
            id: 1,
            filename: "resume_001.pdf".to_string(),
            original_filename: "john_doe_resume.pdf".to_string(),
            file_path: "/uploads/resumes/resume_001.pdf".to_string(),
            file_size: 245760, // ~240KB
            mime_type: "application/pdf".to_string(),
            evaluation_status: Some("pending".to_string()),
            indexing_status: "completed".to_string(),
        },
        DocumentInfo {
            id: 2,
            filename: "resume_002.pdf".to_string(),
            original_filename: "jane_smith_cv.pdf".to_string(),
            file_path: "/uploads/resumes/resume_002.pdf".to_string(),
            file_size: 312450, // ~305KB
            mime_type: "application/pdf".to_string(),
            evaluation_status: Some("accepted".to_string()),
            indexing_status: "completed".to_string(),
        },
        DocumentInfo {
            id: 3,
            filename: "resume_003.pdf".to_string(),
            original_filename: "mike_johnson_resume.pdf".to_string(),
            file_path: "/uploads/resumes/resume_003.pdf".to_string(),
            file_size: 189320, // ~185KB
            mime_type: "application/pdf".to_string(),
            evaluation_status: Some("rejected".to_string()),
            indexing_status: "completed".to_string(),
        },
        DocumentInfo {
            id: 4,
            filename: "resume_004.pdf".to_string(),
            original_filename: "sarah_wilson_cv.pdf".to_string(),
            file_path: "/uploads/resumes/resume_004.pdf".to_string(),
            file_size: 278900, // ~272KB
            mime_type: "application/pdf".to_string(),
            evaluation_status: Some("processing".to_string()),
            indexing_status: "processing".to_string(),
        },
        DocumentInfo {
            id: 5,
            filename: "resume_005.pdf".to_string(),
            original_filename: "alex_brown_resume.pdf".to_string(),
            file_path: "/uploads/resumes/resume_005.pdf".to_string(),
            file_size: 156780, // ~153KB
            mime_type: "application/pdf".to_string(),
            evaluation_status: None,
            indexing_status: "pending".to_string(),
        },
    ];

    Ok(Json(documents))
}

pub async fn view_document(
    AxumPath(document_id): AxumPath<i32>,
) -> Result<Html<String>> {
    // Mock response - replace with actual document serving logic
    let content = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Document Viewer</title>
        </head>
        <body>
            <h1>Document Viewer</h1>
            <p>Viewing document ID: {}</p>
            <p>This is a placeholder for document viewing functionality.</p>
            <p>In a real implementation, this would serve the actual PDF or document content.</p>
        </body>
        </html>
        "#,
        document_id
    );
    
    Ok(Html(content))
}

pub async fn download_document(
    AxumPath(document_id): AxumPath<i32>,
) -> Result<Html<String>> {
    // Mock response - replace with actual file download logic
    let content = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Download Document</title>
        </head>
        <body>
            <h1>Download Document</h1>
            <p>Download for document ID: {}</p>
            <p>This is a placeholder for document download functionality.</p>
            <p>In a real implementation, this would serve the actual file for download.</p>
        </body>
        </html>
        "#,
        document_id
    );
    
    Ok(Html(content))
}
