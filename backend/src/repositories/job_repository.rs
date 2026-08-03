use sqlx::SqlitePool;
use crate::models::job::Job;

pub struct JobRepository;

impl JobRepository {
    pub async fn create(pool: &SqlitePool, id: &str, audio_file_id: &str, user_id: &str) -> Result<Job, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            "INSERT INTO jobs (id, audio_file_id, user_id, status) VALUES (?, ?, ?, 'pending') RETURNING *"
        )
            .bind(id)
            .bind(audio_file_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
    }

    pub async fn find_pending_and_lock(pool: &SqlitePool) -> Result<Option<Job>, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            "UPDATE jobs SET status = 'processing', updated_at = CURRENT_TIMESTAMP
             WHERE id = (SELECT id FROM jobs WHERE status = 'pending' ORDER BY created_at ASC LIMIT 1)
             RETURNING *"
        )
            .fetch_optional(pool)
            .await
    }

    pub async fn update_status(pool: &SqlitePool, id: &str, status: &str, error: Option<&str>) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE jobs SET status = ?, error_message = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(status)
            .bind(error)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}