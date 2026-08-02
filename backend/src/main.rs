use axum::{routing::get, Router, Json};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
}

#[tokio::main]
async fn main() {
    println!("Starting Korgi.Beats OSS...");

    let app = Router::new()
        .route("/api/health", get(health_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

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