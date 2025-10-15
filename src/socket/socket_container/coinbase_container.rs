use std::{collections::HashMap, fmt::Binary, net::TcpStream, sync::Arc, thread::{self, JoinHandle}};
use url::Url;
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket, Message};
use serde_json::{json, Value};
use crate::{log_debug, log_error, log_info, log_warn, models::SymbolMessage, socket};
use std::sync::mpsc::{Sender};
use std::sync::{atomic::AtomicBool, atomic::Ordering};

pub struct CoinBaseContainer {
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    sender: Arc<Sender<SymbolMessage>>,
    socket_thread: Option<JoinHandle<Result<(), String>>>,
    symbols: Vec<String>,
    shutdown: Arc<AtomicBool>,
    max_reconnect_attempts: u32,
    reconnect_attempts: u32,
}

impl CoinBaseContainer {
    pub fn new() -> Self {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let sender = Arc::new(sender);
        Self::new_with_sender(sender)
    }

    pub fn new_with_sender(sender: Arc<Sender<SymbolMessage>>) -> Self {
        CoinBaseContainer { 
            socket: None,
            sender,
            socket_thread: None,
            symbols: Vec::new(),
            shutdown: Arc::new(AtomicBool::new(false)),
            max_reconnect_attempts: 5,
            reconnect_attempts: 0
        }
    }

    pub fn with_reconnect_limit(sender: Arc<Sender<SymbolMessage>>, max_attempts: u32) -> Self {
        CoinBaseContainer { 
            socket: None,
            sender,
            socket_thread: None,
            symbols: Vec::new(),
            shutdown: Arc::new(AtomicBool::new(false)),
            max_reconnect_attempts: max_attempts,
            reconnect_attempts: 0
        }
    }

    pub fn add_symbol(&mut self, symbol: &str) -> Result<(), String> {
        // Add symbol to tracking list if not already present
        if !self.symbols.contains(&symbol.to_string()) {
            self.symbols.push(symbol.to_string());
        }
        
        self.init_socket_connection();
        return self.get_data();
    }

    fn init_socket_connection(&mut self) -> Result<(), String> {
        let url = Url::parse("wss://ws-feed.exchange.coinbase.com").unwrap();
        println!("Connecting to {}", url);

        let subscribe_message = json!({
            "type": "subscribe",
            "channels": [{
                "name": "ticker",
                "product_ids": self.symbols
            }]
        });

        match connect(url) {
            Ok((mut socket, _response)) => {
                socket
                    .send(Message::Text(subscribe_message.to_string()))
                    .unwrap();
                self.socket = Some(socket);
                let success_message = format!("[CoinBaseContainer] successfully connected to socket for {} symbols", self.symbols.len());
                log_info!("{}", success_message);
            }
            Err(err) => {
                let error_log = format!("Cannot connect to CoinBase Websocket for {} symbols, details: {}", self.symbols.len(), err);
                log_error!("{}", error_log); 
                return Err(error_log);
            }
        };
        
        log_info!("Finished connected to Binance WebStocke");
        Ok(())
    }
    
    pub fn disconnect(&self) {
        log_info!("Disconnecting from CoinBase WebSocket");
        // Disconnect logic here
    }

    /// Check if a socket connection exists for the given symbol
    pub fn has_socket_connection(&self) -> bool {
        if let Some(socket) = &self.socket {
            return true;
        } else {
            return false;
        }
    }

    /// Start monitoring multiple cryptocurrency symbols
    pub async fn start_monitoring(&mut self, symbols: Vec<String>) -> Result<(), String> {
        for symbol in symbols {
            self.init_socket_connection(&symbol);
            self.get_data(&symbol)?;
        }
        Ok(())
    }

