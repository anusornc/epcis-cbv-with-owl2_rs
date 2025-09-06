use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;
use serde::{Serialize, Deserialize};
use parking_lot::Mutex;

/// System health and metrics monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// System uptime in seconds
    pub uptime_seconds: u64,
    
    /// Total number of requests processed
    pub total_requests: u64,
    
    /// Number of successful requests
    pub successful_requests: u64,
    
    /// Number of failed requests
    pub failed_requests: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// Current memory usage in MB
    pub memory_usage_mb: u64,
    
    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: f64,
    
    /// Number of active connections
    pub active_connections: u32,
    
    /// Database metrics
    pub database_metrics: DatabaseMetrics,
    
    /// Reasoning metrics
    pub reasoning_metrics: ReasoningMetrics,
    
    /// API endpoint metrics
    pub api_metrics: ApiMetrics,
}

/// Database-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    /// Total triples stored
    pub total_triples: u64,
    
    /// Number of named graphs
    pub named_graphs: u32,
    
    /// Average query time in milliseconds
    pub avg_query_time_ms: f64,
    
    /// Cache hit ratio (0-1)
    pub cache_hit_ratio: f64,
    
    /// Storage size in MB
    pub storage_size_mb: u64,
}

/// Reasoning-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningMetrics {
    /// Total inferences performed
    pub total_inferences: u64,
    
    /// Average inference time in milliseconds
    pub avg_inference_time_ms: f64,
    
    /// Number of materialized triples
    pub materialized_triples: u64,
    
    /// Cache hit ratio for reasoning (0-1)
    pub reasoning_cache_hit_ratio: f64,
    
    /// Current materialization strategy
    pub materialization_strategy: String,
}

/// API endpoint-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    /// SPARQL query metrics
    pub sparql_metrics: EndpointMetrics,
    
    /// EPCIS event processing metrics
    pub epcis_metrics: EndpointMetrics,
    
    /// Inference metrics
    pub inference_metrics: EndpointMetrics,
    
    /// Materialization metrics
    pub materialization_metrics: EndpointMetrics,
}

/// Individual endpoint metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    /// Number of requests to this endpoint
    pub request_count: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// Error rate (0-1)
    pub error_rate: f64,
    
    /// Last request timestamp
    pub last_request: Option<String>,
}

/// Performance alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Response time threshold in milliseconds
    pub response_time_threshold_ms: u64,
    
    /// Error rate threshold (0-1)
    pub error_rate_threshold: f64,
    
    /// Memory usage threshold in MB
    pub memory_threshold_mb: u64,
    
    /// CPU usage threshold (0-100)
    pub cpu_threshold_percent: f64,
    
    /// Database size threshold in MB
    pub db_size_threshold_mb: u64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            response_time_threshold_ms: 5000,
            error_rate_threshold: 0.05,
            memory_threshold_mb: 4096,
            cpu_threshold_percent: 80.0,
            db_size_threshold_mb: 10240,
        }
    }
}

/// System alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAlert {
    /// Alert ID
    pub id: String,
    
    /// Alert severity
    pub severity: AlertSeverity,
    
    /// Alert message
    pub message: String,
    
    /// Alert type
    pub alert_type: AlertType,
    
    /// Timestamp when alert was generated
    pub timestamp: String,
    
    /// Whether alert has been acknowledged
    pub acknowledged: bool,
    
    /// Additional context
    pub context: serde_json::Value,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    Performance,
    Memory,
    Cpu,
    Database,
    Api,
    System,
}

/// Request tracking for monitoring
#[derive(Debug, Clone)]
pub struct RequestTracker {
    /// Request start time
    pub start_time: Instant,
    
    /// Request endpoint
    pub endpoint: String,
    
    /// Request method
    pub method: String,
    
    /// Request ID
    pub request_id: String,
    
    /// Request status
    pub status: RequestStatus,
}

/// Request status
#[derive(Debug, Clone)]
pub enum RequestStatus {
    InProgress,
    Success,
    Failed,
}

impl RequestTracker {
    /// Create a new request tracker
    pub fn new(endpoint: String, method: String) -> Self {
        Self {
            start_time: Instant::now(),
            endpoint,
            method,
            request_id: uuid::Uuid::new_v4().to_string(),
            status: RequestStatus::InProgress,
        }
    }
    
    /// Complete the request with success status
    pub fn complete_success(mut self) -> RequestMetrics {
        self.status = RequestStatus::Success;
        RequestMetrics {
            request_id: self.request_id,
            endpoint: self.endpoint,
            method: self.method,
            duration_ms: self.start_time.elapsed().as_millis() as u64,
            success: true,
            error_message: None,
        }
    }
    
