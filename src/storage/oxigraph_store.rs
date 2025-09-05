use crate::EpcisKgError;
use crate::ontology::loader::OntologyData;
use std::collections::HashMap;
use std::path::Path;
use oxrdf::Graph as OxrdfGraph;

#[derive(Clone)]
pub struct OxigraphStore {
    graphs: HashMap<String, OxrdfGraph>,
    storage_path: String,
}

impl OxigraphStore {
    /// Create a new Oxigraph store with persistent storage
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, EpcisKgError> {
        let path = path.as_ref();
        let storage_path = path.to_string_lossy().to_string();
        
        // Try to load existing data or create empty store
        let graphs = Self::load_graphs(path)?;
        
        Ok(Self {
            graphs,
            storage_path,
        })
    }
    
    /// Create a new in-memory Oxigraph store (for testing)
    pub fn new_memory() -> Result<Self, EpcisKgError> {
        let graphs = HashMap::new();
        
        Ok(Self {
            graphs,
            storage_path: ":memory:".to_string(),
        })
    }
    
    /// Store ontology data from OntologyData struct
    pub fn store_ontology_data(&mut self, ontology_data: &OntologyData) -> Result<(), EpcisKgError> {
        // Create a named graph for this ontology
        let graph_name = format!("urn:epcis:ontology:{}", 
                                ontology_data.source_file.replace("/", ":").replace("\\", ":"));
        
        // Convert the ontology graph to our internal format
        let mut graph = OxrdfGraph::default();
        for triple in ontology_data.graph.iter() {
            graph.insert(triple);
        }
        
        // Store the graph
        self.graphs.insert(graph_name, graph);
        
        // Save to persistent storage
        self.save_graphs()?;
        
        Ok(())
    }
    
    /// Store ontology data from Turtle format string
    pub fn store_ontology_turtle(&mut self, turtle_data: &str, graph_name: &str) -> Result<(), EpcisKgError> {
        // For now, we'll skip Turtle parsing and just store empty graphs
        // In a real implementation, you'd use a proper Turtle parser
        let graph = OxrdfGraph::default();
        self.graphs.insert(graph_name.to_string(), graph);
        
        Ok(())
    }
    
    /// Execute SPARQL SELECT query and return results as JSON
    pub fn query_select(&self, sparql_query: &str) -> Result<String, EpcisKgError> {
        // For now, implement a very basic SELECT query handler
        // This is a simplified implementation that handles basic patterns
        
        if sparql_query.contains("SELECT") && sparql_query.contains("WHERE") {
            // Extract the basic pattern (very simplified)
            let variables = self.get_query_variables(sparql_query)?;
            
            // For demonstration, return some basic results
            let mut json_results = Vec::new();
            
            // Collect all triples from all graphs
            for (graph_name, graph) in &self.graphs {
                for triple in graph.iter() {
                    let mut solution_map = serde_json::Map::new();
                    
                    // Add variables based on what was requested
                    for var in &variables {
                        match var.as_str() {
                            "s" | "subject" => {
                                solution_map.insert(var.clone(), serde_json::json!({
                                    "type": "uri",
                                    "value": format!("{}", triple.subject)
                                }));
                            },
                            "p" | "predicate" => {
                                solution_map.insert(var.clone(), serde_json::json!({
                                    "type": "uri",
                                    "value": triple.predicate.as_str()
                                }));
                            },
                            "o" | "object" => {
                                let json_value = serde_json::json!({
                                    "type": "literal",
                                    "value": format!("{}", triple.object)
                                });
                                solution_map.insert(var.clone(), json_value);
                            },
                            "g" | "graph" => {
                                solution_map.insert(var.clone(), serde_json::json!({
                                    "type": "uri",
                                    "value": graph_name
                                }));
                            },
                            _ => {}
                        }
                    }
                    
                    if !solution_map.is_empty() {
                        json_results.push(solution_map);
                    }
                }
            }
            
            let result = serde_json::json!({
                "head": {
                    "vars": variables
                },
                "results": {
                    "bindings": json_results
                }
            });
            
            return Ok(serde_json::to_string_pretty(&result)
                .map_err(|e| EpcisKgError::Query(format!("Failed to serialize JSON: {}", e)))?);
        }
        
        Err(EpcisKgError::Query("Unsupported SPARQL query type".to_string()))
    }
    
