use ai::{
    chat_completions::{ChatCompletion, ChatCompletionMessage, ChatCompletionRequestBuilder},
    embeddings::{Embeddings, EmbeddingsRequestBuilder},
};
use pgvector::Vector;
use sqlx::{FromRow, PgConnection, postgres::PgPoolOptions};

async fn index_document(
    tx: &mut PgConnection,
    client: &ai::clients::openai::Client,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = EmbeddingsRequestBuilder::default()
        .model("nomic-embed-text")
        .input(vec![content.to_string()])
        .build()?;

    let response = client.create_embeddings(&request).await?;

    let embedding_vec: Vec<f32> = response.data[0]
        .embedding
        .iter()
        .map(|&x| x as f32)
        .collect();
    let embedding = Vector::from(embedding_vec);

    sqlx::query("INSERT INTO documents (content, embedding) VALUES ($1, $2)")
        .bind(content)
        .bind(&embedding)
        .execute(tx)
        .await?;

    Ok(())
}
