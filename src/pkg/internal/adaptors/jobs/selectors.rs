use sqlx::PgConnection;

use crate::{prelude::Result, internal::adaptors::jobs::spec::JobEntry};


pub struct JobSelector<'a>{
    pool: &'a mut PgConnection
}

impl<'a> JobSelector<'a>{
    pub fn new(pool: &'a mut PgConnection) -> Self {
        JobSelector{pool}
    }

    pub async fn get_by_id(&mut self, id: i32) -> Result<Option<JobEntry>> {
        let row = sqlx::query_as::<_, JobEntry>(
            "SELECT id, title, department, description, requirements, url, created_at, updated_at 
             FROM jobs WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&mut *self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_all(&mut self) -> Result<Vec<JobEntry>> {
        let rows = sqlx::query_as::<_, JobEntry>(
            "SELECT id, title, department, description, requirements, url, created_at, updated_at 
             FROM jobs ORDER BY created_at DESC"
        )
        .fetch_all(&mut *self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_by_department(&mut self, department: &str) -> Result<Vec<JobEntry>> {
        let rows = sqlx::query_as::<_, JobEntry>(
            "SELECT id, title, department, description, requirements, url, created_at, updated_at 
             FROM jobs WHERE department = $1 ORDER BY created_at DESC"
        )
        .bind(department)
        .fetch_all(&mut *self.pool)
        .await?;
        Ok(rows)
    }


}