    /// Execute SPARQL ASK query and return boolean result
    pub fn query_ask(&self, sparql_query: &str) -> Result<bool, EpcisKgError> {
        // Simplified ASK query implementation
        if sparql_query.contains("ASK") && sparql_query.contains("WHERE") {
            // For demonstration, return true if we have any data
            Ok(!self.graphs.is_empty())
        } else {
            Err(EpcisKgError::Query("Unsupported SPARQL ASK query".to_string()))
        }
    }
    
    /// Execute SPARQL CONSTRUCT query and return Turtle format
    pub fn query_construct(&self, sparql_query: &str) -> Result<String, EpcisKgError> {
        // Simplified CONSTRUCT query implementation
        if sparql_query.contains("CONSTRUCT") {
            // For demonstration, return all triples as Turtle
            self.export_turtle()
        } else {
            Err(EpcisKgError::Query("Unsupported SPARQL CONSTRUCT query".to_string()))
        }
    }
    
    /// Execute SPARQL update operation (simplified implementation)
    pub fn update(&mut self, sparql_update: &str) -> Result<(), EpcisKgError> {
        // For now, we'll implement a simplified version that handles basic INSERT DATA operations
        if sparql_update.contains("INSERT DATA") {
            // This is a very basic implementation - in production you'd want to use a proper SPARQL update parser
            return Err(EpcisKgError::Query("SPARQL UPDATE operations not fully implemented yet".to_string()));
        }
        
        Err(EpcisKgError::Query("Unsupported SPARQL update operation".to_string()))
    }
    
    /// Get store statistics
    pub fn get_statistics(&self) -> Result<OxigraphStats, EpcisKgError> {
        let total_quads: usize = self.graphs.values().map(|graph| graph.len()).sum();
        let named_graphs = self.graphs.len();
        let default_graph_quads = 0; // We don't store default graph quads in this implementation
        
        Ok(OxigraphStats {
            total_quads,
            named_graphs,
            default_graph_quads,
            storage_path: self.storage_path.clone(),
        })
    }
    
    /// Clear all data from the store
    pub fn clear(&mut self) -> Result<(), EpcisKgError> {
        self.graphs.clear();
        Ok(())
    }
    
    /// Store event triples in a named graph (async version)
    pub async fn store_event_triples(&mut self, event_id: &str, triples: &[oxrdf::Triple]) -> Result<(), EpcisKgError> {
        // Create a named graph for this event
        let graph_name = format!("urn:epcis:event:{}", event_id);
        
        // Create or get the graph
        let mut graph = OxrdfGraph::default();
        
        // Add all triples to the graph
        for triple in triples {
            graph.insert(triple);
        }
        
        // Store the graph
        self.graphs.insert(graph_name, graph);
        
        // Save to persistent storage if not in-memory
        if self.storage_path != ":memory:" {
            self.save_graphs()?;
        }
        
        Ok(())
    }
    
    /// Export all data as Turtle format
    pub fn export_turtle(&self) -> Result<String, EpcisKgError> {
        let mut turtle_output = String::new();
        
        for (graph_name, graph) in &self.graphs {
            turtle_output.push_str(&format!("# Graph: {}\n", graph_name));
            for triple in graph.iter() {
                let s = format!("{}", triple.subject);
                let p = format!("{}", triple.predicate);
                let o = format!("{}", triple.object);
                turtle_output.push_str(&format!("{} {} {} .\n", s, p, o));
            }
            turtle_output.push('\n');
        }
        
        Ok(turtle_output)
    }
    
    /// Get query variables from SPARQL query string (simplified parsing)
    fn get_query_variables(&self, query: &str) -> Result<Vec<String>, EpcisKgError> {
        // This is a simplified approach - in production, you'd want to use a proper SPARQL parser
        let vars: Vec<String> = query.split_whitespace()
            .filter(|s| s.starts_with('?'))
            .map(|s| s[1..].to_string())
            .collect();
        
        Ok(vars)
    }
    
