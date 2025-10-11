use sqlx::PgConnection;
use crate::prelude::Result;
use crate::pkg::internal::adaptors::evaluations::spec::{EvaluationEntry, EvaluationWithJob, ResumeEntry};

pub struct EvaluationSelector<'a> {
    pool: &'a mut PgConnection,
}

impl<'a> EvaluationSelector<'a> {
    pub fn new(pool: &'a mut PgConnection) -> Self {
        EvaluationSelector { pool }
    }

    pub async fn get_by_id(&mut self, id: i32) -> Result<Option<EvaluationEntry>> {
        let row = sqlx::query_as::<_, EvaluationEntry>(
            "SELECT id, name, job_id, created_by, status, total_resumes, processed, accepted, rejected, pending, created_at, updated_at 
             FROM evaluations WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&mut *self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_evaluations_for_user(&mut self, user_id: &str) -> Result<Vec<EvaluationEntry>>{
        let rows = sqlx::query_as::<_, EvaluationEntry>(
            "select id, name, job_id, created_by, status, total_resumes, processed, accepted, rejected, pending, created_at, updated_at from evaluations
            where created_by = $1 order by created_at desc"
        )
            .bind(user_id)
            .fetch_all(&mut *self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn get_resumes_by_evaluation(&mut self, evaluation_id: i32) -> Result<Vec<ResumeEntry>> {
        let rows = sqlx::query_as::<_, ResumeEntry>(
            "SELECT id, evaluation_id, filename, original_filename, file_path, file_size, 
                    mime_type, status, score, feedback, created_at, updated_at 
             FROM resumes WHERE evaluation_id = $1 ORDER BY created_at DESC"
        )
        .bind(evaluation_id)
        .fetch_all(&mut *self.pool)
        .await?;

        Ok(rows)
    }
}
