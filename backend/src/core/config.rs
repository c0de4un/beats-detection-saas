use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub http_server_host: String,
    pub http_server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Config>()
    }
}