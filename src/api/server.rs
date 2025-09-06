use crate::config::AppConfig;
use crate::storage::oxigraph_store::OxigraphStore;
use crate::ontology::reasoner::OntologyReasoner;
use crate::pipeline::EpcisEventPipeline;
use crate::models::events::ProcessingResult;
use crate::monitoring::metrics::{SystemMonitor, AlertSeverity, AlertConfig};
use crate::monitoring::logging::LoggingConfig;
use crate::EpcisKgError;
use axum::{
    extract::Query,
    response::{Json, Response, IntoResponse},
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex, RwLock};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing::info;

pub struct WebServer {
    config: Arc<AppConfig>,
    store: Arc<Mutex<OxigraphStore>>,
    reasoner: Arc<RwLock<OntologyReasoner>>,
    pipeline: Arc<EpcisEventPipeline>,
    system_monitor: Arc<SystemMonitor>,
    logging_config: Arc<LoggingConfig>,
}

impl WebServer {
    pub async fn new(config: AppConfig, store: OxigraphStore) -> Result<Self, EpcisKgError> {
        let reasoner = OntologyReasoner::with_store(store.clone());
        let pipeline = EpcisEventPipeline::new(config.clone(), store.clone(), reasoner.clone()).await?;
        
        // Initialize monitoring
        let alert_config = AlertConfig::default();
        let system_monitor = Arc::new(SystemMonitor::with_alert_config(alert_config));
        
        // Initialize logging
        let logging_config = Arc::new(LoggingConfig::default());
        
        Ok(Self {
            config: Arc::new(config),
            store: Arc::new(Mutex::new(store)),
            reasoner: Arc::new(RwLock::new(reasoner)),
            pipeline: Arc::new(pipeline),
            system_monitor,
            logging_config,
        })
    }
    
    pub async fn run(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.create_app();
        
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
        
        info!("Starting web server on http://{}", addr);
        info!("Available endpoints:");
        info!("  GET  /health - Health check");
        info!("  GET  / - API information");
        info!("  GET  /api/v1/sparql - SPARQL endpoint (GET)");
        info!("  POST /api/v1/sparql - SPARQL endpoint (POST)");
        info!("  POST /api/v1/sparql/query - SPARQL query execution");
        info!("  GET  /api/v1/statistics - Store statistics");
        info!("  GET  /api/v1/ontologies - List ontologies");
        info!("  POST /api/v1/ontologies - Load ontology");
        info!("  POST /api/v1/events - Process EPCIS events");
        info!("  GET  /api/v1/events - List recent events");
        info!("  POST /api/v1/inference - Perform reasoning");
        info!("  GET  /api/v1/inference/stats - Get inference statistics");
        info!("  POST /api/v1/materialize - Manage materialized triples");
        info!("  GET  /api/v1/performance - Get performance metrics");
        info!("  GET  /api/v1/monitoring/metrics - Get system metrics");
        info!("  GET  /api/v1/monitoring/alerts - Get system alerts");
        info!("  GET  /api/v1/monitoring/health - Enhanced health check");
        info!("  POST /api/v1/monitoring/alerts/clear - Clear alerts");
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }
    
    fn create_app(&self) -> Router<()> {
        // Create CORS layer based on configuration
        let cors_layer = if self.config.server.enable_cors {
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        } else {
            CorsLayer::new()
        };
        
        // Create main router
        let app = Router::new()
            .route("/", get(root_handler))
            .route("/health", get(health_handler))
            .nest("/api/v1", self.create_api_router())
            .layer(cors_layer)
            .layer(TraceLayer::new_for_http());
        
        app
    }
    
    fn create_api_router(&self) -> Router<()> {
        Router::new()
            .route("/test", get(|| async { "Hello World" }))
            .route("/statistics", get(api_statistics))
            .route("/sparql", get(api_sparql_get).post(api_sparql_post))
            .route("/sparql/query", post(api_sparql_execute))
            .route("/ontologies", get(api_list_ontologies).post(api_load_ontology))
            .route("/events", get(api_list_events).post(api_process_event))
            .route("/inference", post(api_perform_inference))
            .route("/inference/stats", get(api_inference_stats))
            .route("/materialize", post(api_manage_materialized))
            .route("/performance", get(api_performance_metrics))
            .route("/config", get(api_config))
            .route("/monitoring/metrics", get(api_monitoring_metrics))
            .route("/monitoring/alerts", get(api_monitoring_alerts))
            .route("/monitoring/health", get(api_monitoring_health))
            .route("/monitoring/alerts/clear", post(api_clear_alerts))
    }
}

// Clone implementation for Axum state
impl Clone for WebServer {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            store: Arc::clone(&self.store),
            reasoner: Arc::clone(&self.reasoner),
            pipeline: Arc::clone(&self.pipeline),
            system_monitor: Arc::clone(&self.system_monitor),
            logging_config: Arc::clone(&self.logging_config),
        }
    }
}


