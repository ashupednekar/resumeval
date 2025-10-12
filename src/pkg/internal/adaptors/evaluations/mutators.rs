use crate::pkg::internal::adaptors::evaluations::spec::EvaluationEntry;
use crate::prelude::Result;
use sqlx::PgConnection;

pub struct EvaluationMutator<'a> {
    pool: &'a mut PgConnection,
}

impl<'a> EvaluationMutator<'a> {
    pub fn new(pool: &'a mut PgConnection) -> Self {
        EvaluationMutator { pool }
    }

    pub async fn create(
        &mut self,
        name: &str,
        job_id: i32,
        created_by: &str,
    ) -> Result<EvaluationEntry> {
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

    pub async fn update_status(
        &mut self,
        evaluation_id: i32,
        status: &str,
    ) -> Result<EvaluationEntry> {
        let row = sqlx::query_as::<_, EvaluationEntry>(
            r#"
            update evaluations 
            set status = $2, updated_at = current_timestamp
            where id = $1
            returning id, name, job_id, created_by, status, total_resumes, processed, accepted, rejected, pending, created_at, updated_at
            "#
        )
        .bind(evaluation_id)
        .bind(status)
        .fetch_one(&mut *self.pool)
        .await?;
        Ok(row)
    }

    pub async fn set_pending(
        &mut self,
        evaluation_id: i32,
        pending: i32
    ) -> Result<EvaluationEntry> {
        let row = sqlx::query_as::<_, EvaluationEntry>(
            r#"
            update evaluations 
            set pending = $2, updated_at = current_timestamp
            where id = $1
            returning id, name, job_id, created_by, status, total_resumes, processed, accepted, rejected, pending, created_at, updated_at
            "#
        )
        .bind(evaluation_id)
        .bind(pending)
        .fetch_one(&mut *self.pool)
        .await?;
        Ok(row)
    }

}
