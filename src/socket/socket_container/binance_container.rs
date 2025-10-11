use std::{collections::HashMap, net::TcpStream};
use url::Url;
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket, Message};
use serde_json::Value;
use tungstenite::Error as WsError;
use crate::{log_debug, log_error, log_info, log_warn, socket::socket_container::socket_container::ISocketContainer};

pub struct BinanceContainer {
    symbols: Vec<String>, // ["btcusdt", ...]
    endpoints: HashMap<String, String>,
    sockets: HashMap<String, WebSocket<MaybeTlsStream<TcpStream>>>
}

impl BinanceContainer {
    pub fn new(symbols: Vec<String>) -> Self {
        log_info!("Creating new BinanceContainer with {} symbols", symbols.len());

        let mut endpoints: HashMap<String, String> = HashMap::new();
        for symbol in &symbols {
            let endpoint = format!("wss://stream.binance.com:9443/ws/{}@ticker", symbol);
            endpoints.insert(symbol.to_string(), endpoint);
        }

        BinanceContainer { 
            symbols,
            endpoints,
            sockets: HashMap::new()
        }
    }

    pub fn connect(&mut self) {
        if self.symbols.is_empty() {
            log_warn!("No symbols provided for Binance connection");
            return;
        }
        
        log_info!("Connecting to Binance with symbols: {:?}", self.symbols);
        
        // Your connection logic here
        for symbol in &self.symbols {
            let endpoint = self.endpoints.get(symbol).unwrap();
            match connect(Url::parse(&endpoint).unwrap()) {
                Ok((socket, _response)) => {
                    self.sockets.insert(symbol.clone(), socket);
                    let success_message = format!("[BinanceContainer] successfully connected to socket for symbol ({})", symbol);
                    log_info!("{}", success_message);
                }
                Err(err) => {
                    let error_log = format!("Cannot connect to Binance Websocket for symbol ({}), details: {}", symbol, err);
                    log_error!("{}", error_log); 
                    continue;
                }
            };
        }
        
        log_info!("Finished connected to Binance WebStocke");
    }
    
    pub fn disconnect(&self) {
        log_info!("Disconnecting from Binance WebSocket");
        // Disconnect logic here
    }

    /// Check if a socket connection exists for the given symbol
    pub fn is_connected(&self, symbol: &str) -> bool {
        self.sockets.contains_key(symbol)
    }

    pub fn get_data<T>(&mut self, symbol: String, callback: T) -> Result<(), String>
    where 
        T: Fn(&str, &str),
    {
        log_info!("Starting data stream for symbol: {}", symbol);
        let socket = self.sockets.get_mut(&symbol).ok_or_else(|| format!("No socket found for symbol: {}", symbol))?;

        loop {
            // Single read call - no duplicate reading
            match socket.read() {
                Ok(Message::Text(text)) => {
                    log_debug!("Received message for {}: {}", symbol, text);
        
                    match serde_json::from_str::<Value>(&text) {
                        Ok(json) => {
                            if let Some(price) = json["c"].as_str() {
                                log_debug!("{} price updated: {}", symbol.to_uppercase(), price);
                                callback(&symbol, price);
                            } else {
                                log_warn!("No price field 'c' found in message for {}", symbol);
                            }
                        }
                        Err(e) => {
                            log_warn!("Failed to parse JSON for {}: {}", symbol, e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    return self.handle_close(&symbol);
                }
                Ok(Message::Binary(_)) => {
                    log_debug!("Received binary message for {} (ignoring)", symbol);
                }
                Err(e) => {
                    return self.handle_error(&symbol, e);
                }
                _ => ()
            }
        }
    }

    pub fn start_price_monitoring(&mut self) -> Result<(), String> {
        for symbol in self.symbols.clone() {
            let symbol_clone = symbol.clone();
            
            // Start monitoring in a separate thread or task
            match self.get_data(symbol, |sym, price| {
                // Handle price update
                println!("Price update - {}: {}", sym.to_uppercase(), price);
                
                // You could also send to a channel, update shared state, etc.
            }) {
                Ok(_) => {
                    log_info!("Finished monitoring {}", symbol_clone);
                }
                Err(e) => {
                    log_error!("Error monitoring {}: {}", symbol_clone, e);
                }
            }
        }
        Ok(())
    }
}

// Helper methods to keep the main function clean
impl BinanceContainer {
    fn handle_close(&mut self, symbol: &str) -> Result<(), String> {
        log_info!("WebSocket closed for symbol: {}", symbol);
        self.sockets.remove(symbol);
        Err(format!("WebSocket closed for symbol: {}", symbol))
    }

    fn handle_error(&mut self, symbol: &str, error: WsError) -> Result<(), String> {
        let error_msg = match error {
            WsError::ConnectionClosed => {
                log_warn!("Connection closed for symbol: {}", symbol);
                format!("Connection closed for symbol: {}", symbol)
            }
            WsError::AlreadyClosed => {
                log_warn!("Socket already closed for symbol: {}", symbol);
                format!("Socket already closed for symbol: {}", symbol)
            }
            e => {
                log_error!("WebSocket error for {}: {}", symbol, e);
                format!("WebSocket error for {}: {}", symbol, e)
            }
        };
        
        self.sockets.remove(symbol);
        Err(error_msg)
    }
}
