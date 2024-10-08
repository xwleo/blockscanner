use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct TronConfig {
    pub enable: bool,
    pub api_url: String,
    pub start_block: u64,
}

#[derive(Debug, Deserialize)]
pub struct BscConfig {
    pub enable: bool,
    pub api_url: String,
    pub start_block: u64,
}

#[derive(Debug, Deserialize)]
pub struct SchedulerConfig {
    pub interval_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub output: String,
    pub format: String,
    pub file_path: String,
    pub file_name: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub tron: TronConfig,
    pub bsc: BscConfig,
    pub scheduler: SchedulerConfig,
    pub log: LogConfig,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .build()?;

        s.try_deserialize()
    }
}
