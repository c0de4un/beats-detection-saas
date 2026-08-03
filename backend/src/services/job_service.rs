use sqlx::SqlitePool;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::core::analyzer::AudioAnalyzer;
use crate::repositories::{audio_repository::AudioRepository, job_repository::JobRepository};

pub struct JobService {
    pool: SqlitePool,
    tx: mpsc::Sender<()>, // Канал для пробуждения воркера
}

impl JobService {
    pub fn new(pool: SqlitePool, tx: mpsc::Sender<()>) -> Self {
        Self { pool, tx }
    }

    pub async fn enqueue(&self, audio_file_id: &str, user_id: &str) -> Result<String, String> {
        let job_id = Uuid::new_v4().to_string();
        JobRepository::create(&self.pool, &job_id, audio_file_id, user_id)
            .await
            .map_err(|e| e.to_string())?;

        let _ = self.tx.send(()).await; // Будим воркер
        Ok(job_id)
    }
}

pub async fn run_worker(
    pool: SqlitePool,
    mut rx: mpsc::Receiver<()>,
    cancel_token: CancellationToken,
) {
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                println!("🛑 Worker received shutdown signal. Exiting gracefully...");
                break;
            }

            _ = rx.recv() => {}

            _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {}
        }

        let job = match JobRepository::find_pending_and_lock(&pool).await {
            Ok(Some(j)) => j,
            Ok(None) => continue,
            Err(e) => {
                eprintln!("❌ DB error in worker: {}", e);
                continue;
            }
        };

        println!("⚙️ Processing job {} for file {}", job.id, job.audio_file_id);

        let audio_file = match AudioRepository::find_by_id(&pool, &job.audio_file_id).await {
            Ok(Some(f)) => f,
            _ => {
                let _ = JobRepository::update_status(&pool, &job.id, "failed", Some("File not found")).await;
                continue;
            }
        };

        let path = audio_file.file_path.clone();
        let job_id = job.id.clone();
        let file_id = job.audio_file_id.clone();
        let pool_clone = pool.clone();

        let result = tokio::task::spawn_blocking(move || {
            AudioAnalyzer::analyze(&path)
        }).await;

        match result {
            Ok(Ok(beat_result)) => {
                match AudioRepository::save_analysis(
                    &pool_clone,
                    &file_id,
                    beat_result.bpm,
                    &beat_result.beats_ms
                ).await {
                    Ok(_) => {
                        let _ = JobRepository::update_status(&pool_clone, &job_id, "completed", None).await;
                        println!("✅ Job {} completed. BPM: {}, Beats found: {}",
                                 job.id, beat_result.bpm, beat_result.beats_ms.len());
                    }
                    Err(db_err) => {
                        let err_msg = format!("DB save error: {}", db_err);
                        let _ = JobRepository::update_status(&pool_clone, &job_id, "failed", Some(&err_msg)).await;
                        eprintln!("❌ Failed to save analysis to DB: {}", db_err);
                    }
                }
            }
            Ok(Err(e)) => {
                let _ = AudioRepository::mark_failed(&pool_clone, &file_id).await;
                let _ = JobRepository::update_status(&pool_clone, &job_id, "failed", Some(&e)).await;
                eprintln!("❌ Analysis failed for job {}: {}", job.id, e);
            }
            Err(e) => {
                let _ = AudioRepository::mark_failed(&pool_clone, &file_id).await;
                let _ = JobRepository::update_status(&pool_clone, &job_id, "failed", Some("Thread panic")).await;
                eprintln!("💥 Task panicked: {}", e);
            }
        }
    }
}