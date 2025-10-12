use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;

use crate::models::SymbolMessage;
use crate::{log_error, log_info};
use crate::socket::socket_container::binance_container::BinanceContainer;

pub struct SocketConsumer {
    symbols: Vec<String>, // ["btcusdt", ...]
    binance_container: BinanceContainer,
    binance_symbol_map: HashMap<String, f64>,
    binance_receiver: Receiver<SymbolMessage>
}

impl SocketConsumer {
    fn new() -> Self {
        let (binance_sender, binance_receiver) = mpsc::channel();
        let binance_sender = Arc::new(binance_sender);

        SocketConsumer { 
            symbols: vec![],
            binance_symbol_map: HashMap::new(),
            binance_container: BinanceContainer::new(binance_sender),
            binance_receiver
        }
    }

    pub fn start_binance_price_monitoring(&mut self) -> Result<(), String> {
        for symbol in &self.symbols {
            let symbol_clone = symbol.clone();
            
            // Start monitoring in a separate thread or task
            match self.binance_container.get_data(&symbol) {
                Ok(_) => {
                    log_info!("Start monitoring {}", symbol_clone);
                }
                Err(e) => {
                    log_error!("Error monitoring {}: {}", symbol_clone, e);
                }
            }
        }

        // Receive messages in the main thread
        for message in self.binance_receiver.iter() {
            println!("Received: {} => {}", message.symbol, message.price);

            // TODO: Add to Hashmap
        }
        Ok(())
    }
}