// Root handler
async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "EPCIS Knowledge Graph",
        "version": "0.1.0",
        "description": "EPCIS Knowledge Graph with OWL 2 reasoning and SPARQL querying",
        "endpoints": {
            "health": "GET /health",
            "sparql": "GET/POST /api/v1/sparql",
            "statistics": "GET /api/v1/statistics",
            "ontologies": "GET/POST /api/v1/ontologies",
            "events": "GET/POST /api/v1/events",
            "inference": "POST /api/v1/inference",
            "materialize": "POST /api/v1/materialize",
            "performance": "GET /api/v1/performance",
            "config": "GET /api/v1/config"
        }
    }))
}

// Health check handler
async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "epcis-knowledge-graph",
        "version": "0.1.0"
    }))
}

// API Handlers with proper state management
async fn api_sparql_get(
    Query(params): Query<crate::api::sparql::SparqlQuery>,
) -> Result<Json<serde_json::Value>, Json<serde_json::Value>> {
    // Simple SPARQL endpoint for now - return basic response
    Ok(Json(serde_json::json!({
        "results": "SPARQL query received",
        "query": params.query,
        "query_type": crate::api::sparql::determine_query_type(&params.query),
        "execution_time_ms": 0,
        "status": "success"
    })))
}

async fn api_sparql_post(
    Json(payload): Json<crate::api::sparql::SparqlQuery>,
) -> Result<Json<serde_json::Value>, Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "results": "SPARQL query received via POST",
        "query": payload.query,
        "query_type": crate::api::sparql::determine_query_type(&payload.query),
        "execution_time_ms": 0,
        "status": "success"
    })))
}

async fn api_sparql_execute(
    Json(payload): Json<crate::api::sparql::SparqlQuery>,
) -> Result<Response, Json<serde_json::Value>> {
    let response = serde_json::json!({
        "results": "SPARQL execute endpoint",
        "query": payload.query,
        "query_type": crate::api::sparql::determine_query_type(&payload.query),
        "execution_time_ms": 0,
        "status": "success"
    });
    
    Ok(Json(response).into_response())
}

async fn api_statistics(
) -> Result<Json<serde_json::Value>, Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "status": "operational",
        "total_triples": 0,
        "named_graphs": 0,
        "reasoning_enabled": true,
        "message": "Statistics endpoint - integration with Oxigraph pending"
    })))
}

async fn api_list_ontologies(
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ontologies": ["ontologies/epcis2.ttl", "ontologies/cbv.ttl"],
        "loaded_graphs": 0,
        "total_triples": 0,
        "status": "operational",
        "reasoning_enabled": true,
        "materialization_strategy": "Incremental"
    }))
}

#[derive(serde::Deserialize)]
struct OntologyLoadRequest {
    pub file_path: String,
    pub graph_name: Option<String>,
}

async fn api_load_ontology(
    Json(payload): Json<OntologyLoadRequest>,
) -> Result<Json<serde_json::Value>, Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Ontology loading endpoint (simplified)",
        "file": payload.file_path,
        "graph_name": payload.graph_name,
        "triples_loaded": 0,
        "total_inferences": 0,
        "inference_time_ms": 0
    })))
}

#[derive(serde::Deserialize)]
struct EventProcessRequest {
    pub events: Vec<crate::models::epcis::EpcisEvent>,
    pub validate: Option<bool>,
    pub infer: Option<bool>,
}

async fn api_process_event(
    Json(payload): Json<EventProcessRequest>,
) -> Result<Json<serde_json::Value>, Json<serde_json::Value>> {
    let validate = payload.validate.unwrap_or(true);
    let infer = payload.infer.unwrap_or(true);
    
    let mut results = Vec::new();
    let mut total_triples = 0;
    let mut total_inferences = 0;
    
    // Simplified event processing simulation
    for event in &payload.events {
        let processing_result = ProcessingResult {
            event_id: event.event_id.clone(),
            success: true,
            processing_time_ms: 10,
            error: None,
            triples_generated: 5,
            inferences_made: if validate && infer { 1 } else { 0 },
        };
        
        total_triples += processing_result.triples_generated;
        total_inferences += processing_result.inferences_made;
        results.push(serde_json::json!({
            "event_id": event.event_id,
            "success": true,
            "triples_generated": processing_result.triples_generated,
            "inferences_made": processing_result.inferences_made,
            "processing_time_ms": processing_result.processing_time_ms,
            "note": "Simplified processing"
        }));
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "events_processed": results.len(),
        "total_triples_generated": total_triples,
        "total_inferences_made": total_inferences,
        "validation_enabled": validate,
        "inference_enabled": infer,
        "results": results
    })))
}

async fn api_list_events(
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "events": [],
        "total_events": 0,
        "message": "Event listing functionality to be implemented"
    }))
}

#[derive(serde::Deserialize)]
struct InferenceRequest {
    pub strategy: Option<String>,
    pub clear_existing: Option<bool>,
}