    fn get_data(&mut self) -> Result<(), String>
    {
        if let None = self.socket {
            log_error!("[CoinBaseContainer - get_data] There is no socket connection to get data");
            return Err("[CoinBaseContainer - get_data] There is no socket connection to get data".to_string());
        }
        let socket: WebSocket<MaybeTlsStream<TcpStream>> = self.socket.unwrap();

        let sender = Arc::clone(&self.sender);
        let shutdown = Arc::clone(&self.shutdown);
        let max_attempts = self.max_reconnect_attempts;
        
        let handle = thread::spawn(move || {
            let mut socket = socket;
            loop {
                // Check if shutdown is requested
                if shutdown.load(Ordering::Relaxed) {
                    log_info!("[CoinBaseContainer - get_data] Shutdown requested, stopping data stream");
                    break;
                }
                
                match socket.read() {
                    Ok(Message::Text(text)) => {
                        self.on_message(text, &sender);
                    }
                    Ok(Message::Close(_)) => {
                        on_close();
                    }
                    Err(e) => {
                        log_error!("WebSocket error for {}: {}", symbol_owned, e);
                        
                        // Attempt to reconnect automatically on error
                        if self.reconnect_attempts < max_attempts {
                            self.reconnect_attempts += 1;
                            log_info!("Attempting reconnection {} of {} for {} due to error", self.reconnect_attempts, max_attempts, symbol_owned);
                            
                            // Wait before reconnecting (exponential backoff)
                            let delay = std::time::Duration::from_millis(1000 * self.reconnect_attempts as u64);
                            std::thread::sleep(delay);
                            
                            // Try to create a new connection
                            let endpoint = format!("wss://stream.binance.com:9443/ws/{}@ticker", symbol_owned);
                            match connect(Url::parse(&endpoint).unwrap()) {
                                Ok((new_socket, _response)) => {
                                    socket = new_socket;
                                    log_info!("✅ Successfully reconnected to {} (attempt {})", symbol_owned, self.reconnect_attempts);
                                    continue;
                                }
                                Err(reconnect_error) => {
                                    log_error!("Reconnection attempt {} failed for {}: {}", self.reconnect_attempts, symbol_owned, reconnect_error);
                                    if self.reconnect_attempts >= max_attempts {
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

        self.socket_thread = Some(handle);
        Ok(())
    }

    fn on_message(&mut self, text: String, sender: &Arc<Sender<SymbolMessage>>) -> Result<(), String> {
        self.reconnect_attempts = 0;
        match serde_json::from_str::<Value>(&text) {
            Ok(json) => {
                if json["type"] == "ticker" {
                    let product_id = json["product_id"].as_str().unwrap_or("unknown");
                    let formatted_symbol = product_id.trim().replace("-", "");

                    if let Some(price) = json["price"].as_str() {
                        if let Ok(price) = price.parse::<f64>() {
                            let message = SymbolMessage::new(formatted_symbol, price);
                            
                            // Send to channel
                            if let Err(e) = sender.send(message) {
                                log_error!("[CoinBaseContainer - get_data] Failed to send message to channel: {}", e);
                                return Err(format!(" [CoinBaseContainer - get_data] Channel send error: {}", e));
                            }
                        } else {
                            let error = format!("Failed to parse price: {}", price);
                            log_error!("{}", error);
                        }
                    } else {
                        log_warn!("[CoinBaseContainer - get_data] No price field 'c' found in message for {}", formatted_symbol);
                    }
                }
            }
            Err(e) => {
                log_warn!("[CoinBaseContainer - get_data] Failed to parse JSON for {}: {}", &text, e);
            }
        }
        Ok(())
    }

    fn on_close(&mut self, shutdown: Arc<AtomicBool>) -> Result<(), String> {
        log_info!("[CoinBaseContainer - get_data] WebSocket connection closed");
        // Check if shutdown is requested
        if shutdown.load(Ordering::Relaxed) {
            log_info!("[CoinBaseContainer - get_data] Shutdown requested, no reconnection");
            return Err("[CoinBaseContainer - get_data] Shutdown requested, no reconnection".to_string());
        }   

        // Attempt to reconnect automatically
        if self.reconnect_attempts < self.max_reconnect_attempts {
            self.reconnect_attempts += 1;
            log_info!("[CoinBaseContainer - get_data] Attempting reconnection {} of {}", self.reconnect_attempts, self.max_reconnect_attempts);
            
            // Wait before reconnecting (exponential backoff)
            let delay = std::time::Duration::from_millis(1000 * self.reconnect_attempts as u64);
            std::thread::sleep(delay);
            
            // Try to create a new connection
            self.init_socket_connection();
            self.get_data();
        } else {
            log_error!("Max reconnection attempts reached, giving up");
            return Err(format!("Max reconnection attempts reached"));
        }

        Ok(())
    }

    fn on_error(&mut self, error: tungstenite::Error) {

    }
}

// Helper methods to keep the main function clean
impl CoinBaseContainer {

    /// Gracefully shutdown all connections and threads
    pub fn shutdown(&mut self) {
        log_info!("Initiating graceful shutdown of CoinBaseContainer");
        
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
        
        log_info!("CoinBaseContainer shutdown completed");
    }


}

/// Implement Drop trait for graceful cleanup
impl Drop for CoinBaseContainer {
    fn drop(&mut self) {
        log_info!("CoinBaseContainer is being dropped, initiating cleanup");
        self.shutdown();
    }
}


