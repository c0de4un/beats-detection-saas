CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    audio_file_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending', -- pending, processing, completed, failed
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (audio_file_id) REFERENCES audio_files(id) ON DELETE CASCADE
    );
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);