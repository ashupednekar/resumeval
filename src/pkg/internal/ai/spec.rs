use pgvector::Vector;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct Document {
    #[allow(dead_code)]
    id: i64,
    pub content: String,
    #[allow(dead_code)]
    pub embedding: Vector,
}