    /// Complete the request with failure status
    pub fn complete_failure(mut self, error_message: String) -> RequestMetrics {
        self.status = RequestStatus::Failed;
        RequestMetrics {
            request_id: self.request_id,
            endpoint: self.endpoint,
            method: self.method,
            duration_ms: self.start_time.elapsed().as_millis() as u64,
            success: false,
            error_message: Some(error_message),
        }
    }
}

/// Request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Request ID
    pub request_id: String,
    
    /// Request endpoint
    pub endpoint: String,
    
    /// Request method
    pub method: String,
    
    /// Request duration in milliseconds
    pub duration_ms: u64,
    
    /// Whether request was successful
    pub success: bool,
    
    /// Error message if request failed
    pub error_message: Option<String>,
}

/// System monitor for collecting metrics
pub struct SystemMonitor {
    /// System start time
    start_time: Instant,
    
    /// Request counters
    total_requests: Arc<AtomicU64>,
    successful_requests: Arc<AtomicU64>,
    failed_requests: Arc<AtomicU64>,
    
    /// Performance metrics
    response_times: Arc<Mutex<Vec<u64>>>,
    
    /// Active connections
    active_connections: Arc<AtomicU32>,
    
    /// Alert configuration
    alert_config: AlertConfig,
    
    /// Recent alerts
    alerts: Arc<Mutex<Vec<SystemAlert>>>,
    
    /// Request metrics history
    request_history: Arc<Mutex<Vec<RequestMetrics>>>,
}

impl SystemMonitor {
    /// Create a new system monitor
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            response_times: Arc::new(Mutex::new(Vec::new())),
            active_connections: Arc::new(AtomicU32::new(0)),
            alert_config: AlertConfig::default(),
            alerts: Arc::new(Mutex::new(Vec::new())),
            request_history: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Create a new system monitor with custom alert configuration
    pub fn with_alert_config(alert_config: AlertConfig) -> Self {
        Self {
            alert_config,
            ..Self::new()
        }
    }
    
    /// Track a new request
    pub fn track_request(&self, endpoint: String, method: String) -> RequestTracker {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        RequestTracker::new(endpoint, method)
    }
    
    /// Record a successful request
    pub fn record_success(&self, duration_ms: u64) {
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.response_times.lock().push(duration_ms);
        
        // Keep only last 1000 response times for average calculation
        let mut times = self.response_times.lock();
        if times.len() > 1000 {
            times.remove(0);
        }
    }
    
    /// Record a failed request
    pub fn record_failure(&self, duration_ms: u64) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
        self.response_times.lock().push(duration_ms);
        