async fn api_perform_inference(
    Json(payload): Json<InferenceRequest>,
) -> Result<Json<serde_json::Value>, Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Inference endpoint (simplified)",
        "strategy": payload.strategy,
        "clear_existing": payload.clear_existing,
        "inferences_performed": 0,
        "materialized_triples_count": 0
    })))
}

async fn api_inference_stats(
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "message": "Inference stats endpoint (simplified)",
        "statistics": {
            "total_inferences": 0,
            "materialized_triples_count": 0,
            "total_processing_time_ms": 0
        },
        "materialized_triples": {
            "total_graphs": 0,
            "total_triples": 0,
            "by_graph": {}
        },
        "performance_metrics": {
            "cache_hits": 0,
            "cache_misses": 0,
            "average_processing_time_ms": 0.0
        }
    }))
}

#[derive(serde::Deserialize)]
struct MaterializationRequest {
    pub action: String,
    pub graph_name: Option<String>,
}

async fn api_manage_materialized(
    Json(payload): Json<MaterializationRequest>,
) -> Result<Json<serde_json::Value>, Json<serde_json::Value>> {
    match payload.action.to_lowercase().as_str() {
        "clear" => {
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Cleared materialized triples (simplified)",
                "action": "clear",
                "triples_cleared": 0
            })))
        },
        "show" => {
            let materialized = if let Some(graph_name) = &payload.graph_name {
                serde_json::json!({
                    "graph_name": graph_name,
                    "triples": [],
                    "total_count": 0,
                    "message": "No triples found for this graph"
                })
            } else {
                serde_json::json!({
                    "all_graphs": {},
                    "total_triples": 0
                })
            };
            
            Ok(Json(serde_json::json!({
                "success": true,
                "materialized_triples": materialized,
                "action": "show"
            })))
        },
        _ => {
            Ok(Json(serde_json::json!({
                "success": false,
                "message": "Unknown action. Use 'clear' or 'show'",
                "action": payload.action
            })))
        }
    }
}

async fn api_performance_metrics(
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "message": "Performance metrics endpoint (simplified)",
        "metrics": {
            "cache_hits": 0,
            "cache_misses": 0,
            "average_processing_time_ms": 0.0
        },
        "report": "Performance report not available",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn api_config(
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "operational",
        "version": "0.1.0",
        "reasoning_enabled": true,
        "materialization_strategy": "Incremental",
        "parallel_processing": true,
        "cache_size_limit": 1000,
        "batch_size": 100,
        "performance_optimization": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Monitoring API Handlers
async fn api_monitoring_metrics(
) -> Json<serde_json::Value> {
    let monitor = SystemMonitor::new();
    let metrics = monitor.get_metrics();
    
    Json(serde_json::json!({
        "success": true,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "metrics": metrics
    }))
}

async fn api_monitoring_alerts(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let monitor = SystemMonitor::new();
    let limit = params.get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);
    
    let alerts = monitor.get_alerts(Some(limit));
    let active_alerts = monitor.check_alerts();
    
    Json(serde_json::json!({
        "success": true,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "alerts": alerts,
        "active_alerts": active_alerts,
        "total_alerts": alerts.len(),
        "active_count": active_alerts.len()
    }))
}

async fn api_monitoring_health(
) -> Json<serde_json::Value> {
    let monitor = SystemMonitor::new();
    let metrics = monitor.get_metrics();
    let alerts = monitor.check_alerts();
    
    let health_status = if alerts.is_empty() {
        "healthy"
    } else if alerts.iter().any(|a| matches!(a.severity, AlertSeverity::Critical)) {
        "critical"
    } else if alerts.iter().any(|a| matches!(a.severity, AlertSeverity::Error)) {
        "degraded"
    } else {
        "warning"
    };
    
    Json(serde_json::json!({
        "success": true,
        "status": health_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime_seconds": metrics.uptime_seconds,
        "total_requests": metrics.total_requests,
        "successful_requests": metrics.successful_requests,
        "failed_requests": metrics.failed_requests,
        "active_connections": metrics.active_connections,
        "memory_usage_mb": metrics.memory_usage_mb,
        "cpu_usage_percent": metrics.cpu_usage_percent,
        "active_alerts_count": alerts.len(),
        "alerts": alerts
    }))
}

#[derive(serde::Deserialize)]
struct ClearAlertsRequest {
    pub alert_id: Option<String>,
    pub severity: Option<String>,
    pub alert_type: Option<String>,
}

async fn api_clear_alerts(
    Json(payload): Json<ClearAlertsRequest>,
) -> Json<serde_json::Value> {
    // Since SystemMonitor doesn't have specific clear methods, we'll simulate clearing
    Json(serde_json::json!({
        "success": true,
        "message": "Alert clearing endpoint (simplified)",
        "cleared_alerts": 0,
        "filter": {
            "alert_id": payload.alert_id,
            "severity": payload.severity,
            "alert_type": payload.alert_type
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}