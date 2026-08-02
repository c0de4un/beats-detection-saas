use sqlx::SqlitePool;
use sha2::{Sha256, Digest};

use crate::models::api_token::ApiToken;

pub struct ApiTokenRepository;

impl ApiTokenRepository {
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn create(
        pool: &SqlitePool,
        id: &str,
        user_id: &str,
        token_hash: &str,
        name: &str,
    ) -> Result<ApiToken, sqlx::Error> {
        let token = sqlx::query_as::<_, ApiToken>(
            "INSERT INTO api_tokens (id, user_id, token_hash, name) VALUES (?, ?, ?, ?) RETURNING *",
        )
            .bind(id)
            .bind(user_id)
            .bind(token_hash)
            .bind(name)
            .fetch_one(pool)
            .await?;

        Ok(token)
    }

    pub async fn find_by_hash(pool: &SqlitePool, token_hash: &str) -> Result<Option<ApiToken>, sqlx::Error> {
        let token = sqlx::query_as::<_, ApiToken>("SELECT * FROM api_tokens WHERE token_hash = ?")
            .bind(token_hash)
            .fetch_optional(pool)
            .await?;

        Ok(token)
    }

    pub async fn touch_last_used(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE api_tokens SET last_used_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}