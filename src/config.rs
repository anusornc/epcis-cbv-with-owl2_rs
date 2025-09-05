use crate::EpcisKgError;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database_path: String,
    pub server_port: u16,
    pub log_level: String,
    pub ontology_paths: Vec<String>,
    pub reasoning: ReasoningConfig,
    pub sparql: SparqlConfig,
    pub server: ServerConfig,
    pub persistence: PersistenceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    pub default_profile: String,
    pub enable_inference: bool,
    pub max_inference_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparqlConfig {
    pub max_query_time: u64,
    pub max_results: usize,
    pub enable_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub enable_cors: bool,
    pub cors_origins: Vec<String>,
    pub request_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub auto_save: bool,
    pub save_interval: u64,
    pub backup_on_startup: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_path: "./data".to_string(),
            server_port: 8080,
            log_level: "info".to_string(),
            ontology_paths: vec![
                "ontologies/epcis2.ttl".to_string(),
                "ontologies/cbv.ttl".to_string(),
            ],
            reasoning: ReasoningConfig::default(),
            sparql: SparqlConfig::default(),
            server: ServerConfig::default(),
            persistence: PersistenceConfig::default(),
        }
    }
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            default_profile: "el".to_string(),
            enable_inference: true,
            max_inference_time: 30,
        }
    }
}

impl Default for SparqlConfig {
    fn default() -> Self {
        Self {
            max_query_time: 60,
            max_results: 1000,
            enable_updates: true,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            enable_cors: true,
            cors_origins: vec!["*".to_string()],
            request_timeout: 30,
        }
    }
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            auto_save: true,
            save_interval: 300,
            backup_on_startup: true,
        }
    }
}

impl AppConfig {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, EpcisKgError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(EpcisKgError::Config(format!(
                "Configuration file not found: {}",
                path.display()
            )));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| EpcisKgError::Io(e))?;

        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| EpcisKgError::Config(format!(
                "Failed to parse configuration file: {}",
                e
            )))?;

        Ok(config)
    }

    /// Load configuration from file or use defaults if file doesn't exist
    pub fn from_file_or_default<P: AsRef<Path>>(path: P) -> Result<Self, EpcisKgError> {
        let path = path.as_ref();
        if path.exists() {
            Self::from_file(path)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to a TOML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), EpcisKgError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| EpcisKgError::Config(format!(
                "Failed to serialize configuration: {}",
                e
            )))?;

        std::fs::write(path, content)
            .map_err(|e| EpcisKgError::Io(e))?;

        Ok(())
    }

    /// Validate configuration settings
    pub fn validate(&self) -> Result<(), EpcisKgError> {
        // Validate database path
        if self.database_path.is_empty() {
            return Err(EpcisKgError::Config(
                "Database path cannot be empty".to_string(),
            ));
        }

        // Validate server port
        if self.server_port == 0 {
            return Err(EpcisKgError::Config(
                "Server port must be greater than 0".to_string(),
            ));
        }

        // Validate log level
        match self.log_level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(EpcisKgError::Config(format!(
                    "Invalid log level: {}. Must be one of: trace, debug, info, warn, error",
                    self.log_level
                )));
            }
        }

        // Validate reasoning profile
        match self.reasoning.default_profile.as_str() {
            "el" | "ql" | "rl" => {}
            _ => {
                return Err(EpcisKgError::Config(format!(
                    "Invalid reasoning profile: {}. Must be one of: el, ql, rl",
                    self.reasoning.default_profile
                )));
            }
        }

        // Validate timeout values
        if self.reasoning.max_inference_time == 0 {
            return Err(EpcisKgError::Config(
                "Max inference time must be greater than 0".to_string(),
            ));
        }

        if self.sparql.max_query_time == 0 {
            return Err(EpcisKgError::Config(
                "Max query time must be greater than 0".to_string(),
            ));
        }

        if self.server.request_timeout == 0 {
            return Err(EpcisKgError::Config(
                "Request timeout must be greater than 0".to_string(),
            ));
        }

        if self.persistence.save_interval == 0 {
            return Err(EpcisKgError::Config(
                "Save interval must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Create a new configuration with specific overrides
    pub fn with_overrides<F>(mut self, overrides: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        overrides(&mut self);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database_path, "./data");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.reasoning.default_profile, "el");
        assert!(config.reasoning.enable_inference);
        assert_eq!(config.ontology_paths.len(), 2);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        
        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid log level should fail
        config.log_level = "invalid".to_string();
        assert!(config.validate().is_err());

        // Invalid reasoning profile should fail
        config.log_level = "info".to_string();
        config.reasoning.default_profile = "invalid".to_string();
        assert!(config.validate().is_err());

        // Zero port should fail
        config.reasoning.default_profile = "el".to_string();
        config.server_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_file_io() {
        let config = AppConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Save config
        config.to_file(path).unwrap();

        // Load config
        let loaded_config = AppConfig::from_file(path).unwrap();
        assert_eq!(config.database_path, loaded_config.database_path);
        assert_eq!(config.server_port, loaded_config.server_port);
        assert_eq!(config.log_level, loaded_config.log_level);
    }

    #[test]
    fn test_config_with_overrides() {
        let config = AppConfig::default()
            .with_overrides(|c| {
                c.server_port = 9090;
                c.log_level = "debug".to_string();
            });

        assert_eq!(config.server_port, 9090);
        assert_eq!(config.log_level, "debug");
        // Other values should remain default
        assert_eq!(config.database_path, "./data");
    }
}