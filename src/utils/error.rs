use config::ConfigError;
use std::env::VarError;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),
    #[error("Logging error: {0}")]
    LoggingError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Watcher error: {0}")]
    WatcherError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] VarError),
    #[error("Parse int error: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("JSON parse error: {0}")]
    JsonParseError(String),
}

pub type ScannerResult<T> = Result<T, AppError>;
