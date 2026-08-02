use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    Json,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand_core::OsRng;
use uuid::Uuid;
use validator::Validate;

use crate::core::state::AppState;
use crate::http::requests::register_request::RegisterRequest;
use crate::models::user::User;
use crate::repositories::user_repository::UserRepository;

pub async fn register(
    State(state): State<AppState>,
    payload_result: Result<Json<RegisterRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<User>), (StatusCode, Json<serde_json::Value>)> {
    // 2. Обработка ошибки парсинга (например, если поле email вообще забыли указать)
    let Json(payload) = match payload_result {
        Ok(j) => j,
        Err(rejection) => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({
                    "errors": {
                        "detail": rejection.body_text()
                    }
                })),
            ));
        }
    };

    if let Err(validation_errors) = payload.validate() {
        let mut error_map = serde_json::Map::new();

        // Превращаем ошибки validator в красивый JSON объект по полям
        for (field, errors) in validation_errors.field_errors() {
            let messages: Vec<String> = errors
                .iter()
                .map(|e| e.message.as_ref().map(|m| m.to_string()).unwrap_or_else(|| e.code.to_string()))
                .collect();
            error_map.insert(field.to_string(), serde_json::Value::String(messages.join(", ")));
        }

        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "errors": error_map })),
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(payload.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to hash password"})),
            ))
        }
    };

    let user_id = Uuid::new_v4().to_string();

    match UserRepository::create(&state.db, &user_id, &payload.email, &password_hash).await {
        Ok(user) => Ok((StatusCode::CREATED, Json(user))),
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.message().contains("UNIQUE constraint failed: users.email") {
                    return Err((
                        StatusCode::CONFLICT,
                        Json(serde_json::json!({
                            "errors": {
                                "email": "Email already exists"
                            }
                        })),
                    ));
                }
            }
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error", "details": e.to_string()})),
            ))
        }
    }
}