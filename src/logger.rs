use tracing::{info, warn, error, debug, trace};
use tracing_subscriber::EnvFilter;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize the global logger - call this once at application startup
pub fn init_logger(log_level: Option<&str>) {
    INIT.call_once(|| {
        let env_filter = EnvFilter::try_from_default_env()
            .or_else(|_| {
                let level = log_level.unwrap_or("info");
                EnvFilter::try_new(level)
            })
            .unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(false)
            .with_file(true)
            .with_line_number(true)
            .init();
        
        info!("Logger initialized successfully");
    });
}

/// Macro for easy logging throughout the application
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*);
    };
}

/// Logger struct for dependency injection (if needed)
#[derive(Debug, Clone)]
pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Self
    }
    
    pub fn info(&self, message: &str) {
        info!("{}", message);
    }
    
    pub fn error(&self, message: &str) {
        error!("{}", message);
    }
    
    pub fn warn(&self, message: &str) {
        warn!("{}", message);
    }
    
    pub fn debug(&self, message: &str) {
        debug!("{}", message);
    }
    
    pub fn trace(&self, message: &str) {
        trace!("{}", message);
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}