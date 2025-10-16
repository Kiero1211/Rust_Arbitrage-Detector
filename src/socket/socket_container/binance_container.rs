use std::{collections::HashMap, net::TcpStream, sync::{mpsc::Receiver, Arc}, thread::{self, JoinHandle}};
use url::Url;
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket, Message};
use serde_json::Value;
use crate::{log_debug, log_error, log_info, log_warn, models::SymbolMessage};
use std::sync::mpsc::{Sender};
use std::sync::{atomic::AtomicBool, atomic::Ordering};

pub struct BinanceContainer {
    sockets: HashMap<String, WebSocket<MaybeTlsStream<TcpStream>>>,
    sender: Arc<Sender<SymbolMessage>>,
    sender: Receiver<SymbolMessage>,
    socket_threads: HashMap<String, JoinHandle<Result<(), String>>>,
    symbols: Vec<String>,
    shutdown: Arc<AtomicBool>,
    max_reconnect_attempts: u32,
}

impl BinanceContainer {
    pub fn new() -> Self {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let sender = Arc::new(sender);

        BinanceContainer { 
            sockets: HashMap::new(),
            sender,
            socket_threads: HashMap::new(),
            symbols: Vec::new(),
            shutdown: Arc::new(AtomicBool::new(false)),
            max_reconnect_attempts: 5,
        }
    }

    pub fn add_symbol(&mut self, symbol: &str) -> Result<(), String> {
        // Add symbol to tracking list if not already present
        if !self.symbols.contains(&symbol.to_string()) {
            self.symbols.push(symbol.to_string());
        }
        
        self.init_socket_connection(symbol);
        return self.get_data(symbol);
    }

    fn init_socket_connection(&mut self, symbol: &str) {
        // Your connection logic here
        if self.has_socket_connection(symbol) {
            return;
        }

        let endpoint = format!("wss://stream.binance.com:9443/ws/{}@ticker", symbol);
        match connect(Url::parse(&endpoint).unwrap()) {
            Ok((socket, _response)) => {
                self.sockets.insert(symbol.to_owned(), socket);
                let success_message = format!("[BinanceContainer] successfully connected to socket for symbol ({})", symbol);
                log_info!("{}", success_message);
            }
            Err(err) => {
                let error_log = format!("Cannot connect to Binance Websocket for symbol ({}), details: {}", symbol, err);
                log_error!("{}", error_log); 
            }
        };
        
        log_info!("Finished connected to Binance WebStocke");
    }
    
    pub fn disconnect(&self) {
        log_info!("Disconnecting from Binance WebSocket");
        // Disconnect logic here
    }

    /// Check if a socket connection exists for the given symbol
    pub fn has_socket_connection(&self, symbol: &str) -> bool {
        self.sockets.contains_key(symbol)
    }

    /// Start monitoring multiple cryptocurrency symbols
    pub async fn start_monitoring(&mut self, symbols: Vec<String>) -> Result<(), String> {
        for symbol in symbols {
            self.init_socket_connection(&symbol);
            self.get_data(&symbol)?;
        }
        Ok(())
    }

