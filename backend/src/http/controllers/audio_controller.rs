use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::path::Path;

use crate::core::state::AppState;
use crate::http::middlewares::auth_middleware::AuthUser;
use crate::repositories::audio_repository::AudioRepository;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MB
const MIN_FILE_SIZE: usize = 1 * 1024; // 1 KB

pub async fn upload_audio(
    State(state): State<AppState>,
    auth_user: AuthUser,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    loop {
        let mut field = match multipart.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": format!("Multipart error: {}", e)})),
                ))
            }
        };

        if field.name() == Some("file") {
            let original_filename = field.file_name().unwrap_or("audio.mp3").to_string();
            let extension = Path::new(&original_filename)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("mp3");

            let new_filename = format!("{}.{}", Uuid::new_v4(), extension);
            let file_path = format!("uploads/{}", new_filename);

            let mut file = File::create(&file_path).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Failed to create file: {}", e)})),
                )
            })?;

            let mut total_size: usize = 0;
            let mut file_bytes = Vec::new();

            while let Some(chunk) = field.chunk().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": format!("Chunk read error: {}", e)})),
                )
            })? {
                total_size += chunk.len();

                if total_size > MAX_FILE_SIZE {
                    tokio::fs::remove_file(&file_path).await.ok();
                    return Err((
                        StatusCode::PAYLOAD_TOO_LARGE,
                        Json(serde_json::json!({"error": "File size exceeds 10MB limit"})),
                    ));
                }

                file_bytes.extend_from_slice(&chunk);
            }

            if total_size < MIN_FILE_SIZE {
                tokio::fs::remove_file(&file_path).await.ok();
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "File size is less than 1KB"})),
                ));
            }

            file.write_all(&file_bytes).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Failed to write file: {}", e)})),
                )
            })?;

            let file_id = Uuid::new_v4().to_string();

            match AudioRepository::create(
                &state.db,
                &file_id,
                &auth_user.id,
                &new_filename,
                &original_filename,
                &file_path,
                total_size as i64,
            ).await {
                Ok(file) => file,
                Err(e) => {
                    tokio::fs::remove_file(&file_path).await.ok();
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Database error", "details": e.to_string()})),
                    ));
                }
            };

            let job_id = match state.job_service.enqueue(&file_id, &auth_user.id).await {
                Ok(id) => id,
                Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))))
            };

            return Ok((
                StatusCode::ACCEPTED,
                Json(serde_json::json!({
                    "message": "Файл загружен и поставлен в очередь на анализ",
                    "job_id": job_id,
                    "audio_file_id": file_id,
                    "status": "processing"
                })),
            ))

        }
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "No 'file' field found in multipart data"})),
    ))
}