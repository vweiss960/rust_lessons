#![cfg(feature = "rest-api")]
//! Configuration management for REST API server
//!
//! Handles loading configuration from JSON files and environment variables.
//! Supports specifying database path, server port, and other settings.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// REST API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to SQLite database file
    #[serde(default = "default_db_path")]
    pub path: String,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Port to listen on
    #[serde(default = "default_port")]
    pub port: u16,

    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,
}

fn default_db_path() -> String {
    "analysis.db".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: default_db_path(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

/// Errors that can occur during configuration loading
#[derive(Debug)]
pub enum ConfigError {
    /// File not found
    FileNotFound(String),
    /// JSON parsing error
    ParseError(String),
    /// IO error
    IoError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => write!(f, "Config file not found: {}", path),
            ConfigError::ParseError(msg) => write!(f, "Failed to parse config: {}", msg),
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Load configuration from a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to the JSON configuration file
    ///
    /// # Example
    /// ```ignore
    /// let config = Config::from_file("config.json")?;
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.display().to_string()));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Load configuration from file, or use default if file doesn't exist
    ///
    /// # Arguments
    /// * `path` - Path to the JSON configuration file
    ///
    /// # Returns
    /// Configuration loaded from file, or defaults if file not found
    pub fn from_file_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::from_file(path.as_ref()) {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }

    /// Get the full listen address (host:port)
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Override database path
    pub fn with_db_path(mut self, path: impl Into<String>) -> Self {
        self.database.path = path.into();
        self
    }

    /// Override server port
    pub fn with_port(mut self, port: u16) -> Self {
        self.server.port = port;
        self
    }

    /// Override server host
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.server.host = host.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.database.path, "analysis.db");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.host, "127.0.0.1");
    }

    #[test]
    fn test_listen_addr() {
        let config = Config::default();
        assert_eq!(config.listen_addr(), "127.0.0.1:3000");
    }

    #[test]
    fn test_with_db_path() {
        let config = Config::default().with_db_path("custom.db");
        assert_eq!(config.database.path, "custom.db");
    }

    #[test]
    fn test_with_port() {
        let config = Config::default().with_port(8080);
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_json_parsing() {
        let json = r#"{
            "database": {
                "path": "mydata.db"
            },
            "server": {
                "port": 8080,
                "host": "0.0.0.0"
            }
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.database.path, "mydata.db");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "0.0.0.0");
    }

    #[test]
    fn test_json_partial() {
        let json = r#"{"database": {"path": "custom.db"}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.database.path, "custom.db");
        assert_eq!(config.server.port, 3000); // Should use default
    }
}
