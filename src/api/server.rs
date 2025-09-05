use crate::config::AppConfig;
use crate::storage::oxigraph_store::OxigraphStore;
use axum::{
    extract::Query,
    response::{Json, Response, IntoResponse},
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing::info;

pub struct WebServer {
    config: Arc<AppConfig>,
    store: Arc<Mutex<OxigraphStore>>,
}

impl WebServer {
    pub fn new(config: AppConfig, store: OxigraphStore) -> Self {
        Self {
            config: Arc::new(config),
            store: Arc::new(Mutex::new(store)),
        }
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
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        
        axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;
        
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
            .route("/sparql", get(api_sparql_get).post(api_sparql_post))
            .route("/sparql/query", post(api_sparql_execute))
            .route("/statistics", get(api_statistics))
            .route("/ontologies", get(api_list_ontologies).post(api_load_ontology))
            .route("/config", get(api_config))
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

// API Handlers
async fn api_sparql_get(
    Query(params): Query<crate::api::sparql::SparqlQuery>,
) -> Result<Json<crate::api::sparql::SparqlResponse>, crate::api::sparql::ErrorResponse> {
    // Create a simple response for now
    Ok(Json(crate::api::sparql::SparqlResponse {
        results: "Web server running - SPARQL endpoint not fully implemented".to_string(),
        query_type: "SELECT".to_string(),
        execution_time_ms: 0,
    }))
}

async fn api_sparql_post(
    Json(payload): Json<crate::api::sparql::SparqlQuery>,
) -> Result<Json<crate::api::sparql::SparqlResponse>, crate::api::sparql::ErrorResponse> {
    // Create a simple response for now
    Ok(Json(crate::api::sparql::SparqlResponse {
        results: "Web server running - SPARQL endpoint not fully implemented".to_string(),
        query_type: "SELECT".to_string(),
        execution_time_ms: 0,
    }))
}

async fn api_sparql_execute(
    Json(payload): Json<crate::api::sparql::SparqlQuery>,
) -> Result<Response, crate::api::sparql::ErrorResponse> {
    // Create a simple response for now
    let response = crate::api::sparql::SparqlResponse {
        results: "Web server running - SPARQL endpoint not fully implemented".to_string(),
        query_type: "SELECT".to_string(),
        execution_time_ms: 0,
    };
    Ok(Json(response).into_response())
}

async fn api_statistics(
) -> Result<Json<serde_json::Value>, crate::api::sparql::ErrorResponse> {
    Ok(Json(serde_json::json!({
        "status": "web_server_running",
        "message": "Web server is running - statistics not fully implemented",
        "version": "0.1.0"
    })))
}

async fn api_list_ontologies(
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ontologies": ["ontologies/epcis2.ttl", "ontologies/cbv.ttl"],
        "loaded_graphs": 0,
        "total_triples": 0,
        "status": "web_server_running"
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
        "message": "Web server running - ontology loading not fully implemented",
        "file": payload.file_path,
        "triples_loaded": 0,
        "total_triples": 0,
        "named_graphs": 0
    })))
}

async fn api_config(
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "web_server_running",
        "version": "0.1.0",
        "message": "Web server is running - configuration not fully implemented"
    }))
}