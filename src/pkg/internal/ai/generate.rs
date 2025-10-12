use std::sync::Arc;

use ai::{
    chat_completions::{ChatCompletion, ChatCompletionMessage, ChatCompletionRequestBuilder}, clients::openai::Client, embeddings::{Embeddings, EmbeddingsRequestBuilder}
};
use standard_error::{Interpolate, StandardError};

use crate::{conf::settings, prelude::Result};

#[async_trait::async_trait]
pub trait GenerateOps {
    async fn direct_query(
        &self,
        query: &str,
        context: Option<&str>,
    ) -> Result<String>;

    // async fn rag_query(
    //     &self,
    //     tx: &mut PgConnection,
    //     query: &str,
    // ) -> Result<String>;

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

}


