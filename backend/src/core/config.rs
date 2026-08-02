use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub http_server_host: String,
    pub http_server_port: u16,
    pub db_url: String,
    pub jwt_secret: String,
    pub jwt_expires_hours: i64,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Config>()
    }
}