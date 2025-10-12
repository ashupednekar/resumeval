use crate::pkg::internal::adaptors::evaluations::spec::EvaluationEntry;
use crate::prelude::Result;
use sqlx::PgConnection;

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

    pub async fn get_evaluations_for_user(
        &mut self,
        user_id: &str,
    ) -> Result<Vec<EvaluationEntry>> {
        let rows = sqlx::query_as::<_, EvaluationEntry>(
            "select id, name, job_id, created_by, status, total_resumes, processed, accepted, rejected, pending, created_at, updated_at from evaluations
            where created_by = $1 order by created_at desc"
        )
            .bind(user_id)
            .fetch_all(&mut *self.pool)
            .await?;
        Ok(rows)
    }
}
