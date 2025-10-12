use std::sync::Arc;

use ai::{
    chat_completions::{ChatCompletion, ChatCompletionMessage, ChatCompletionRequestBuilder}, clients::openai::Client, embeddings::{Embeddings, EmbeddingsRequestBuilder}
};
use pgvector::Vector;
use sqlx::PgConnection;
use standard_error::{Interpolate, StandardError};

use crate::pkg::internal::ai::spec::Document;
use crate::{conf::settings, prelude::Result};

#[async_trait::async_trait]
pub trait GenerateOps {
    async fn direct_query(
        &self,
        query: &str,
        context: Option<&str>,
    ) -> Result<String>;

    async fn rag_query(
        &self,
        tx: &mut PgConnection,
        query: &str,
    ) -> Result<String>;

}

#[async_trait::async_trait]
impl GenerateOps for Arc<Client>{

    async fn direct_query(
        &self,
        query: &str,
        context: Option<&str>,
    ) -> Result<String> {
        let prompt = format!(
            "Context:\n{}\n\nQuestion: {}\n\nAnswer based on the context above:",
            context.unwrap_or(""),
            query
        );
        let request = ChatCompletionRequestBuilder::default()
            .model(&settings.ai_model)
            .messages(vec![ChatCompletionMessage::User(prompt.into())])
            .build()
            .map_err(|e| StandardError::new("ERR-AI-001").interpolate_err(e.to_string()))?;
        let response = self 
            .chat_completions(&request)
            .await
            .map_err(|e| StandardError::new("ERR-AI-002").interpolate_err(e.to_string()))?;
        let answer = response.choices[0]
            .message
            .content
            .as_ref()
            .unwrap() // TODO: address this later
            .clone();
        Ok(answer)
    }

    async fn rag_query(
        &self,
        tx: &mut PgConnection,
        query: &str,
    ) -> Result<String> {
        let request = EmbeddingsRequestBuilder::default()
            .model("nomic-embed-text")
            .input(vec![query.to_string()])
            .build()
            .map_err(|e| StandardError::new("ERR-AI-003").interpolate_err(e.to_string()))?;
        let response = self 
            .create_embeddings(&request)
            .await
            .map_err(|e| StandardError::new("ERR-AI-004").interpolate_err(e.to_string()))?;
        let query_embedding_vec: Vec<f32> = response.data[0]
            .embedding
            .clone()
            .iter()
            .map(|&x| x as f32)
            .collect();
        let query_embedding = Vector::from(query_embedding_vec);
        let similar_docs: Vec<Document> = sqlx::query_as(
            "SELECT id, content, embedding
             FROM documents 
             ORDER BY embedding <=> $1 
             LIMIT 3",
        )
        .bind(&query_embedding)
        .fetch_all(tx)
        .await?;
        let context = similar_docs
            .iter()
            .map(|doc| doc.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        let answer = self.direct_query(query, Some(&context)).await?;
        Ok(answer)
    }
}


