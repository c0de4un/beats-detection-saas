use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct ApiToken {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub name: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
}