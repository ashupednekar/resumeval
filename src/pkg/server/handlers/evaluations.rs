use std::path::Path;
use std::sync::Arc;
use axum::body::Bytes;
use axum::response::IntoResponse;
use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;
use tokio::io::AsyncWriteExt;
use tokio::{fs, task::JoinSet};
use uuid::Uuid;

use axum::{
    Extension, Json,
    extract::{Multipart, Path as AxumPath, State},
    response::Html,
};
use serde::Serialize;
use standard_error::{Interpolate, StandardError, Status};

use crate::conf::settings;
use crate::pkg::internal::adaptors::resumes::mutators::{CreateResumeData, ResumeMutator};
use crate::pkg::internal::adaptors::resumes::selectors::ResumeSelector;
use crate::pkg::internal::adaptors::resumes::spec::ResumeEntry;
use crate::pkg::internal::ai::index::IndexOps;
use crate::pkg::internal::ai::read::extract_document;
use crate::pkg::internal::minio::S3Ops;
use crate::{
    pkg::{
        internal::{
            adaptors::evaluations::{
                mutators::EvaluationMutator, selectors::EvaluationSelector, spec::EvaluationWithJob,
            },
            auth::User,
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

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<User>>,
    mut multipart: Multipart,
) -> Result<Json<EvaluationTask>> {
    let mut name = String::new();
    let mut job_id_str = String::new();
    let mut resume_files = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| StandardError::new(&format!("EVAL-001: {}", e)))?
    {
        let field_name = field.name().unwrap_or("");
        match field_name {
            "name" => {
                name = field
                    .text()
                    .await
                    .map_err(|e| StandardError::new(&format!("EVAL-002: {}", e)))?;
            }
            "jobId" => {
                job_id_str = field
                    .text()
                    .await
                    .map_err(|e| StandardError::new(&format!("EVAL-003: {}", e)))?;
            }
            "resumes" => {
                let file_name = field.file_name().unwrap_or("unknown").to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| StandardError::new(&format!("EVAL-004: {}", e)).interpolate_err(e.to_string()))?;
                let file_extension = Path::new(&file_name)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if !["pdf", "doc", "docx"].contains(&file_extension.as_str()) {
                    return Err(StandardError::new(
                        "EVAL-006: Invalid file type. Only PDF, DOC, DOCX files are allowed",
                    )
                    .into());
                }
                if data.len() > 10 * 1024 * 1024 {
                    // 10MB limit
                    return Err(StandardError::new(
                        "EVAL-007: File too large. Maximum size is 10MB",
                    )
                    .into());
                }
                resume_files.push((file_name, data));
            }
            _ => {
                let _ = field
                    .bytes()
                    .await
                    .map_err(|e| StandardError::new(&format!("EVAL-005: {}", e)))?;
            }
        }
    }

    let job_id: i32 = job_id_str
        .parse()
        .map_err(|_| StandardError::new("EVAL-008: Invalid job ID"))?;

    let mut tx = state.db_pool.begin().await?;

    let evaluation = EvaluationMutator::new(&mut tx)
        .create(&name, job_id, &user.user_id)
        .await?;

    let upload_dir = format!("uploads/{}", &evaluation.name);
    let mut resumes: Vec<CreateResumeData> = vec![];
    let mut set = JoinSet::new();
    for (original_filename, data) in resume_files {
        let file_id = Uuid::new_v4();
        let file_extension = Path::new(&original_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");
        let filename = format!("{}-{}.{}", &original_filename, file_id, file_extension);
        let file_path = format!("{}/{}", upload_dir, filename);
        let mime_type = match file_extension {
            "pdf" => "application/pdf",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            _ => "application/octet-stream",
        };
        let s3_client = state.s3_client.clone();
        let key = file_path.clone();
        let file_data: Vec<u8> = data.into();
        let data_len = file_data.len();
        let evaluation_id = evaluation.id; 
        set.spawn(async move {
            s3_client.upload_object(
                &settings.s3_bucket_name,
                &key,
                file_data,
                mime_type 
            ).await?;
            Ok::<CreateResumeData, StandardError>(CreateResumeData {
                evaluation_id,
                filename,
                original_filename,
                file_path: key,
                file_size: data_len as i64,
                mime_type: mime_type.into(),
            })
        });
    }
    while let Some(result) = set.join_next().await {
        let resume_data = result
            .map_err(|e| StandardError::new(&format!("EVAL-012: {}", e)))??;
        resumes.push(resume_data);
    }
    let resumes = ResumeMutator::new(&mut tx).bulk_create(resumes).await?;
    for resume in resumes{
        let db_pool = state.db_pool.clone(); 
        let s3_client = state.s3_client.clone(); 
        let ai_client = state.ai_client.clone();
        tokio::spawn(async move{
            let mut tx = db_pool.begin().await?;
            let (data, content_type) =  s3_client.retrieve_object(&settings.s3_bucket_name, &resume.file_path).await?;
            let content = extract_document(data, &content_type)?;
            let content = "this is a good resume";
            match ai_client.index_document(&content).await{
                Ok(embedding) => {
                    tracing::debug!("embeddings created for {}", &resume.filename);
                    ResumeMutator::new(&mut *tx).add_embedding(resume.id, embedding).await?;
                    EvaluationMutator::new(&mut *tx).update_status(evaluation.id, "indexed").await?;
                    tx.commit().await?;
                    Ok::<(), StandardError>(())
                },
                Err(err) => {
                    tracing::error!("error creating embeddings: {}", &err);
                    Ok::<(), StandardError>(())
                }
            }
        });
    }
    // TODO: trigger indexing tasks
    // TODO: trigger evaluation tasks
    let updated_evaluation = EvaluationMutator::new(&mut tx)
        .update_counts(evaluation.id)
        .await?; // TODO: since indexing is async, maybe remove this
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
    let evaluations = EvaluationSelector::new(&mut tx)
        .get_evaluations_for_user(&user.user_id)
        .await?;

    let tasks: Vec<EvaluationTask> = evaluations
        .into_iter()
        .map(|eval| EvaluationTask {
            id: eval.id,
            name: eval.name,
            job_title: "".into(), //eval.job_title, TODO: join to get job id
            status: eval.status,
            total_resumes: eval.total_resumes,
            processed: eval.processed,
            accepted: eval.accepted,
            rejected: eval.rejected,
            pending: eval.pending,
        })
        .collect();

    Ok(Json(tasks))
}

pub async fn details_page(AxumPath(evaluation_id): AxumPath<i32>) -> Result<Html<String>> {
    let template = tokio::fs::read_to_string("templates/evaluation_details.html")
        .await
        .map_err(|e| StandardError::new(&format!("EVAL-TEMPLATE-001: {}", e)))?;

    Ok(Html(template))
}

pub async fn get_details(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<User>>,
    AxumPath(evaluation_id): AxumPath<i32>,
) -> Result<Json<EvaluationDetails>> {
    let mut tx = state.db_pool.begin().await?;
    let evaluations = EvaluationSelector::new(&mut tx)
        .get_evaluations_for_user(&user.user_id)
        .await?;

    let evaluation = evaluations
        .into_iter()
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
) -> Result<Json<Vec<ResumeEntry>>>{
    let mut tx = state.db_pool.begin().await?;

    let evaluation = match EvaluationSelector::new(&mut *tx)
        .get_by_id(evaluation_id)
        .await?{
            Some(eval) => eval,
            None => {
                return Err(StandardError::new("ERR-RESUME-001"))
            }
        };
    if evaluation.created_by != user.user_id{
        return Err(StandardError::new("ERR-RESUME-002").code(StatusCode::FORBIDDEN))
    }
    let documents = ResumeSelector::new(&mut *tx).get_resumes_by_evaluation(evaluation.id).await?;
    Ok(Json(documents))
}

pub async fn retrieve_document(
    State(state): State<AppState>,
    AxumPath(document_id): AxumPath<i32>
) -> Result<impl IntoResponse>{
    let mut tx = state.db_pool.begin().await?;
    let resume = ResumeSelector::new(&mut tx).get_resume_by_id(document_id).await?;
    
    let (file_data, content_type) = state.s3_client
        .retrieve_object(&settings.s3_bucket_name, &resume.file_path)
        .await?;
    tracing::debug!("retrieved {} of type: {}, size: {} bytes", 
        &resume.file_path, &content_type, file_data.len());
    Ok((
        [(CONTENT_TYPE, content_type.to_string())],
        file_data
    ))
}



