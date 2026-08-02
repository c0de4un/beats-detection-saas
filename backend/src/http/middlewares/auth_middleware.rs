use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};
use serde_json::json;

use crate::core::auth::verify_jwt;
use crate::core::state::AppState;

pub struct AuthUser {
    pub id: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok());

        let token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => &header[7..],
            _ => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Missing or invalid Authorization header"})),
                ));
            }
        };

        let claims = match verify_jwt(token, &state.config) {
            Ok(c) => c,
            Err(_) => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Invalid or expired token"})),
                ));
            }
        };

        Ok(AuthUser { id: claims.sub })
    }
}