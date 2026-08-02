use serde::Serialize;
use crate::models::user::User;

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}