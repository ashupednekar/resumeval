use serde::Deserialize;
use sqlx::FromRow;


#[derive(Deserialize, FromRow)]
pub struct JobEntry {
    pub title: String,
    pub department: String,
    pub description: String,
    pub requirements: String,
    pub url: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}


