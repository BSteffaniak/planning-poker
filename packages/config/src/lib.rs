use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database_url: Option<String>,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                cors_origins: vec!["*".to_string()],
            },
            database_url: None,
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
            },
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(host) = std::env::var("PLANNING_POKER_HOST") {
            config.server.host = host;
        }

        if let Ok(port) = std::env::var("PLANNING_POKER_PORT") {
            if let Ok(port) = port.parse() {
                config.server.port = port;
            }
        }

        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            config.database_url = Some(database_url);
        }

        if let Ok(log_level) = std::env::var("RUST_LOG") {
            config.logging.level = log_level;
        }

        config
    }

    pub fn merge_with_env(mut self) -> Self {
        let env_config = Self::from_env();

        if env_config.server.host != "0.0.0.0" {
            self.server.host = env_config.server.host;
        }

        if env_config.server.port != 8080 {
            self.server.port = env_config.server.port;
        }

        if env_config.database_url.is_some() {
            self.database_url = env_config.database_url;
        }

        if env_config.logging.level != "info" {
            self.logging.level = env_config.logging.level;
        }

        self
    }
}
