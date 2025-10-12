use crate::pkg::internal::adaptors::resumes::spec::ResumeEntry;
use crate::prelude::Result;
use sqlx::PgConnection;

pub struct ResumeSelector<'a> {
    pool: &'a mut PgConnection,
}

impl<'a> ResumeSelector<'a> {
    pub fn new(pool: &'a mut PgConnection) -> Self {
        ResumeSelector { pool }
    }
  
    pub async fn get_resume_by_id(
        &mut self,
        resume_id: i32,
    ) -> Result<ResumeEntry> {
        let rows = sqlx::query_as::<_, ResumeEntry>(
            "SELECT id, evaluation_id, filename, original_filename, file_path, file_size, 
                    mime_type, status, score, feedback, created_at, updated_at 
             FROM resumes WHERE id = $1 ORDER BY created_at DESC",
        )
        .bind(resume_id)
        .fetch_one(&mut *self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_resumes_by_evaluation(
        &mut self,
        evaluation_id: i32,
    ) -> Result<Vec<ResumeEntry>> {
        let rows = sqlx::query_as::<_, ResumeEntry>(
            "SELECT id, evaluation_id, filename, original_filename, file_path, file_size, 
                    mime_type, status, score, feedback, created_at, updated_at 
             FROM resumes WHERE evaluation_id = $1 ORDER BY created_at DESC",
        )
        .bind(evaluation_id)
        .fetch_all(&mut *self.pool)
        .await?;

        Ok(rows)
    }
}
