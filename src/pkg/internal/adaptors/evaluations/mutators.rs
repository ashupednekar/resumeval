use sqlx::PgConnection;
use crate::prelude::Result;
use crate::pkg::internal::adaptors::evaluations::spec::{EvaluationEntry, ResumeEntry};

pub struct EvaluationMutator<'a> {
    pool: &'a mut PgConnection,
}

impl<'a> EvaluationMutator<'a> {
    pub fn new(pool: &'a mut PgConnection) -> Self {
        EvaluationMutator { pool }
    }

    pub async fn create(&mut self, name: &str, job_id: i32, created_by: &str) -> Result<EvaluationEntry> {
        let row = sqlx::query_as::<_, EvaluationEntry>(
            r#"
            INSERT INTO evaluations (name, job_id, created_by, status, total_resumes, processed, accepted, rejected, pending)
            VALUES ($1, $2, $3, 'pending', 0, 0, 0, 0, 0)
            RETURNING id, name, job_id, created_by, status, total_resumes, processed, accepted, rejected, pending, created_at, updated_at
            "#
        )
        .bind(name)
        .bind(job_id)
        .bind(created_by)
        .fetch_one(&mut *self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update_counts(&mut self, evaluation_id: i32) -> Result<EvaluationEntry> {
        let row = sqlx::query_as::<_, EvaluationEntry>(
            r#"
            UPDATE evaluations 
            SET 
                total_resumes = (SELECT COUNT(*) FROM resumes WHERE evaluation_id = $1),
                processed = (SELECT COUNT(*) FROM resumes WHERE evaluation_id = $1 AND status != 'pending'),
                accepted = (SELECT COUNT(*) FROM resumes WHERE evaluation_id = $1 AND status = 'accepted'),
                rejected = (SELECT COUNT(*) FROM resumes WHERE evaluation_id = $1 AND status = 'rejected'),
                pending = (SELECT COUNT(*) FROM resumes WHERE evaluation_id = $1 AND status = 'pending'),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, name, job_id, created_by, status, total_resumes, processed, accepted, rejected, pending, created_at, updated_at
            "#
        )
        .bind(evaluation_id)
        .fetch_one(&mut *self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update_status(&mut self, evaluation_id: i32, status: &str) -> Result<EvaluationEntry> {
        let row = sqlx::query_as::<_, EvaluationEntry>(
            r#"
            UPDATE evaluations 
            SET status = $2, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, name, job_id, created_by, status, total_resumes, processed, accepted, rejected, pending, created_at, updated_at
            "#
        )
        .bind(evaluation_id)
        .bind(status)
        .fetch_one(&mut *self.pool)
        .await?;

        Ok(row)
    }
}

pub struct ResumeMutator<'a> {
    pool: &'a mut PgConnection,
}

impl<'a> ResumeMutator<'a> {
    pub fn new(pool: &'a mut PgConnection) -> Self {
        ResumeMutator { pool }
    }

    pub async fn create(&mut self, evaluation_id: i32, filename: &str, original_filename: &str, 
                       file_path: &str, file_size: i64, mime_type: &str) -> Result<ResumeEntry> {
        let row = sqlx::query_as::<_, ResumeEntry>(
            r#"
            INSERT INTO resumes (evaluation_id, filename, original_filename, file_path, file_size, mime_type, status)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending')
            RETURNING id, evaluation_id, filename, original_filename, file_path, file_size, mime_type, status, score, feedback, created_at, updated_at
            "#
        )
        .bind(evaluation_id)
        .bind(filename)
        .bind(original_filename)
        .bind(file_path)
        .bind(file_size)
        .bind(mime_type)
        .fetch_one(&mut *self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update_status(&mut self, resume_id: i32, status: &str, score: Option<f64>, feedback: Option<&str>) -> Result<ResumeEntry> {
        let row = sqlx::query_as::<_, ResumeEntry>(
            r#"
            UPDATE resumes 
            SET status = $2, score = $3, feedback = $4, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, evaluation_id, filename, original_filename, file_path, file_size, mime_type, status, score, feedback, created_at, updated_at
            "#
        )
        .bind(resume_id)
        .bind(status)
        .bind(score)
        .bind(feedback)
        .fetch_one(&mut *self.pool)
        .await?;

        Ok(row)
    }
}