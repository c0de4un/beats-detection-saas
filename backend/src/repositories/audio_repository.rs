use sqlx::SqlitePool;

use crate::models::audio_file::AudioFile;

pub struct AudioRepository;

impl AudioRepository {
    pub async fn create(
        pool: &SqlitePool,
        id: &str,
        user_id: &str,
        filename: &str,
        original_name: &str,
        file_path: &str,
        size: i64,
    ) -> Result<AudioFile, sqlx::Error> {
        let file = sqlx::query_as::<_, AudioFile>(
            "INSERT INTO audio_files (id, user_id, filename, original_name, file_path, size)
             VALUES (?, ?, ?, ?, ?, ?)
             RETURNING *",
        )
            .bind(id)
            .bind(user_id)
            .bind(filename)
            .bind(original_name)
            .bind(file_path)
            .bind(size)
            .fetch_one(pool)
            .await?;

        Ok(file)
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<AudioFile>, sqlx::Error> {
        let file = sqlx::query_as::<_, AudioFile>("SELECT * FROM audio_files WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(file)
    }

    pub async fn find_by_user_id(pool: &SqlitePool, user_id: &str) -> Result<Vec<AudioFile>, sqlx::Error> {
        let files = sqlx::query_as::<_, AudioFile>("SELECT * FROM audio_files WHERE user_id = ? ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(pool)
            .await?;

        Ok(files)
    }

    pub async fn update_status(pool: &SqlitePool, id: &str, status: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE audio_files SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn save_analysis(
        pool: &SqlitePool,
        id: &str,
        bpm: f32,
        beats_ms: &[u64],
    ) -> Result<(), sqlx::Error> {
        let beats_json = serde_json::to_string(beats_ms).unwrap_or_else(|_| "[]".to_string());

        sqlx::query(
            "UPDATE audio_files
             SET status = 'analyzed', bpm = ?, beats_ms = ?
             WHERE id = ?"
        )
            .bind(bpm)
            .bind(beats_json)
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn mark_failed(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE audio_files SET status = 'failed' WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}