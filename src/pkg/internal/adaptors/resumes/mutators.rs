use crate::{pkg::internal::adaptors::resumes::spec::ResumeEntry, prelude::Result};
use pgvector::Vector;
use sqlx::{types::BigDecimal, PgConnection};

pub struct CreateResumeData {
    pub evaluation_id: i32,
    pub filename: String,
    pub original_filename: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
}

pub struct ResumeMutator<'a> {
    pool: &'a mut PgConnection,
}

impl<'a> ResumeMutator<'a> {
    pub fn new(pool: &'a mut PgConnection) -> Self {
        ResumeMutator { pool }
    }

    pub async fn bulk_create(
        &mut self,
        resumes: Vec<CreateResumeData>,
    ) -> Result<Vec<ResumeEntry>> {
        if resumes.is_empty() {
            return Ok(Vec::new());
        }
        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO resumes (evaluation_id, filename, original_filename, file_path, file_size, mime_type, status) ",
        );
        query_builder.push_values(resumes, |mut b, resume| {
            b.push_bind(resume.evaluation_id)
                .push_bind(resume.filename)
                .push_bind(resume.original_filename)
                .push_bind(resume.file_path)
                .push_bind(resume.file_size)
                .push_bind(resume.mime_type)
                .push_bind("pending");
        });
        query_builder.push(
            " RETURNING id, evaluation_id, filename, original_filename, file_path, file_size, mime_type, status, score, feedback, created_at, updated_at"
        );
        let rows = query_builder
            .build_query_as::<ResumeEntry>()
            .fetch_all(&mut *self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn add_embedding(
        &mut self,
        resume_id: i32,
        embedding: Vector
    ) -> Result<ResumeEntry> {
        let row = sqlx::query_as::<_, ResumeEntry>(
            r#"
            UPDATE resumes 
            SET embedding = $2, status='indexed', updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, evaluation_id, filename, original_filename, file_path, file_size, mime_type, status, score, feedback, created_at, updated_at
            "#
        )
        .bind(resume_id)
        .bind(&embedding)
        .fetch_one(&mut *self.pool)
        .await?;
        Ok(row)
    }


    pub async fn add_verdict(
        &mut self,
        resume_id: i32,
        status: &str,
        score: Option<&str>,
        feedback: Option<&str>,
    ) -> Result<ResumeEntry> {
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
