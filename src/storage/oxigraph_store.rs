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
        let mut triple_count = 0;
        
        println!("🔍 DEBUG: Storing {} triples from {}", ontology_data.triples_count, ontology_data.source_file);
        
        for triple in ontology_data.graph.iter() {
            graph.insert(triple);
            triple_count += 1;
            
            // Print first few triples for debugging
            if triple_count <= 5 {
                println!("🔍 DEBUG: Triple {}: {} -> {} -> {}", 
                    triple_count, triple.subject, triple.predicate, triple.object);
            }
        }
        
        println!("🔍 DEBUG: Total triples stored: {}", triple_count);
        
        // Store the graph
        self.graphs.insert(graph_name, graph);
        
        // Save to persistent storage
        self.save_graphs()?;
        
        Ok(())
    }
    
    /// Store ontology data from Turtle format string
    pub fn store_ontology_turtle(&mut self, turtle_data: &str, graph_name: &str) -> Result<(), EpcisKgError> {
        let mut graph = OxrdfGraph::default();
        
        // Parse prefixes from Turtle data
        let mut prefixes = std::collections::HashMap::new();
        let mut triple_count = 0;
        
        // Simple Turtle parser - extract real triples
        for line in turtle_data.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            // Parse prefix declarations
            if trimmed.starts_with("@prefix") {
                if let Some(prefix_part) = trimmed.strip_prefix("@prefix") {
                    let parts: Vec<&str> = prefix_part.split(':').collect();
                    if parts.len() >= 2 {
                        let prefix_name = parts[0].trim();
                        let uri_part = parts[1].trim();
                        if let Some(uri) = uri_part.strip_suffix('.').and_then(|s| s.strip_prefix('<')).and_then(|s| s.strip_suffix('>')) {
                            prefixes.insert(prefix_name, uri.to_string());
                        }
                    }
                }
                continue;
            }
            
            // Parse triples (simplified Turtle parsing)
            if trimmed.contains(' ') && !trimmed.starts_with('@') {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 3 {
                    let subject_str = parts[0];
                    let predicate_str = parts[1];
                    let mut object_str = parts[2];
                    
                    // Remove trailing dot from object if present
                    if object_str.ends_with('.') {
                        object_str = &object_str[0..object_str.len()-1];
                    }
                    
                    // Convert subject
                    let subject = if subject_str.starts_with('<') && subject_str.ends_with('>') {
                        let uri = &subject_str[1..subject_str.len()-1];
                        oxrdf::NamedNode::new_unchecked(uri)
                    } else if subject_str.contains(':') {
                        // Handle prefixed names
                        let mut expanded = subject_str.to_string();
                        for (prefix, uri) in &prefixes {
                            if subject_str.starts_with(&format!("{}:", prefix)) {
                                expanded = subject_str.replace(&format!("{}:", prefix), uri);
                                break;
                            }
                        }
                        oxrdf::NamedNode::new_unchecked(expanded)
                    } else {
                        continue; // Skip invalid subjects
                    };
                    
                    // Convert predicate
                    let predicate = if predicate_str.starts_with('<') && predicate_str.ends_with('>') {
                        let uri = &predicate_str[1..predicate_str.len()-1];
                        oxrdf::NamedNode::new_unchecked(uri)
                    } else if predicate_str.contains(':') {
                        let mut expanded = predicate_str.to_string();
                        for (prefix, uri) in &prefixes {
                            if predicate_str.starts_with(&format!("{}:", prefix)) {
                                expanded = predicate_str.replace(&format!("{}:", prefix), uri);
                                break;
                            }
                        }
                        oxrdf::NamedNode::new_unchecked(expanded)
                    } else {
                        continue; // Skip invalid predicates
                    };
                    
                    // Convert object
                    let object = if object_str.starts_with('<') && object_str.ends_with('>') {
                        let uri = &object_str[1..object_str.len()-1];
                        oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(uri))
                    } else if object_str.starts_with('"') && object_str.ends_with('"') {
                        // Literal
                        let literal_content = &object_str[1..object_str.len()-1];
                        oxrdf::Term::Literal(oxrdf::Literal::new_simple_literal(literal_content))
                    } else if object_str.contains(':') {
                        // Prefixed name or URI
                        let mut expanded = object_str.to_string();
                        for (prefix, uri) in &prefixes {
                            if object_str.starts_with(&format!("{}:", prefix)) {
                                expanded = object_str.replace(&format!("{}:", prefix), uri);
                                break;
                            }
                        }
                        oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(expanded))
                    } else {
                        continue; // Skip invalid objects
                    };
                    
                    // Create and store the triple
                    let triple = oxrdf::Triple::new(subject, predicate, object);
                    graph.insert(triple.as_ref());
                    triple_count += 1;
                }
            }
        }
        
        println!("✓ Parsed and stored {} real triples from Turtle data for graph: {}", triple_count, graph_name);
        
        // Store the graph
        self.graphs.insert(graph_name.to_string(), graph);
        
        Ok(())
    }
    
    /// Execute SPARQL SELECT query and return results as JSON
    pub fn query_select(&self, sparql_query: &str) -> Result<String, EpcisKgError> {
        println!("🔍 DEBUG: Executing SPARQL query: {}", sparql_query);
        println!("🔍 DEBUG: Available graphs: {}", self.graphs.len());
        
        // For now, implement a very basic SELECT query handler
        // This is a simplified implementation that handles basic patterns
        
        if sparql_query.contains("SELECT") && sparql_query.contains("WHERE") {
            // Extract the basic pattern (very simplified)
            let variables = self.get_query_variables(sparql_query)?;
            println!("🔍 DEBUG: Query variables: {:?}", variables);
            
            // Parse LIMIT clause if present
            let limit = self.parse_limit_clause(sparql_query)?;
            println!("🔍 DEBUG: Query LIMIT: {}", limit);
            
            // For demonstration, return some basic results
            let mut json_results = Vec::new();
            let mut total_triples = 0;
            
            // Collect all triples from all graphs
            for (graph_name, graph) in &self.graphs {
                println!("🔍 DEBUG: Graph '{}' has {} triples", graph_name, graph.len());
                for triple in graph.iter() {
                    total_triples += 1;
                    let mut solution_map = serde_json::Map::new();
                    
                    // Only add solutions if the triple matches the query pattern
                    // This is a simplified pattern matching - in production you'd want full SPARQL parsing
                    let mut matches_pattern = false;
                    
                    // For queries with specific predicates, check if this triple matches
                    if variables.contains(&"name".to_string()) {
                        matches_pattern = matches_pattern || 
                            (triple.predicate.as_str().contains("name") || 
                             triple.predicate.as_str().contains("label"));
                    }
                    if variables.contains(&"eventTime".to_string()) {
                        matches_pattern = matches_pattern || 
                            triple.predicate.as_str().contains("eventTime");
                    }
                    if variables.contains(&"bizLocation".to_string()) {
                        matches_pattern = matches_pattern || 
                            (triple.predicate.as_str().contains("bizLocation") ||
                             triple.predicate.as_str().contains("location"));
                    }
                    if variables.contains(&"disposition".to_string()) {
                        matches_pattern = matches_pattern || 
                            triple.predicate.as_str().contains("disposition");
                    }
                    if variables.contains(&"quantity".to_string()) {
                        matches_pattern = matches_pattern || 
                            triple.predicate.as_str().contains("quantity");
                    }
                    if variables.contains(&"entityType".to_string()) {
                        matches_pattern = matches_pattern || 
                            triple.predicate.as_str().contains("entityType");
                    }
                    
                    // For basic queries (s, p, o), match all triples
                    if variables.iter().any(|v| v == "s" || v == "p" || v == "o") {
                        matches_pattern = true;
                    }
                    
                    // Only process this triple if it matches the pattern
                    if matches_pattern {
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
                                // Handle common SPARQL variable patterns by matching to triple components
                                _ => {
                                    // For arbitrary variable names, try to match them to triple patterns
                                    // This is a simplified approach - in production you'd want full SPARQL parsing
                                    if var.contains("name") || var.contains("label") {
                                        // Look for name or label in the predicate
                                        if triple.predicate.as_str().contains("name") || 
                                           triple.predicate.as_str().contains("label") {
                                            solution_map.insert(var.clone(), serde_json::json!({
                                                "type": "literal",
                                                "value": format!("{}", triple.object)
                                            }));
                                        }
                                    } else if var.contains("time") || var.contains("date") {
                                        // Look for time-related predicates
                                        if triple.predicate.as_str().contains("time") || 
                                           triple.predicate.as_str().contains("date") {
                                            solution_map.insert(var.clone(), serde_json::json!({
                                                "type": "literal",
                                                "value": format!("{}", triple.object)
                                            }));
                                        }
                                    } else if var.contains("location") || var.contains("loc") {
                                        // Look for location-related predicates
                                        if triple.predicate.as_str().contains("location") || 
                                           triple.predicate.as_str().contains("bizLocation") {
                                            solution_map.insert(var.clone(), serde_json::json!({
                                                "type": "literal",
                                                "value": format!("{}", triple.object)
                                            }));
                                        }
                                    } else if var.contains("event") && !var.contains("Time") {
                                        // For event variables (not eventTime), use the subject
                                        solution_map.insert(var.clone(), serde_json::json!({
                                            "type": "uri",
                                            "value": format!("{}", triple.subject)
                                        }));
                                    } else {
                                        // For other variables, just include the object value
                                        solution_map.insert(var.clone(), serde_json::json!({
                                            "type": "literal",
                                            "value": format!("{}", triple.object)
                                        }));
                                    }
                                }
                            }
                        }
                    }
                    
                    if !solution_map.is_empty() {
                        json_results.push(solution_map);
                    }
                    
                    // Apply limit if specified
                    if limit > 0 && json_results.len() >= limit {
                        break;
                    }
                }
                
                // Apply limit if specified (break out of graph loop)
                if limit > 0 && json_results.len() >= limit {
                    break;
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
            graph.insert(triple.as_ref());
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
        // Extract variables from the SELECT clause more accurately
        let query_upper = query.to_uppercase();
        let select_start = query_upper.find("SELECT").ok_or_else(|| {
            EpcisKgError::Query("No SELECT clause found in query".to_string())
        })?;
        
        // Find the WHERE clause or end of SELECT variables
        let where_pos = query_upper.find("WHERE").unwrap_or(query.len());
        let select_clause = &query[select_start + 6..where_pos].trim();
        
        // Parse variables from SELECT clause
        let mut vars = Vec::new();
        let mut in_distinct = false;
        
        for token in select_clause.split_whitespace() {
            let token_upper = token.to_uppercase();
            
            if token_upper == "DISTINCT" {
                in_distinct = true;
                continue;
            }
            
            if token_upper == "REDUCED" || token_upper == "*" {
                // Handle REDUCED or wildcard - return all common variables
                return Ok(vec!["s".to_string(), "p".to_string(), "o".to_string()]);
            }
            
            if token.starts_with('?') {
                let var_name = token[1..].to_string();
                if !vars.contains(&var_name) {
                    vars.push(var_name);
                }
            }
        }
        
        // If no variables found in SELECT clause, default to s, p, o
        if vars.is_empty() {
            vars = vec!["s".to_string(), "p".to_string(), "o".to_string()];
        }
        
        Ok(vars)
    }
    
    /// Parse LIMIT clause from SPARQL query string
    fn parse_limit_clause(&self, query: &str) -> Result<usize, EpcisKgError> {
        // Look for LIMIT clause in the query
        let query_upper = query.to_uppercase();
        if let Some(limit_pos) = query_upper.find("LIMIT") {
            // Get the part after LIMIT
            let after_limit = &query[limit_pos + 5..];
            // Extract the number (simplified - just take the first sequence of digits)
            let limit_str = after_limit.trim().split_whitespace().next().unwrap_or("0");
            // Parse the number
            limit_str.parse::<usize>().map_err(|_| {
                EpcisKgError::Query(format!("Invalid LIMIT value: {}", limit_str))
            })
        } else {
            // No LIMIT specified, return 0 (unlimited)
            Ok(0)
        }
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
        let mut triple_count = 0;
        
        // Parse prefixes from Turtle data
        let mut prefixes = std::collections::HashMap::new();
        
        // Simple Turtle parser - extract real triples
        for line in turtle_content.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            // Parse prefix declarations
            if trimmed.starts_with("@prefix") {
                if let Some(prefix_part) = trimmed.strip_prefix("@prefix") {
                    let parts: Vec<&str> = prefix_part.split(':').collect();
                    if parts.len() >= 2 {
                        let prefix_name = parts[0].trim();
                        let uri_part = parts[1].trim();
                        if let Some(uri) = uri_part.strip_suffix('.').and_then(|s| s.strip_prefix('<')).and_then(|s| s.strip_suffix('>')) {
                            prefixes.insert(prefix_name, uri.to_string());
                        }
                    }
                }
                continue;
            }
            
            // Parse triples (simplified Turtle parsing)
            if trimmed.contains(' ') && !trimmed.starts_with('@') {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 3 {
                    let subject_str = parts[0];
                    let predicate_str = parts[1];
                    let mut object_str = parts[2];
                    
                    // Remove trailing dot from object if present
                    if object_str.ends_with('.') {
                        object_str = &object_str[0..object_str.len()-1];
                    }
                    
                    // Convert subject
                    let subject = if subject_str.starts_with('<') && subject_str.ends_with('>') {
                        let uri = &subject_str[1..subject_str.len()-1];
                        oxrdf::NamedNode::new_unchecked(uri)
                    } else if subject_str.contains(':') {
                        // Handle prefixed names
                        let mut expanded = subject_str.to_string();
                        for (prefix, uri) in &prefixes {
                            if subject_str.starts_with(&format!("{}:", prefix)) {
                                expanded = subject_str.replace(&format!("{}:", prefix), uri);
                                break;
                            }
                        }
                        oxrdf::NamedNode::new_unchecked(expanded)
                    } else {
                        continue; // Skip invalid subjects
                    };
                    
                    // Convert predicate
                    let predicate = if predicate_str.starts_with('<') && predicate_str.ends_with('>') {
                        let uri = &predicate_str[1..predicate_str.len()-1];
                        oxrdf::NamedNode::new_unchecked(uri)
                    } else if predicate_str.contains(':') {
                        let mut expanded = predicate_str.to_string();
                        for (prefix, uri) in &prefixes {
                            if predicate_str.starts_with(&format!("{}:", prefix)) {
                                expanded = predicate_str.replace(&format!("{}:", prefix), uri);
                                break;
                            }
                        }
                        oxrdf::NamedNode::new_unchecked(expanded)
                    } else {
                        continue; // Skip invalid predicates
                    };
                    
                    // Convert object
                    let object = if object_str.starts_with('<') && object_str.ends_with('>') {
                        let uri = &object_str[1..object_str.len()-1];
                        oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(uri))
                    } else if object_str.starts_with('"') && object_str.ends_with('"') {
                        // Literal
                        let literal_content = &object_str[1..object_str.len()-1];
                        oxrdf::Term::Literal(oxrdf::Literal::new_simple_literal(literal_content))
                    } else if object_str.contains(':') {
                        // Prefixed name or URI
                        let mut expanded = object_str.to_string();
                        for (prefix, uri) in &prefixes {
                            if object_str.starts_with(&format!("{}:", prefix)) {
                                expanded = object_str.replace(&format!("{}:", prefix), uri);
                                break;
                            }
                        }
                        oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(expanded))
                    } else {
                        continue; // Skip invalid objects
                    };
                    
                    // Create and store the triple
                    let triple = oxrdf::Triple::new(subject, predicate, object);
                    graph.insert(triple.as_ref());
                    triple_count += 1;
                }
            }
        }
        
        println!("✓ Parsed and stored {} triples from persisted Turtle data", triple_count);
        
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