    fn get_data(&mut self, symbol: &str) -> Result<(), String>
    {
        log_info!("Starting data stream for symbol: {}", symbol);
        let socket = self.sockets.remove(symbol).ok_or_else(|| format!("No socket found for symbol: {}", symbol))?;
        let symbol_owned = symbol.to_owned();
        let sender = Arc::clone(&self.sender);
        let shutdown = Arc::clone(&self.shutdown);
        let max_attempts = self.max_reconnect_attempts;
        
        let handle = thread::spawn(move || {
            let mut socket = socket;
            let mut reconnect_attempts = 0;
            
            loop {
                // Check if shutdown is requested
                if shutdown.load(Ordering::Relaxed) {
                    log_info!("Shutdown requested for {}, stopping data stream", symbol_owned);
                    break;
                }
                
                match socket.read() {
                    Ok(Message::Text(text)) => {
                        // Reset reconnect attempts on successful read
                        reconnect_attempts = 0;
                        match serde_json::from_str::<Value>(&text) {
                            Ok(json) => {
                                if let Some(price) = json["c"].as_str() {
                                    if let Ok(price) = price.parse::<f64>() {
                                        let message = SymbolMessage::new(symbol_owned.clone(), price);
                                        
                                        // Send to channel
                                        if let Err(e) = sender.send(message) {
                                            log_error!("Failed to send message to channel: {}", e);
                                            return Err(format!("Channel send error: {}", e));
                                        }
                                    } else {
                                        let error = format!("Failed to parse price: {}", price);
                                        log_error!("{}", error);
                                    }
                                } else {
                                    log_warn!("No price field 'c' found in message for {}", symbol_owned);
                                }
                            }
                            Err(e) => {
                                log_warn!("Failed to parse JSON for {}: {}", symbol_owned, e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        log_info!("WebSocket connection closed for {}", symbol_owned);
                        
                        // Attempt to reconnect automatically
                        if reconnect_attempts < max_attempts {
                            reconnect_attempts += 1;
                            log_info!("Attempting reconnection {} of {} for {}", reconnect_attempts, max_attempts, symbol_owned);
                            
                            // Wait before reconnecting (exponential backoff)
                            let delay = std::time::Duration::from_millis(1000 * reconnect_attempts as u64);
                            std::thread::sleep(delay);
                            
                            // Try to create a new connection
                            let endpoint = format!("wss://stream.binance.com:9443/ws/{}@ticker", symbol_owned);
                            match connect(Url::parse(&endpoint).unwrap()) {
                                Ok((new_socket, _response)) => {
                                    socket = new_socket;
                                    log_info!("✅ Successfully reconnected to {} (attempt {})", symbol_owned, reconnect_attempts);
                                    continue;
                                }
                                Err(reconnect_error) => {
                                    log_error!("Reconnection attempt {} failed for {}: {}", reconnect_attempts, symbol_owned, reconnect_error);
                                    if reconnect_attempts >= max_attempts {
                                        log_error!("Max reconnection attempts reached for {}, giving up", symbol_owned);
                                        return Err(format!("Max reconnection attempts reached"));
                                    }
                                }
                            }
                        } else {
                            log_error!("Max reconnection attempts reached for {}, giving up", symbol_owned);
                            return Err(format!("Max reconnection attempts reached"));
                        }
                    }
                    Ok(Message::Binary(_)) => {
                        log_debug!("Received binary message for {} (ignoring)", symbol_owned);
                    }
                    Err(e) => {
                        log_error!("WebSocket error for {}: {}", symbol_owned, e);
                        
                        // Attempt to reconnect automatically on error
                        if reconnect_attempts < max_attempts {
                            reconnect_attempts += 1;
                            log_info!("Attempting reconnection {} of {} for {} due to error", reconnect_attempts, max_attempts, symbol_owned);
                            
                            // Wait before reconnecting (exponential backoff)
                            let delay = std::time::Duration::from_millis(1000 * reconnect_attempts as u64);
                            std::thread::sleep(delay);
                            
                            // Try to create a new connection
                            let endpoint = format!("wss://stream.binance.com:9443/ws/{}@ticker", symbol_owned);
                            match connect(Url::parse(&endpoint).unwrap()) {
                                Ok((new_socket, _response)) => {
                                    socket = new_socket;
                                    log_info!("✅ Successfully reconnected to {} (attempt {})", symbol_owned, reconnect_attempts);
                                    continue;
                                }
                                Err(reconnect_error) => {
                                    log_error!("Reconnection attempt {} failed for {}: {}", reconnect_attempts, symbol_owned, reconnect_error);
                                    if reconnect_attempts >= max_attempts {
                                        log_error!("Max reconnection attempts reached for {}, giving up", symbol_owned);
                                        return Err(format!("Max reconnection attempts reached"));
                                    }
                                }
                            }
                        } else {
                            log_error!("Max reconnection attempts reached for {}, giving up", symbol_owned);
                            return Err(format!("Max reconnection attempts reached"));
                        }
                    }
                    _ => {
                        log_warn!("Unknown message type for {}", symbol_owned);
                    }
                }
            }
            
            Ok(())
        });

        self.socket_threads.insert(symbol.to_owned(), handle);
        Ok(())
    }
}

// Helper methods to keep the main function clean
impl BinanceContainer {

    /// Gracefully shutdown all connections and threads
    pub fn shutdown(&mut self) {
        log_info!("Initiating graceful shutdown of BinanceContainer");
        
        // Set shutdown flag
        self.shutdown.store(true, Ordering::Relaxed);
        
        // Close all WebSocket connections
        log_info!("Closing {} WebSocket connections", self.sockets.len());
        for (symbol, mut socket) in self.sockets.drain() {
            log_debug!("Closing socket for symbol: {}", symbol);
            if let Err(e) = socket.close(None) {
                log_warn!("Error closing socket for {}: {}", symbol, e);
            }
        }
        
        // Wait for all socket threads to complete
        log_info!("Waiting for {} socket threads to complete", self.socket_threads.len());
        for (symbol, handle) in self.socket_threads.drain() {
            log_debug!("Waiting for thread to complete: {}", symbol);
            match handle.join() {
                Ok(result) => {
                    match result {
                        Ok(_) => log_debug!("Thread for {} completed successfully", symbol),
                        Err(e) => log_warn!("Thread for {} ended with error: {}", symbol, e),
                    }
                }
                Err(e) => {
                    log_error!("Error joining thread for {}: {:?}", symbol, e);
                }
            }
        }
        
        log_info!("BinanceContainer shutdown completed");
    }


}

/// Implement Drop trait for graceful cleanup
impl Drop for BinanceContainer {
    fn drop(&mut self) {
        log_info!("BinanceContainer is being dropped, initiating cleanup");
        self.shutdown();
    }
}


