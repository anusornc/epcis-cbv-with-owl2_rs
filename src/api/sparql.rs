use crate::EpcisKgError;
use crate::storage::oxigraph_store::OxigraphStore;

pub struct SparqlEndpoint {
    store: OxigraphStore,
}

impl SparqlEndpoint {
    pub fn new(store: OxigraphStore) -> Self {
        Self {
            store,
        }
    }
    
    pub async fn execute_query(&self, query: &str) -> Result<String, EpcisKgError> {
        // Determine query type and execute accordingly
        let query_upper = query.to_uppercase();
        
        if query_upper.contains("SELECT") {
            self.store.query_select(query)
        } else if query_upper.contains("ASK") {
            let result = self.store.query_ask(query)?;
            Ok(format!("{{\"boolean\": {}}}", result))
        } else if query_upper.contains("CONSTRUCT") {
            self.store.query_construct(query)
        } else {
            Err(EpcisKgError::Query("Unsupported SPARQL query type".to_string()))
        }
    }
    
    pub fn get_store_statistics(&self) -> Result<String, EpcisKgError> {
        let stats = self.store.get_statistics()?;
        Ok(format!("{{\"total_quads\": {}, \"named_graphs\": {}, \"storage_path\": \"{}\"}}", 
                   stats.total_quads, stats.named_graphs, stats.storage_path))
    }
}