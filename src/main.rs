use std::thread;

use dotenvy::dotenv;
use arbitrage_detector::{config::Config, create_app, error::AppError, log_info, logger, socket::{socket_consumer::{self, SocketConsumer}, socket_container::coinbase_container::CoinBaseContainer}};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize environment variables
    dotenv().ok();
    
    // Load configuration first to get log level
    let config = Config::from_env()
        .map_err(|e| AppError::ConfigError(format!("Failed to load config: {}", e)))?;
    
    // Initialize singleton logger
    logger::init_logger(config.log_level.as_deref());
    
    log_info!("Starting server with config: {:?}", config);

    // Create the application
    let app = create_app(config.clone()).await?;

    // Start the server
    let server_address = config.server_address();
    log_info!("Server starting on {}", server_address);
    
    // let listener = tokio::net::TcpListener::bind(&server_address)
    //     .await
    //     .map_err(|e| AppError::InternalServerError(format!("Failed to bind to {}: {}", server_address, e)))?;
    
    // axum::serve(listener, app)
    //     .await
    //     .map_err(|e| AppError::InternalServerError(format!("Server error: {}", e)))?;

    // let mut socket_consumer = SocketConsumer::new();
    // socket_consumer.add_symbol("btcusdt".to_string());
    // let handle = thread::spawn(move|| {
    //     socket_consumer.start_binance_price_monitoring();
    // });
    // handle.join();

    let mut coinbase_container = CoinBaseContainer::new();
    coinbase_container.add_symbol("ETH-USDT");
    coinbase_container.add_symbol("BTC-USDT");
    let handle = thread::spawn(move || {
        coinbase_container.start_monitoring();

        let handle = thread::spawn(move || {
            coinbase_container.on_symbol_update(|symbol, price| {
                println!("Got {} - ${}", symbol, price);
            });
        });

        handle.join();
    });

    handle.join();
    Ok(())
}