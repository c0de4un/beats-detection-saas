mod core;
mod http;
mod models;
mod repositories;
mod services;

use std::net::SocketAddr;
use std::sync::Arc;
use axum::{routing::get, routing::post, Router, Json};
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePoolOptions;
use tokio_util::sync::CancellationToken;
use tokio::signal;

use crate::core::config::Config;
use crate::core::state::AppState;
use crate::http::responses::health_response::HealthResponse;
use crate::http::controllers::{auth_controller, audio_controller, job_controller};
use crate::services::job_service::{JobService, run_worker}; // Импортируем сервис и воркер

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

    let cancel_token = CancellationToken::new();
    let (tx, rx) = tokio::sync::mpsc::channel::<()>(100);

    let job_service = Arc::new(JobService::new(db_pool.clone(), tx));

    let worker_token = cancel_token.clone();
    let worker_pool = db_pool.clone();
    tokio::spawn(async move {
        run_worker(worker_pool, rx, worker_token).await;
    });

    let state = AppState {
        db: db_pool,
        config: config.clone(),
        job_service: job_service.clone(),
    };

    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/auth/register", post(auth_controller::register))
        .route("/api/auth/login", post(auth_controller::login))
        .route("/api/audio/upload", post(audio_controller::upload_audio))
        .route("/api/jobs", get(job_controller::get_job_status))
        .with_state(state);

    let host = config.http_server_host.clone();
    let port = config.http_server_port;
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🚀 Server listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(cancel_token))
        .await
        .unwrap();
}

async fn health_handler() -> Json<HealthResponse> {
    let response = HealthResponse {
        status: "ok".to_string(),
        service: "korgi-beats".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    Json(response)
}

async fn shutdown_signal(token: CancellationToken) {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("🛑 Shutdown signal received, notifying worker...");
    token.cancel();

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
}