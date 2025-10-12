use arbitrage_detector::{
    log_info, log_error,
    socket::socket_container::binance_container::BinanceContainer,
    logger,
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    logger::init_logger(Some("info"));
    log_info!("Starting Binance WebSocket example");

    // Create a new BinanceContainer
    let mut container = BinanceContainer::new();
    
    // Add some cryptocurrency symbols to monitor
    let symbols = vec!["btcusdt", "ethusdt"];
    
    log_info!("Adding symbols to monitor: {:?}", symbols);
    
    // Start monitoring each symbol
    for symbol in symbols {
        match container.add_symbol(symbol) {
            Ok(_) => log_info!("✅ Started monitoring {}", symbol),
            Err(e) => log_error!("❌ Failed to start monitoring {}: {}", symbol, e),
        }
    }
    
    log_info!("WebSocket connections established! Monitoring for 30 seconds...");
    log_info!("You should see live cryptocurrency price data below:");
    
    // Let it run for 30 seconds to see live data
    sleep(Duration::from_secs(30)).await;
    
    log_info!("Example completed - container will be dropped and connections closed");
    
    // Container is automatically dropped here, triggering the Drop trait
    // which will shutdown all WebSocket connections gracefully
    
    log_info!("Binance WebSocket example finished");
    Ok(())
}