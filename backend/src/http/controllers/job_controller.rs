use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::core::state::AppState;
use crate::http::middlewares::auth_middleware::AuthUser;
use crate::repositories::{audio_repository::AudioRepository, job_repository::JobRepository};

#[derive(Deserialize)]
pub struct JobQueryParams {
    pub job_id: String,
}

#[derive(Serialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<AudioFileResponse>,
}

#[derive(Serialize)]
pub struct AudioFileResponse {
    pub id: String,
    pub filename: String,
    pub original_name: String,
    pub size: i64,
    pub status: String,
    pub bpm: Option<f32>,
    pub beats: Option<Vec<u64>>,
    pub created_at: String,
}

pub async fn get_job_status(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(params): Query<JobQueryParams>,
) -> Result<Json<JobStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    let job_id = params.job_id;

    let job = match JobRepository::find_by_id(&state.db, &job_id).await {
        Ok(Some(j)) => j,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Job not found"})),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error", "details": e.to_string()})),
            ));
        }
    };

    if job.user_id != auth_user.id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    let mut result = None;
    if job.status == "completed" {
        if let Ok(Some(audio_file)) = AudioRepository::find_by_id(&state.db, &job.audio_file_id).await {
            let beats = audio_file.get_beats();

            result = Some(AudioFileResponse {
                id: audio_file.id,
                filename: audio_file.filename,
                original_name: audio_file.original_name,
                size: audio_file.size,
                status: audio_file.status,
                bpm: audio_file.bpm,
                beats, // Вставляем уже готовый Vec<u64>
                created_at: audio_file.created_at,
            });
        }
    }

    Ok(Json(JobStatusResponse {
        job_id: job.id,
        status: job.status,
        error_message: job.error_message,
        result,
    }))
}