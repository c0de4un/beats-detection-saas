use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    Json,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;
use uuid::Uuid;
use validator::Validate;

use crate::core::auth::create_jwt;
use crate::core::config::Config;
use crate::core::state::AppState;
use crate::http::requests::register_request::RegisterRequest;
use crate::http::requests::login_request::LoginRequest;
use crate::http::responses::auth_response::AuthResponse;
use crate::models::user::User;
use crate::repositories::user_repository::UserRepository;

type ApiResult<T> = Result<T, (StatusCode, Json<serde_json::Value>)>;

fn json_error(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": msg })))
}

fn handle_validation_errors(errors: validator::ValidationErrors) -> (StatusCode, Json<serde_json::Value>) {
    let mut error_map = serde_json::Map::new();
    for (field, field_errors) in errors.field_errors() {
        let messages: Vec<String> = field_errors
            .iter()
            .map(|e| e.message.as_ref().map(|m| m.to_string()).unwrap_or_else(|| e.code.to_string()))
            .collect();
        error_map.insert(field.to_string(), serde_json::Value::String(messages.join(", ")));
    }
    (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "errors": error_map })))
}

fn parse_json<T: serde::de::DeserializeOwned>(payload: Result<Json<T>, JsonRejection>) -> ApiResult<T> {
    payload.map(|Json(p)| p).map_err(|rejection| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "errors": { "detail": rejection.body_text() } })),
        )
    })
}

fn generate_token(user: &User, config: &Config) -> ApiResult<String> {
    create_jwt(&user.id, config).map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token"))
}

pub async fn register(
    State(state): State<AppState>,
    payload_result: Result<Json<RegisterRequest>, JsonRejection>,
) -> ApiResult<(StatusCode, Json<AuthResponse>)> {
    let payload = parse_json(payload_result)?;
    payload.validate().map_err(handle_validation_errors)?;

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password"))?
        .to_string();

    let user_id = Uuid::new_v4().to_string();
    let user = match UserRepository::create(&state.db, &user_id, &payload.email, &password_hash).await {
        Ok(user) => user,
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.message().contains("UNIQUE constraint failed: users.email") {
                    return Err((StatusCode::CONFLICT, Json(serde_json::json!({"errors": { "email": "Email already exists" }}))));
                }
            }
            return Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "Database error"));
        }
    };

    let token = generate_token(&user, &state.config)?;
    Ok((StatusCode::CREATED, Json(AuthResponse { token, user })))
}

pub async fn login(
    State(state): State<AppState>,
    payload_result: Result<Json<LoginRequest>, JsonRejection>,
) -> ApiResult<(StatusCode, Json<AuthResponse>)> {
    let payload = parse_json(payload_result)?;
    payload.validate().map_err(handle_validation_errors)?;

    let user = UserRepository::find_by_email(&state.db, &payload.email).await
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"errors": { "detail": "Invalid credentials" }}))))?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Password hash error"))?;

    if Argon2::default().verify_password(payload.password.as_bytes(), &parsed_hash).is_err() {
        return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"errors": { "detail": "Invalid credentials" }}))));
    }

    let token = generate_token(&user, &state.config)?;
    Ok((StatusCode::OK, Json(AuthResponse { token, user })))
}