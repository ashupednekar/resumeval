use std::sync::Arc;

use ai::{
    clients::openai::Client, embeddings::{Embeddings, EmbeddingsRequestBuilder}
};
use pgvector::Vector;
use standard_error::{Interpolate, StandardError};
use crate::prelude::Result;


#[async_trait::async_trait]
pub trait IndexOps {
    async fn index_document(
        &self,
        content: &str,
    ) -> Result<Vector>;
}

#[async_trait::async_trait]
impl IndexOps for Arc<Client>{

    async fn index_document(
        &self,
        content: &str,
    ) -> Result<Vector> {
        let request = EmbeddingsRequestBuilder::default()
            .model("nomic-embed-text")
            .input(vec![content.to_string()])
            .build()
            .map_err(|e|StandardError::new("ERR-AI-004").interpolate_err(e.to_string()))
            ?;
        let response = self.create_embeddings(&request).await.map_err(|e|StandardError::new("ERR-AI-004").interpolate_err(e.to_string()))?;
        let embedding_vec: Vec<f32> = response.data[0]
            .embedding
            .iter()
            .map(|&x| x as f32)
            .collect();
        Ok(Vector::from(embedding_vec))
    }
}
