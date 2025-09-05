use crate::EpcisKgError;
use crate::storage::oxigraph_store::OxigraphStore;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct SparqlEndpoint {
    store: Arc<OxigraphStore>,
}

#[derive(Deserialize)]
pub struct SparqlQuery {
    pub query: String,
    pub format: Option<String>,
}

#[derive(Serialize)]
pub struct SparqlResponse {
    pub results: String,
    pub query_type: String,
    pub execution_time_ms: u64,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl SparqlEndpoint {
    pub fn new(store: OxigraphStore) -> Self {
        Self {
            store: Arc::new(store),
        }
    }
    
    pub fn router(&self) -> Router {
        Router::new()
            .route("/sparql", get(sparql_get).post(sparql_post))
            .route("/sparql/query", post(execute_sparql))
            .route("/statistics", get(get_statistics))
            .route("/health", get(health_check))
            .with_state(self.clone())
    }
    
    pub async fn execute_query(&self, query: &str) -> Result<String, EpcisKgError> {
        let start_time = std::time::Instant::now();
        
        // Determine query type and execute accordingly
        let query_upper = query.to_uppercase();
        let result = if query_upper.contains("SELECT") {
            self.store.query_select(query)?
        } else if query_upper.contains("ASK") {
            let result = self.store.query_ask(query)?;
            format!("{{\"boolean\": {}}}", result)
        } else if query_upper.contains("CONSTRUCT") {
            self.store.query_construct(query)?
        } else {
            return Err(EpcisKgError::Query("Unsupported SPARQL query type".to_string()));
        };
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        Ok(result)
    }
    
    pub fn get_store_statistics(&self) -> Result<String, EpcisKgError> {
        let stats = self.store.get_statistics()?;
        Ok(serde_json::to_string_pretty(&stats)?)
    }
}

// Clone implementation for Axum state
impl Clone for SparqlEndpoint {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
        }
    }
}

// Public HTTP Handlers
pub async fn sparql_get(
    State(endpoint): State<SparqlEndpoint>,
    Query(params): Query<SparqlQuery>,
) -> Result<Json<SparqlResponse>, ErrorResponse> {
    let start_time = std::time::Instant::now();
    
    match endpoint.execute_query(&params.query).await {
        Ok(results) => {
            let query_type = determine_query_type(&params.query);
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            Ok(Json(SparqlResponse {
                results,
                query_type,
                execution_time_ms: execution_time,
            }))
        }
        Err(e) => {
            Err(ErrorResponse {
                error: "Query execution failed".to_string(),
                message: e.to_string(),
            })
        }
    }
}

pub async fn sparql_post(
    State(endpoint): State<SparqlEndpoint>,
    Json(payload): Json<SparqlQuery>,
) -> Result<Json<SparqlResponse>, ErrorResponse> {
    sparql_get(State(endpoint), Query(payload)).await
}

pub async fn execute_sparql(
    State(endpoint): State<SparqlEndpoint>,
    Json(payload): Json<SparqlQuery>,
) -> Result<Response, ErrorResponse> {
    match endpoint.execute_query(&payload.query).await {
        Ok(results) => {
            let query_type = determine_query_type(&payload.query);
            
            // Return different content types based on format parameter
            match payload.format.as_deref() {
                Some("json") | None => {
                    let response = SparqlResponse {
                        results,
                        query_type,
                        execution_time_ms: 0,
                    };
                    Ok(Json(response).into_response())
                }
                Some("csv") => {
                    Ok((
                        StatusCode::OK,
                        [("content-type", "text/csv")],
                        results,
                    ).into_response())
                }
                Some("xml") => {
                    Ok((
                        StatusCode::OK,
                        [("content-type", "application/xml")],
                        format!("<?xml version=\"1.0\"?><results>{}</results>", results),
                    ).into_response())
                }
                _ => {
                    Err(ErrorResponse {
                        error: "Unsupported format".to_string(),
                        message: "Supported formats: json, csv, xml".to_string(),
                    })
                }
            }
        }
        Err(e) => {
            Err(ErrorResponse {
                error: "Query execution failed".to_string(),
                message: e.to_string(),
            })
        }
    }
}

pub async fn get_statistics(
    State(endpoint): State<SparqlEndpoint>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    match endpoint.get_store_statistics() {
        Ok(stats_json) => {
            let stats: serde_json::Value = serde_json::from_str(&stats_json)
                .map_err(|e| ErrorResponse {
                    error: "Failed to parse statistics".to_string(),
                    message: e.to_string(),
                })?;
            Ok(Json(stats))
        }
        Err(e) => {
            Err(ErrorResponse {
                error: "Failed to get statistics".to_string(),
                message: e.to_string(),
            })
        }
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "epcis-knowledge-graph",
        "version": "0.1.0"
    }))
}

// Helper function to determine query type
fn determine_query_type(query: &str) -> String {
    let query_upper = query.to_uppercase();
    if query_upper.contains("SELECT") {
        "SELECT".to_string()
    } else if query_upper.contains("ASK") {
        "ASK".to_string()
    } else if query_upper.contains("CONSTRUCT") {
        "CONSTRUCT".to_string()
    } else if query_upper.contains("INSERT") || query_upper.contains("DELETE") {
        "UPDATE".to_string()
    } else {
        "UNKNOWN".to_string()
    }
}

// Error response implementation
impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let body = Json(self);
        
        (status, body).into_response()
    }
}