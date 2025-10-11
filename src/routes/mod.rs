use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::{handlers, AppState};

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health and info routes
        .route("/health", get(handlers::health))
        .route("/hello", get(handlers::hello))
        .route("/info", get(handlers::app_info))
        // API v1 routes
        .nest("/api/v1", api_v1_routes())
        // Add shared state
        .with_state(state)
}

fn api_v1_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Future arbitrage detection endpoints will go here
        // .route("/arbitrage", get(handlers::get_arbitrage_opportunities))
        // .route("/markets", get(handlers::get_markets))
}