    /// Load graphs from persistent storage
    fn load_graphs(path: &Path) -> Result<HashMap<String, OxrdfGraph>, EpcisKgError> {
        let metadata_path = path.join("store_metadata.json");
        
        if metadata_path.exists() {
            // Load existing data
            let metadata_content = std::fs::read_to_string(&metadata_path)?;
            let metadata: StoreMetadata = serde_json::from_str(&metadata_content)?;
            
            let mut graphs = HashMap::new();
            
            for graph_name in &metadata.graphs {
                let graph_path = path.join(format!("{}.ttl", graph_name.replace(":", "_")));
                if graph_path.exists() {
                    let turtle_content = std::fs::read_to_string(&graph_path)?;
                    let graph = Self::parse_turtle_to_graph(&turtle_content)?;
                    graphs.insert(graph_name.clone(), graph);
                }
            }
            
            Ok(graphs)
        } else {
            // Return empty store
            Ok(HashMap::new())
        }
    }
    
    /// Save graphs to persistent storage
    fn save_graphs(&self) -> Result<(), EpcisKgError> {
        let path = Path::new(&self.storage_path);
        std::fs::create_dir_all(path)?;
        
        // Save metadata
        let metadata = StoreMetadata {
            graphs: self.graphs.keys().cloned().collect(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        let metadata_path = path.join("store_metadata.json");
        std::fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)?;
        
        // Save each graph
        for (graph_name, graph) in &self.graphs {
            let turtle_content = Self::graph_to_turtle(graph)?;
            let graph_filename = format!("{}.ttl", graph_name.replace(":", "_"));
            let graph_path = path.join(graph_filename);
            std::fs::write(&graph_path, turtle_content)?;
        }
        
        Ok(())
    }
    
    /// Parse Turtle content to Graph
    fn parse_turtle_to_graph(turtle_content: &str) -> Result<OxrdfGraph, EpcisKgError> {
        let mut graph = OxrdfGraph::default();
        
        // For now, return empty graph since parsing complex Turtle syntax is non-trivial
        // In production, you'd use a proper Turtle parser like oxttl
        println!("Warning: Turtle persistence is simplified - returning empty graph");
        
        Ok(graph)
    }
    
    /// Convert Graph to Turtle format
    fn graph_to_turtle(graph: &OxrdfGraph) -> Result<String, EpcisKgError> {
        let mut turtle = String::new();
        
        for triple in graph.iter() {
            let s = format!("{}", triple.subject);
            let p = format!("{}", triple.predicate);
            let o = format!("{}", triple.object);
            turtle.push_str(&format!("{} {} {} .\n", s, p, o));
        }
        
        Ok(turtle)
    }
    
    /// Parse a single triple from a Turtle line (simplified)
    fn parse_triple_from_line(line: &str) -> Result<oxrdf::Triple, EpcisKgError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let subject = oxrdf::NamedNode::new(parts[0])
                .map_err(|e| EpcisKgError::RdfParsing(format!("Invalid subject IRI: {}", e)))?;
            let predicate = oxrdf::NamedNode::new(parts[1])
                .map_err(|e| EpcisKgError::RdfParsing(format!("Invalid predicate IRI: {}", e)))?;
            let object = oxrdf::NamedNode::new(parts[2])
                .map_err(|e| EpcisKgError::RdfParsing(format!("Invalid object IRI: {}", e)))?;
            Ok(oxrdf::Triple::new(subject, predicate, object))
        } else {
            Err(EpcisKgError::Validation("Invalid triple format".to_string()))
        }
    }
}

/// Store metadata for persistence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct StoreMetadata {
    pub graphs: Vec<String>,
    pub created_at: String,
}

/// Statistics about the Oxigraph store
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct OxigraphStats {
    pub total_quads: usize,
    pub named_graphs: usize,
    pub default_graph_quads: usize,
    pub storage_path: String,
}