use tracing::{info, warn, error, debug, trace, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use std::path::PathBuf;
use std::fs;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use parking_lot::Mutex;

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    
    /// Whether to log to console
    pub console_output: bool,
    
    /// Whether to log to file
    pub file_output: bool,
    
    /// Log file directory
    pub log_directory: PathBuf,
    
    /// Maximum log file size in MB
    pub max_file_size_mb: usize,
    
    /// Maximum number of log files to keep
    pub max_files: usize,
    
    /// Whether to include timestamps
    pub include_timestamps: bool,
    
    /// Whether to include request IDs in logs
    pub include_request_ids: bool,
    
    /// Log format (json, text)
    pub format: LogFormat,
}

/// Log format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Text,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            console_output: true,
            file_output: false,
            log_directory: PathBuf::from("./logs"),
            max_file_size_mb: 100,
            max_files: 5,
            include_timestamps: true,
            include_request_ids: true,
            format: LogFormat::Text,
        }
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Log level
    pub level: String,
    
    /// Log message
    pub message: String,
    
    /// Request ID (if applicable)
    pub request_id: Option<String>,
    
    /// User ID (if applicable)
    pub user_id: Option<String>,
    
    /// Operation being performed
    pub operation: Option<String>,
    
    /// Duration of operation in milliseconds
    pub duration_ms: Option<u64>,
    
    /// Additional structured data
    pub metadata: serde_json::Value,
    
    /// Error details (if applicable)
    pub error: Option<LogError>,
}

/// Error details for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogError {
    /// Error type
    pub error_type: String,
    
    /// Error message
    pub message: String,
    
    /// Stack trace (if available)
    pub stack_trace: Option<String>,
    
    /// Error code (if applicable)
    pub error_code: Option<String>,
}

/// Request logger for tracking API requests
pub struct RequestLogger {
    /// Logging configuration
    config: Arc<LoggingConfig>,
    
    /// Request ID for current request
    request_id: Option<String>,
    
    /// User ID for current request
    user_id: Option<String>,
    
    /// Operation start time
    start_time: Option<std::time::Instant>,
}

impl RequestLogger {
    /// Create a new request logger
    pub fn new(config: Arc<LoggingConfig>) -> Self {
        Self {
            config,
            request_id: None,
            user_id: None,
            start_time: None,
        }
    }
    
    /// Set request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
    
    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    /// Start timing an operation
    pub fn start_operation(mut self) -> Self {
        self.start_time = Some(std::time::Instant::now());
        self
    }
    
    /// Log an info message
    pub fn info(&self, message: String, metadata: Option<serde_json::Value>) {
        self.log(Level::INFO, message, metadata, None);
    }
    
    /// Log a warning message
    pub fn warn(&self, message: String, metadata: Option<serde_json::Value>) {
        self.log(Level::WARN, message, metadata, None);
    }
    
    /// Log an error message
    pub fn error(&self, message: String, error: Option<LogError>, metadata: Option<serde_json::Value>) {
        self.log(Level::ERROR, message, metadata, error);
    }
    
    /// Log a debug message
    pub fn debug(&self, message: String, metadata: Option<serde_json::Value>) {
        self.log(Level::DEBUG, message, metadata, None);
    }
    
    /// Log a trace message
    pub fn trace(&self, message: String, metadata: Option<serde_json::Value>) {
        self.log(Level::TRACE, message, metadata, None);
    }
    
    /// Log a message with specified level
    fn log(&self, level: Level, message: String, metadata: Option<serde_json::Value>, error: Option<LogError>) {
        let duration_ms = self.start_time.map(|start| start.elapsed().as_millis() as u64);
        
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: level.to_string(),
            message: message.clone(),
            request_id: self.request_id.clone(),
            user_id: self.user_id.clone(),
            operation: None, // Could be set by calling method
            duration_ms,
            metadata: metadata.unwrap_or_default(),
            error,
        };
        
        // Log based on format
        match self.config.format {
            LogFormat::Json => {
                let json = serde_json::to_string(&entry).unwrap_or_else(|_| message.clone());
                match level {
                    Level::ERROR => error!("{}", json),
                    Level::WARN => warn!("{}", json),
                    Level::INFO => info!("{}", json),
                    Level::DEBUG => debug!("{}", json),
                    Level::TRACE => trace!("{}", json),
                }
            }
            LogFormat::Text => {
                let formatted = self.format_text_entry(&entry);
                match level {
                    Level::ERROR => error!("{}", formatted),
                    Level::WARN => warn!("{}", formatted),
                    Level::INFO => info!("{}", formatted),
                    Level::DEBUG => debug!("{}", formatted),
                    Level::TRACE => trace!("{}", formatted),
                }
            }
        }
    }
    
    /// Format log entry as text
    fn format_text_entry(&self, entry: &LogEntry) -> String {
        let mut parts = Vec::new();
        
        if self.config.include_timestamps {
            parts.push(entry.timestamp.to_rfc3339());
        }
        
        parts.push(format!("[{}]", entry.level));
        
        if let Some(ref req_id) = entry.request_id {
            parts.push(format!("req_id={}", req_id));
        }
        
        if let Some(ref user_id) = entry.user_id {
            parts.push(format!("user_id={}", user_id));
        }
        
        if let Some(duration) = entry.duration_ms {
            parts.push(format!("duration={}ms", duration));
        }
        
        parts.push(entry.message.clone());
        
        if !entry.metadata.is_object() || entry.metadata.as_object().map_or(false, |obj| !obj.is_empty()) {
            parts.push(format!("metadata={}", entry.metadata));
        }
        
        if let Some(ref error) = entry.error {
            parts.push(format!("error={}:{}", error.error_type, error.message));
        }
        
        parts.join(" ")
    }
}

