pub mod socket_container;
use serde::{Deserialize, Serialize};

/// Common response structure for API endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}

/// Arbitrage opportunity model (for future use)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub symbol: String,
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit_percentage: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Market data model (for future use)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Market {
    pub exchange: String,
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}