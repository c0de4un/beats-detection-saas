mod core;
mod http;

use std::net::SocketAddr;

use axum::{routing::get, Router, Json};
use dotenvy::dotenv;

use crate::core::config::Config;
use crate::http::responses::health_response::HealthResponse;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = Config::from_env().expect("Failed to load configuration from .env");

    println!("Starting Korgi.Beats OSS with config: {:?}", config);

    let app = Router::new()
        .route("/api/health", get(health_handler));

    let host = config.http_server_host.clone();
    let port = config.http_server_port;
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("🚀 Server listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn health_handler() -> Json<HealthResponse> {
    let response = HealthResponse {
        status: "ok".to_string(),
        service: "korgi-beats".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Json(response)
}