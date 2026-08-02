mod core;
mod http;
mod models;
mod repositories;

use std::net::SocketAddr;

use axum::{routing::get, routing::post, Router, Json};
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePoolOptions;

use crate::core::config::Config;
use crate::core::state::AppState;
use crate::http::responses::health_response::HealthResponse;
use crate::http::controllers::auth_controller;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = Config::from_env().expect("Failed to load configuration from .env");

    println!("Connecting to database: {}", config.db_url);

    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.db_url)
        .await
        .expect("Failed to connect to SQLite");

    println!("✅ Database connected.");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    println!("✅ Migrations applied successfully.");

    let state = AppState { db: db_pool };

    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/auth/register", post(auth_controller::register))
        .with_state(state);

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