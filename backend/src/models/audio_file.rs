use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AudioFile {
    pub id: String,
    pub user_id: String,
    pub filename: String,
    pub original_name: String,
    pub file_path: String,
    pub size: i64,
    pub status: String,
    pub bpm: Option<f32>,

    #[serde(skip_serializing)]
    pub beats_ms: Option<String>,

    pub created_at: String,
}

impl AudioFile {
    pub fn get_beats(&self) -> Option<Vec<u64>> {
        self.beats_ms.as_ref().and_then(|s| serde_json::from_str(s).ok())
    }
}