/// Logger for database operations
pub struct DatabaseLogger {
    config: Arc<LoggingConfig>,
}

impl DatabaseLogger {
    /// Create a new database logger
    pub fn new(config: Arc<LoggingConfig>) -> Self {
        Self { config }
    }
    
    /// Log database query
    pub fn log_query(&self, query: &str, duration_ms: u64, success: bool, error: Option<String>) {
        let metadata = serde_json::json!({
            "query_type": self.detect_query_type(query),
            "query_length": query.len(),
            "success": success,
        });
        
        let logger = RequestLogger::new(self.config.clone());
        
        if success {
            logger.info(
                format!("Database query executed successfully in {}ms", duration_ms),
                Some(metadata),
            );
        } else {
            let error_details = LogError {
                error_type: "DatabaseError".to_string(),
                message: error.unwrap_or_else(|| "Unknown database error".to_string()),
                stack_trace: None,
                error_code: None,
            };
            
            logger.error(
                format!("Database query failed after {}ms", duration_ms),
                Some(error_details),
                Some(metadata),
            );
        }
    }
    
    /// Log database connection event
    pub fn log_connection(&self, event: &str, success: bool, error: Option<String>) {
        let metadata = serde_json::json!({
            "event": event,
            "success": success,
        });
        
        let logger = RequestLogger::new(self.config.clone());
        
        if success {
            logger.info("Database connection established".to_string(), Some(metadata));
        } else {
            let error_details = LogError {
                error_type: "ConnectionError".to_string(),
                message: error.unwrap_or_else(|| "Unknown connection error".to_string()),
                stack_trace: None,
                error_code: None,
            };
            
            logger.error(
                "Database connection failed".to_string(),
                Some(error_details),
                Some(metadata),
            );
        }
    }
    
    /// Detect query type from SQL string
    fn detect_query_type(&self, query: &str) -> String {
        let query_upper = query.trim().to_uppercase();
        if query_upper.starts_with("SELECT") {
            "SELECT".to_string()
        } else if query_upper.starts_with("INSERT") {
            "INSERT".to_string()
        } else if query_upper.starts_with("UPDATE") {
            "UPDATE".to_string()
        } else if query_upper.starts_with("DELETE") {
            "DELETE".to_string()
        } else if query_upper.starts_with("CREATE") {
            "CREATE".to_string()
        } else if query_upper.starts_with("DROP") {
            "DROP".to_string()
        } else {
            "UNKNOWN".to_string()
        }
    }
}

/// Logger for reasoning operations
pub struct ReasoningLogger {
    config: Arc<LoggingConfig>,
}

impl ReasoningLogger {
    /// Create a new reasoning logger
    pub fn new(config: Arc<LoggingConfig>) -> Self {
        Self { config }
    }
    
    /// Log reasoning operation
    pub fn log_reasoning(&self, operation: &str, input_triples: usize, output_triples: usize, duration_ms: u64, success: bool, error: Option<String>) {
        let metadata = serde_json::json!({
            "operation": operation,
            "input_triples": input_triples,
            "output_triples": output_triples,
            "success": success,
        });
        
        let logger = RequestLogger::new(self.config.clone());
        
        if success {
            logger.info(
                format!("Reasoning operation completed in {}ms", duration_ms),
                Some(metadata),
            );
        } else {
            let error_details = LogError {
                error_type: "ReasoningError".to_string(),
                message: error.unwrap_or_else(|| "Unknown reasoning error".to_string()),
                stack_trace: None,
                error_code: None,
            };
            
            logger.error(
                format!("Reasoning operation failed after {}ms", duration_ms),
                Some(error_details),
                Some(metadata),
            );
        }
    }
    
    /// Log materialization operation
    pub fn log_materialization(&self, strategy: &str, triples_materialized: usize, duration_ms: u64, success: bool) {
        let metadata = serde_json::json!({
            "strategy": strategy,
            "triples_materialized": triples_materialized,
            "success": success,
        });
        
        let logger = RequestLogger::new(self.config.clone());
        
        if success {
            logger.info(
                format!("Materialization completed with {} triples in {}ms", triples_materialized, duration_ms),
                Some(metadata),
            );
        } else {
            let error_details = LogError {
                error_type: "MaterializationError".to_string(),
                message: "Materialization operation failed".to_string(),
                stack_trace: None,
                error_code: None,
            };
            
            logger.error(
                format!("Materialization failed after {}ms", duration_ms),
                Some(error_details),
                Some(metadata),
            );
        }
    }
}

/// Initialize logging system
pub fn init_logging(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create log directory if it doesn't exist
    if config.file_output {
        fs::create_dir_all(&config.log_directory)?;
    }
    
    // Parse log level
    let level = config.level.parse::<Level>()
        .unwrap_or(Level::INFO);
    
    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true)
                .compact()
        )
        .with(
            tracing_subscriber::filter::LevelFilter::from_level(level)
        )
        .init();
    
    info!("Logging system initialized with level: {}", level);
    
    Ok(())
}

/// Get a request logger instance
pub fn get_request_logger(config: Arc<LoggingConfig>) -> RequestLogger {
    RequestLogger::new(config)
}

/// Get a database logger instance
pub fn get_database_logger(config: Arc<LoggingConfig>) -> DatabaseLogger {
    DatabaseLogger::new(config)
}

/// Get a reasoning logger instance
pub fn get_reasoning_logger(config: Arc<LoggingConfig>) -> ReasoningLogger {
    ReasoningLogger::new(config)
}