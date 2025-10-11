use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json::{json, Value};
use std::sync::Arc;
use chrono::Utc;

use crate::AppState;

/// Health check endpoint
pub async fn health() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": Utc::now().to_rfc3339()
    })))
}

/// Hello world endpoint
pub async fn hello() -> &'static str {
    "Hello, world!"
}

/// Get application info
pub async fn app_info(State(_state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "name": "Arbitrage Detector API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "A web service for detecting arbitrage opportunities"
    }))
}