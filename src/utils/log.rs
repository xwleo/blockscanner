use crate::utils::config::LogConfig;
use crate::utils::error::AppError;
use std::path::Path;
use tracing_appender::rolling::{daily, RollingFileAppender};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

pub struct Logger;

impl Logger {
    pub fn init(log_config: &LogConfig) -> Result<(), AppError> {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&log_config.level));

        match log_config.output.as_str() {
            "file" => {
                let file_appender =
                    Self::create_file_appender(&log_config.file_path, &log_config.file_name)?;
                let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
                let subscriber = Registry::default().with(env_filter).with(
                    fmt::layer()
                        .with_writer(non_blocking)
                        .with_timer(fmt::time::UtcTime::rfc_3339())
                        .with_ansi(false)
                        .json(),
                );
                tracing::subscriber::set_global_default(subscriber)
                    .map_err(|e| AppError::LoggingError(e.to_string()))?;
                // 保持 _guard 存活
                std::mem::forget(_guard);
            }
            _ => {
                let subscriber = Registry::default()
                    .with(env_filter)
                    .with(fmt::layer().with_timer(fmt::time::UtcTime::rfc_3339()));
                tracing::subscriber::set_global_default(subscriber)
                    .map_err(|e| AppError::LoggingError(e.to_string()))?;
            }
        }

        Ok(())
    }

    fn create_file_appender(
        log_dir: &str,
        file_name: &str,
    ) -> Result<RollingFileAppender, AppError> {
        let log_path = Path::new(log_dir);
        std::fs::create_dir_all(log_path)?;

        println!("Creating log file in directory: {:?}", log_path);

        Ok(daily(log_path, file_name))
    }
}
