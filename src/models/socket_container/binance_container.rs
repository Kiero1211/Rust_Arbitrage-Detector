use crate::{log_info, log_warn, log_debug};

pub struct BinanceContainer {
    symbols: Vec<String> // ["btcusdt", ...]
}

impl BinanceContainer {
    pub fn new(symbols: Vec<String>) -> Self {
        log_info!("Creating new BinanceContainer with {} symbols", symbols.len());
        BinanceContainer { symbols }
    }

    pub fn connect(&self) {
        if self.symbols.is_empty() {
            log_warn!("No symbols provided for Binance connection");
            return;
        }
        
        log_info!("Connecting to Binance with symbols: {:?}", self.symbols);
        
        // Your connection logic here
        for symbol in &self.symbols {
            log_debug!("Processing symbol: {}", symbol);
        }
        
        log_info!("Successfully connected to Binance WebSocket");
    }
    
    pub fn disconnect(&self) {
        log_info!("Disconnecting from Binance WebSocket");
        // Disconnect logic here
    }
}