        // Keep only last 1000 response times for average calculation
        let mut times = self.response_times.lock();
        if times.len() > 1000 {
            times.remove(0);
        }
    }
    
    /// Increment active connections
    pub fn increment_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Decrement active connections
    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
    
    /// Add a request to history
    pub fn add_request_to_history(&self, metrics: RequestMetrics) {
        let mut history = self.request_history.lock();
        history.push(metrics);
        
        // Keep only last 1000 requests
        if history.len() > 1000 {
            history.remove(0);
        }
    }
    
    /// Generate system alert
    pub fn add_alert(&self, severity: AlertSeverity, alert_type: AlertType, message: String, context: serde_json::Value) {
        let alert = SystemAlert {
            id: uuid::Uuid::new_v4().to_string(),
            severity,
            alert_type,
            message,
            timestamp: chrono::Utc::now().to_rfc3339(),
            acknowledged: false,
            context,
        };
        
        self.alerts.lock().push(alert);
        
        // Keep only last 100 alerts
        let mut alerts = self.alerts.lock();
        if alerts.len() > 100 {
            alerts.remove(0);
        }
    }
    
    /// Get current system metrics
    pub fn get_metrics(&self) -> SystemMetrics {
        let uptime = self.start_time.elapsed();
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let successful_requests = self.successful_requests.load(Ordering::Relaxed);
        let failed_requests = self.failed_requests.load(Ordering::Relaxed);
        
        // Calculate average response time
        let avg_response_time = {
            let times = self.response_times.lock();
            if times.is_empty() {
                0.0
            } else {
                times.iter().sum::<u64>() as f64 / times.len() as f64
            }
        };
        
        // Get memory usage (platform-specific implementation would go here)
        let memory_usage_mb = self.get_memory_usage();
        
        // Get CPU usage (platform-specific implementation would go here)
        let cpu_usage_percent = self.get_cpu_usage();
        
        SystemMetrics {
            uptime_seconds: uptime.as_secs(),
            total_requests,
            successful_requests,
            failed_requests,
            avg_response_time_ms: avg_response_time,
            memory_usage_mb,
            cpu_usage_percent,
            active_connections: self.active_connections.load(Ordering::Relaxed),
            database_metrics: DatabaseMetrics {
                total_triples: 0, // Would be populated from actual database
                named_graphs: 0,
                avg_query_time_ms: 0.0,
                cache_hit_ratio: 0.0,
                storage_size_mb: 0,
            },
            reasoning_metrics: ReasoningMetrics {
                total_inferences: 0,
                avg_inference_time_ms: 0.0,
                materialized_triples: 0,
                reasoning_cache_hit_ratio: 0.0,
                materialization_strategy: "Incremental".to_string(),
            },
            api_metrics: ApiMetrics {
                sparql_metrics: EndpointMetrics {
                    request_count: 0,
                    avg_response_time_ms: 0.0,
                    error_rate: 0.0,
                    last_request: None,
                },
                epcis_metrics: EndpointMetrics {
                    request_count: 0,
                    avg_response_time_ms: 0.0,
                    error_rate: 0.0,
                    last_request: None,
                },
                inference_metrics: EndpointMetrics {
                    request_count: 0,
                    avg_response_time_ms: 0.0,
                    error_rate: 0.0,
                    last_request: None,
                },
                materialization_metrics: EndpointMetrics {
                    request_count: 0,
                    avg_response_time_ms: 0.0,
                    error_rate: 0.0,
                    last_request: None,
                },
            },
        }
    }
    
    /// Get recent alerts
    pub fn get_alerts(&self, limit: Option<usize>) -> Vec<SystemAlert> {
        let alerts = self.alerts.lock();
        match limit {
            Some(limit) => alerts.iter().rev().take(limit).cloned().collect(),
            None => alerts.clone(),
        }
    }
    
    /// Get request history
    pub fn get_request_history(&self, limit: Option<usize>) -> Vec<RequestMetrics> {
        let history = self.request_history.lock();
        match limit {
            Some(limit) => history.iter().rev().take(limit).cloned().collect(),
            None => history.clone(),
        }
    }
    
    /// Check for system alerts based on current metrics
    pub fn check_alerts(&self) -> Vec<SystemAlert> {
        let metrics = self.get_metrics();
        let mut alerts = Vec::new();
        
        // Check response time
        if metrics.avg_response_time_ms > self.alert_config.response_time_threshold_ms as f64 {
            alerts.push(SystemAlert {
                id: uuid::Uuid::new_v4().to_string(),
                severity: AlertSeverity::Warning,
                alert_type: AlertType::Performance,
                message: format!("Average response time ({:.2}ms) exceeds threshold ({}ms)", 
                    metrics.avg_response_time_ms, self.alert_config.response_time_threshold_ms),
                timestamp: chrono::Utc::now().to_rfc3339(),
                acknowledged: false,
                context: serde_json::json!({"current_avg_response_time": metrics.avg_response_time_ms}),
            });
        }
        
        // Check memory usage
        if metrics.memory_usage_mb > self.alert_config.memory_threshold_mb {
            alerts.push(SystemAlert {
                id: uuid::Uuid::new_v4().to_string(),
                severity: AlertSeverity::Warning,
                alert_type: AlertType::Memory,
                message: format!("Memory usage ({}MB) exceeds threshold ({}MB)", 
                    metrics.memory_usage_mb, self.alert_config.memory_threshold_mb),
                timestamp: chrono::Utc::now().to_rfc3339(),
                acknowledged: false,
                context: serde_json::json!({"current_memory_usage_mb": metrics.memory_usage_mb}),
            });
        }
        
        // Check CPU usage
        if metrics.cpu_usage_percent > self.alert_config.cpu_threshold_percent {
            alerts.push(SystemAlert {
                id: uuid::Uuid::new_v4().to_string(),
                severity: AlertSeverity::Warning,
                alert_type: AlertType::Cpu,
                message: format!("CPU usage ({:.2}%) exceeds threshold ({:.2}%)", 
                    metrics.cpu_usage_percent, self.alert_config.cpu_threshold_percent),
                timestamp: chrono::Utc::now().to_rfc3339(),
                acknowledged: false,
                context: serde_json::json!({"current_cpu_usage_percent": metrics.cpu_usage_percent}),
            });
        }
        
        alerts
    }
    
    /// Get memory usage (placeholder implementation)
    fn get_memory_usage(&self) -> u64 {
        // In a real implementation, this would use platform-specific APIs
        // For now, return a reasonable placeholder
        512
    }
    
    /// Get CPU usage (placeholder implementation)
    fn get_cpu_usage(&self) -> f64 {
        // In a real implementation, this would use platform-specific APIs
        // For now, return a reasonable placeholder
        25.0
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}