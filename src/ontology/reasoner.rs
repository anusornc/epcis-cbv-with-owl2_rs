use crate::EpcisKgError;
use crate::Config;
use crate::storage::oxigraph_store::OxigraphStore;
use crate::ontology::loader::OntologyData;

pub struct OntologyReasoner {
    config: Config,
    store: Option<OxigraphStore>,
}

impl OntologyReasoner {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            store: None,
        }
    }
    
    pub fn with_store(store: OxigraphStore) -> Self {
        Self {
            config: Config::default(),
            store: Some(store),
        }
    }
    
    pub fn with_config(config: &Config) -> Self {
        Self {
            config: config.clone(),
            store: None,
        }
    }
    
    /// Basic ontology validation using stored data
    pub fn validate_ontology(&self, ontology_data: &OntologyData) -> Result<(), EpcisKgError> {
        // Basic validation checks
        if ontology_data.triples_count == 0 {
            return Err(EpcisKgError::Validation("Ontology contains no triples".to_string()));
        }
        
        // Check for required RDF and RDFS vocabulary
        let rdf_type = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        let rdfs_class = "http://www.w3.org/2000/01/rdf-schema#Class";
        
        let mut has_classes = false;
        let mut has_type_statements = false;
        
        for triple in ontology_data.graph.iter() {
            let predicate_str = format!("{}", triple.predicate);
            let object_str = format!("{}", triple.object);
            
            // Check for rdf:type (both full URI and prefixed form)
            if predicate_str == rdf_type || predicate_str.contains("#type") || predicate_str.ends_with("type") {
                has_type_statements = true;
                if object_str == rdfs_class || object_str.contains("Class") {
                    has_classes = true;
                }
            }
        }
        
        if !has_type_statements {
            return Err(EpcisKgError::Validation("Ontology missing rdf:type statements".to_string()));
        }
        
        if !has_classes {
            return Err(EpcisKgError::Validation("Ontology missing class definitions".to_string()));
        }
        
        Ok(())
    }
    
    /// Perform basic inference using stored ontologies
    pub fn perform_inference(&self) -> Result<Vec<String>, EpcisKgError> {
        let mut inferred_triples = Vec::new();
        
        if let Some(ref store) = self.store {
            // Basic inference: find all subclasses and infer superclass relationships
            let subclass_query = r#"
                SELECT ?subclass ?superclass
                WHERE {
                    ?subclass <http://www.w3.org/2000/01/rdf-schema#subClassOf> ?superclass .
                }
            "#;
            
            if let Ok(results) = store.query_select(subclass_query) {
                // Parse results and generate inferences
                // This is a simplified version - in practice you'd use proper SPARQL results parsing
                inferred_triples.push(format!("Found subclass relationships: {}", results.len()));
            }
            
            // Basic type inference
            let type_query = r#"
                SELECT ?instance ?class
                WHERE {
                    ?instance a ?class .
                }
                LIMIT 10
            "#;
            
            if let Ok(results) = store.query_select(type_query) {
                inferred_triples.push(format!("Found type instances: {}", results.len()));
            }
            
            // Domain/range inference
            let domain_query = r#"
                SELECT ?property ?domain
                WHERE {
                    ?property <http://www.w3.org/2000/01/rdf-schema#domain> ?domain .
                }
                LIMIT 5
            "#;
            
            if let Ok(results) = store.query_select(domain_query) {
                inferred_triples.push(format!("Found property domains: {}", results.len()));
            }
        } else {
            return Err(EpcisKgError::Query("No store available for inference".to_string()));
        }
        
        Ok(inferred_triples)
    }
    
    /// Check if ontology conforms to OWL profile (simplified)
    pub fn check_owl_profile(&self, ontology_data: &OntologyData, profile: &str) -> Result<(), EpcisKgError> {
        match profile.to_lowercase().as_str() {
            "el" | "owl2el" => {
                // OWL 2 EL Profile check (simplified)
                self.check_owl_el_profile(ontology_data)?;
            },
            "ql" | "owl2ql" => {
                // OWL 2 QL Profile check (simplified)
                self.check_owl_ql_profile(ontology_data)?;
            },
            "rl" | "owl2rl" => {
                // OWL 2 RL Profile check (simplified)
                self.check_owl_rl_profile(ontology_data)?;
            },
            _ => {
                return Err(EpcisKgError::Validation(format!("Unknown OWL profile: {}", profile)));
            }
        }
        
        Ok(())
    }
    
    /// Check OWL 2 EL profile compliance (simplified)
    fn check_owl_el_profile(&self, ontology_data: &OntologyData) -> Result<(), EpcisKgError> {
        // EL profile allows: subclass, equivalence, property restrictions, etc.
        // Check for unsupported constructs
        let unsupported_constructs = vec![
            "http://www.w3.org/2002/07/owl#unionOf",
            "http://www.w3.org/2002/07/owl#intersectionOf",
            "http://www.w3.org/2002/07/owl#complementOf",
            "http://www.w3.org/2002/07/owl#oneOf",
            "http://www.w3.org/2002/07/owl#disjointWith",
        ];
        
        for triple in ontology_data.graph.iter() {
            let predicate_str = format!("{}", triple.predicate);
            if unsupported_constructs.contains(&predicate_str.as_str()) {
                return Err(EpcisKgError::Validation(format!(
                    "OWL 2 EL profile violation: found unsupported construct {}",
                    predicate_str
                )));
            }
        }
        
        Ok(())
    }
    
    /// Check OWL 2 QL profile compliance (simplified)
    fn check_owl_ql_profile(&self, ontology_data: &OntologyData) -> Result<(), EpcisKgError> {
        // QL profile is designed for query rewriting
        // More restrictive than EL
        self.check_owl_el_profile(ontology_data)?; // Start with EL checks
        
        // Additional QL restrictions could be added here
        
        Ok(())
    }
    
    /// Check OWL 2 RL profile compliance (simplified)
    fn check_owl_rl_profile(&self, ontology_data: &OntologyData) -> Result<(), EpcisKgError> {
        // RL profile is designed for rule-based reasoning
        // Less restrictive than EL and QL
        
        // Basic checks for rule safety
        let mut complex_constructs = 0;
        let restricted_constructs = vec![
            "http://www.w3.org/2002/07/owl#unionOf",
            "http://www.w3.org/2002/07/owl#intersectionOf",
        ];
        
        for triple in ontology_data.graph.iter() {
            let predicate_str = format!("{}", triple.predicate);
            if restricted_constructs.contains(&predicate_str.as_str()) {
                complex_constructs += 1;
            }
        }
        
        if complex_constructs > 10 { // Arbitrary threshold
            return Err(EpcisKgError::Validation(format!(
                "OWL 2 RL profile warning: found {} complex constructs, may impact reasoning performance",
                complex_constructs
            )));
        }
        
        Ok(())
    }
    
    /// Get reasoning statistics
    pub fn get_reasoning_stats(&self) -> Result<String, EpcisKgError> {
        if let Some(ref store) = self.store {
            let stats = store.get_statistics()?;
            Ok(format!("{{\"total_triples\": {}, \"named_graphs\": {}, \"reasoning_ready\": true}}", 
                       stats.total_quads, stats.named_graphs))
        } else {
            Ok("{\"reasoning_ready\": false, \"reason\": \"No store available\"}".to_string())
        }
    }
}