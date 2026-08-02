use sqlx::SqlitePool;

use crate::models::user::User;

pub struct UserRepository;

impl UserRepository {
    pub async fn create(
        pool: &SqlitePool,
        id: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (id, email, password_hash) VALUES (?, ?, ?) RETURNING *",
        )
            .bind(id)
            .bind(email)
            .bind(password_hash)
            .fetch_one(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_by_email(pool: &SqlitePool, email: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }
}