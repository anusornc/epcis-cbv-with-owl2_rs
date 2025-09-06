pub mod api;
pub mod config;
pub mod models;
pub mod ontology;
pub mod pipeline;
pub mod storage;
pub mod utils;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, EpcisKgError>;

#[derive(Error, Debug)]
pub enum EpcisKgError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Ontology error: {0}")]
    Ontology(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("RDF parsing error: {0}")]
    RdfParsing(String),
    
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
    
    #[error("IRI parsing error: {0}")]
    IriParse(#[from] oxrdf::IriParseError),
    
    #[error("Blank node ID parsing error: {0}")]
    BlankNodeIdParse(#[from] oxrdf::BlankNodeIdParseError),
}

// Re-export the new AppConfig for backwards compatibility
pub use config::AppConfig as Config;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.database_path, "./data");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
        assert!(config.ontology_paths.is_empty());
    }

    #[test]
    fn test_error_display() {
        let error = EpcisKgError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(error.to_string().contains("I/O error"));
        
        let error = EpcisKgError::Config("Invalid config".to_string());
        assert_eq!(error.to_string(), "Configuration error: Invalid config");
        
        let error = EpcisKgError::Validation("Invalid data".to_string());
        assert_eq!(error.to_string(), "Validation error: Invalid data");
    }

    #[test]
    fn test_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let epcis_error: EpcisKgError = io_error.into();
        assert!(matches!(epcis_error, EpcisKgError::Io(_)));
    }
}