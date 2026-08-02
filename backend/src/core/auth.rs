use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, errors::Error};
use serde::{Deserialize, Serialize};

use crate::core::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // subject (user_id)
    pub exp: usize,  // expiration time (UNIX timestamp)
    pub iat: usize,  // issued at (UNIX timestamp)
}

pub fn create_jwt(user_id: &str, config: &Config) -> Result<String, Error> {
    let now = Utc::now();
    let exp = now + Duration::hours(config.jwt_expires_hours);

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )
}

pub fn verify_jwt(token: &str, config: &Config) -> Result<Claims, Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}