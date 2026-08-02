use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct AudioFile {
    pub id: String,
    pub user_id: String,
    pub filename: String,
    pub original_name: String,
    pub file_path: String,
    pub size: i64,
    pub status: String,
    pub created_at: String,
}