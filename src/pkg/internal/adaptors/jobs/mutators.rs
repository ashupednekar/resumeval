use sqlx::PgConnection;
use crate::pkg::internal::adaptors::jobs::spec::JobEntry;
use crate::prelude::Result;
use crate::pkg::server::handlers::jobs::{CreateJobInput, PatchJobInput};


pub struct JobMutator<'a>{
    pool: &'a mut PgConnection
}

impl<'a> JobMutator<'a>{
    pub fn new(pool: &'a mut PgConnection) -> Self {
        JobMutator{pool}
    }

    pub async fn create(&mut self, job: CreateJobInput) -> Result<JobEntry> {
        let row = sqlx::query_as::<_, JobEntry>(
            r#"
            INSERT INTO jobs (title, department, description, requirements, url)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, title, department, description, requirements, url, created_at, updated_at
            "#
        )
        .bind(&job.title)
        .bind(&job.department)
        .bind(&job.description)
        .bind(&job.requirements)
        .bind(&job.url)
        .fetch_one(&mut *self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update(&mut self, id: i32, job: PatchJobInput) -> Result<Option<JobEntry>> {
        let mut query = String::from("UPDATE jobs SET updated_at = CURRENT_TIMESTAMP");
        let mut param_count = 1;

        if job.title.is_some() {
            param_count += 1;
            query.push_str(&format!(", title = ${}", param_count));
        }
        if job.department.is_some() {
            param_count += 1;
            query.push_str(&format!(", department = ${}", param_count));
        }
        if job.description.is_some() {
            param_count += 1;
            query.push_str(&format!(", description = ${}", param_count));
        }
        if job.requirements.is_some() {
            param_count += 1;
            query.push_str(&format!(", requirements = ${}", param_count));
        }
        if job.url.is_some() {
            param_count += 1;
            query.push_str(&format!(", url = ${}", param_count));
        }

        query.push_str(" WHERE id = $1 RETURNING id, title, department, description, requirements, url, created_at, updated_at");

        let mut q = sqlx::query_as::<_, JobEntry>(&query).bind(id);

        if let Some(title) = job.title {
            q = q.bind(title);
        }
        if let Some(dept) = job.department {
            q = q.bind(dept);
        }
        if let Some(desc) = job.description {
            q = q.bind(desc);
        }
        if let Some(reqs) = job.requirements {
            q = q.bind(reqs);
        }
        if let Some(url) = job.url {
            q = q.bind(url);
        }
        let row = q.fetch_optional(&mut *self.pool).await?;
        Ok(row)
    }

    pub async fn delete(&mut self, id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM jobs WHERE id = $1")
            .bind(id)
            .execute(&mut *self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_by_department(&mut self, department: &str) -> Result<u64> {
        let result = sqlx::query("DELETE FROM jobs WHERE department = $1")
            .bind(department)
            .execute(&mut *self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
