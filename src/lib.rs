pub mod config;
pub mod error;
pub mod handlers;
pub mod routes;
pub mod models;

use config::Config;
use std::sync::Arc;

/// Shared application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Config,
    // Add other shared state like database connections, HTTP clients, etc.
    // pub db: Arc<Database>,
    // pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            // Initialize other state here
        }
    }
}

/// Create the application with all dependencies
pub async fn create_app(config: Config) -> Result<axum::Router, error::AppError> {
    let state = Arc::new(AppState::new(config));
    let app = routes::create_router(state);
    Ok(app)
}