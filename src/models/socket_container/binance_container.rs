use std::{collections::HashMap, net::TcpStream};
use url::Url;
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};

use crate::{log_debug, log_error, log_info, log_warn, models::socket_container::socket_container::ISocketContainer};

pub struct BinanceContainer {
    symbols: Vec<String>, // ["btcusdt", ...]
    endpoints: HashMap<String, String>,
    sockets: Vec<WebSocket<MaybeTlsStream<TcpStream>>>
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
            sockets: vec![]
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
                    self.sockets.push(socket);
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
}

impl ISocketContainer for BinanceContainer {
    fn intialize(&self) {
        todo!()
    }

    fn connect(&self) {
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

    fn disconnect(&self) {
        log_info!("Disconnecting from Binance WebSocket");
        // Disconnect logic here
    }

    fn add_symbol(&self) {
        todo!()
    }

    fn remove_symbol(&self) {
        todo!()
    }
}