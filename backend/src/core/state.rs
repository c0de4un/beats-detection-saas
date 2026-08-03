use std::sync::Arc;
use sqlx::SqlitePool;
use crate::core::config::Config;
use crate::services::job_service::JobService;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Config,
    pub job_service: Arc<JobService>,
}