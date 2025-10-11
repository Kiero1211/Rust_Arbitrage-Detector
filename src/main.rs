use dotenvy::dotenv;
use tracing::info;
use arbitrage_detector::{config::Config, create_app, error::AppError};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize environment variables
    dotenv().ok();
    
    // Initialize tracing/logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::from_env()
        .map_err(|e| AppError::ConfigError(format!("Failed to load config: {}", e)))?;
    
    info!("Starting server with config: {:?}", config);

    // Create the application
    let app = create_app(config.clone()).await?;

    // Start the server
    let server_address = config.server_address();
    info!("Server starting on {}", server_address);
    
    let listener = tokio::net::TcpListener::bind(&server_address)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to bind to {}: {}", server_address, e)))?;
    
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Server error: {}", e)))?;

    Ok(())
}