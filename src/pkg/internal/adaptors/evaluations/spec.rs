use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EvaluationEntry {
    pub id: i32,
    pub name: String,
    pub job_id: i32,
    pub created_by: String,
    pub status: String,
    pub total_resumes: i32,
    pub processed: i32,
    pub accepted: i32,
    pub rejected: i32,
    pub pending: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationWithJob {
    pub id: i32,
    pub name: String,
    pub job_id: i32,
    pub job_title: String,
    pub created_by: String,
    pub status: String,
    pub total_resumes: i32,
    pub processed: i32,
    pub accepted: i32,
    pub rejected: i32,
    pub pending: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
