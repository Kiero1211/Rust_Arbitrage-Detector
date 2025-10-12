use std::{collections::HashMap, net::TcpStream, sync::Arc, thread::{self, JoinHandle}};
use url::Url;
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket, Message};
use serde_json::Value;
use tungstenite::Error as WsError;
use crate::{log_debug, log_error, log_info, log_warn, models::SymbolMessage, socket};
use std::sync::mpsc::{Sender};
pub struct BinanceContainer {
    sockets: HashMap<String, WebSocket<MaybeTlsStream<TcpStream>>>,
    sender: Arc<Sender<SymbolMessage>>,
    socket_threads: HashMap<String, JoinHandle<Result<(), String>>>
}

impl BinanceContainer {
    pub fn new(sender: Arc<Sender<SymbolMessage>>) -> Self {
        BinanceContainer { 
            sockets: HashMap::new(),
            sender,
            socket_threads: HashMap::new()
        }
    }

    pub fn add_symbol(&mut self, symbol: &str) -> Result<(), String> {
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

    fn get_data(&mut self, symbol: &str) -> Result<(), String>
    {
        log_info!("Starting data stream for symbol: {}", symbol);
        let socket = self.sockets.remove(symbol).ok_or_else(|| format!("No socket found for symbol: {}", symbol))?;
        let sender = self.sender.clone();
        let symbol_owned = symbol.to_owned();
        let sender = Arc::clone(&self.sender);
        let handle = thread::spawn(move || {
            let mut socket = socket;
            loop {
            match socket.read() {
                Ok(Message::Text(text)) => {
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
                    // TODO: Handle close later
                    log_info!("WebSocket connection closed for {}", symbol_owned);
                }
                Ok(Message::Binary(_)) => {
                    log_debug!("Received binary message for {} (ignoring)", symbol_owned);
                }
                Err(e) => {
                    log_error!("WebSocket error for {}: {}", symbol_owned, e);
                }
                _ => {
                    log_warn!("Unknown message type for {}", symbol_owned);
                }
            }
        }
        });

        self.socket_threads.insert(symbol.to_owned(), handle);
        Ok(())
    }
}

// Helper methods to keep the main function clean
impl BinanceContainer {
    fn handle_close(&mut self, symbol: &str) {
        log_info!("WebSocket closed for symbol: {}", symbol);
        self.sockets.remove(symbol);
    }

    fn handle_error(&mut self, symbol: &str, error: WsError) {
        let error_msg = match error {
            WsError::ConnectionClosed => {
                log_warn!("Connection closed for symbol: {}", symbol);
            }
            WsError::AlreadyClosed => {
                log_warn!("Socket already closed for symbol: {}", symbol);
            }
            e => {
                log_error!("WebSocket error for {}: {}", symbol, e);
            }
        };
        
        self.sockets.remove(symbol);
    }
}
