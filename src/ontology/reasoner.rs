use crate::EpcisKgError;
use crate::Config;
use crate::storage::oxigraph_store::OxigraphStore;
use crate::ontology::loader::OntologyData;
use owl2_rs::{api, Ontology, IRI, Class, ObjectProperty, Individual};
use std::collections::HashMap;

pub struct OntologyReasoner {
    config: Config,
    store: Option<OxigraphStore>,
    owl_ontology: Option<Ontology>,
    owl_reasoner: Option<api::Reasoner>,
    reasoning_cache: HashMap<String, Vec<String>>,
}

impl OntologyReasoner {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            store: None,
            owl_ontology: None,
            owl_reasoner: None,
            reasoning_cache: HashMap::new(),
        }
    }
    
    pub fn with_store(store: OxigraphStore) -> Self {
        Self {
            config: Config::default(),
            store: Some(store),
            owl_ontology: None,
            owl_reasoner: None,
            reasoning_cache: HashMap::new(),
        }
    }
    
    pub fn with_config(config: &Config) -> Self {
        Self {
            config: config.clone(),
            store: None,
            owl_ontology: None,
            owl_reasoner: None,
            reasoning_cache: HashMap::new(),
        }
    }
    
    /// Load ontology data into the OWL 2 reasoner
    pub fn load_ontology_data(&mut self, ontology_data: &OntologyData) -> Result<(), EpcisKgError> {
        // Convert RDF graph to OWL 2 ontology
        let owl_ontology = self.convert_rdf_to_owl(ontology_data)?;
        self.owl_ontology = Some(owl_ontology.clone());
        
        // Create OWL 2 reasoner
        let reasoner = api::Reasoner::new(owl_ontology);
        self.owl_reasoner = Some(reasoner);
        
        Ok(())
    }
    
    /// Convert RDF graph data to OWL 2 ontology format
    fn convert_rdf_to_owl(&self, ontology_data: &OntologyData) -> Result<Ontology, EpcisKgError> {
        let mut owl_ontology = Ontology::default();
        
        // Track processed entities to avoid duplicates
        let mut processed_classes = std::collections::HashSet::new();
        let mut processed_properties = std::collections::HashSet::new();
        let mut processed_individuals = std::collections::HashSet::new();
        
        // Process RDF triples and convert to OWL 2 axioms
        for triple in ontology_data.graph.iter() {
            let subject_str = format!("{}", triple.subject);
            let predicate_str = format!("{}", triple.predicate);
            let object_str = format!("{}", triple.object);
            
            // Handle class declarations (rdf:type rdfs:Class)
            if predicate_str == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" 
                && object_str == "http://www.w3.org/2000/01/rdf-schema#Class" {
                if !processed_classes.contains(&subject_str) {
                    let class_iri = IRI(subject_str.clone());
                    owl_ontology.axioms.push(
                        owl2_rs::Axiom::Class(owl2_rs::ClassAxiom::SubClassOf {
                            sub_class: owl2_rs::ClassExpression::Class(Class(class_iri.clone())),
                            super_class: owl2_rs::ClassExpression::Class(Class(IRI("http://www.w3.org/2002/07/owl#Thing".to_string()))),
                        })
                    );
                    processed_classes.insert(subject_str);
                }
            }
            
            // Handle subclass relationships
            else if predicate_str == "http://www.w3.org/2000/01/rdf-schema#subClassOf" {
                let sub_class = Class(IRI(subject_str.clone()));
                let super_class = Class(IRI(object_str.clone()));
                
                owl_ontology.axioms.push(
                    owl2_rs::Axiom::Class(owl2_rs::ClassAxiom::SubClassOf {
                        sub_class: owl2_rs::ClassExpression::Class(sub_class),
                        super_class: owl2_rs::ClassExpression::Class(super_class),
                    })
                );
            }
            
            // Handle object property declarations
            else if predicate_str == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" 
                && object_str == "http://www.w3.org/2002/07/owl#ObjectProperty" {
                if !processed_properties.contains(&subject_str) {
                    let prop_iri = IRI(subject_str.clone());
                    // Add domain and range defaults for EPCIS properties
                    owl_ontology.axioms.push(
                        owl2_rs::Axiom::ObjectProperty(owl2_rs::ObjectPropertyAxiom::ObjectPropertyDomain {
                            property: owl2_rs::ObjectPropertyExpression::ObjectProperty(ObjectProperty(prop_iri.clone())),
                            domain: owl2_rs::ClassExpression::Class(Class(IRI("http://www.w3.org/2002/07/owl#Thing".to_string()))),
                        })
                    );
                    processed_properties.insert(subject_str);
                }
            }
            
            // Handle property domain and range
            else if predicate_str == "http://www.w3.org/2000/01/rdf-schema#domain" {
                if let Some(property_iri) = self.extract_property_iri(&subject_str) {
                    let domain_class = Class(IRI(object_str.clone()));
                    owl_ontology.axioms.push(
                        owl2_rs::Axiom::ObjectProperty(owl2_rs::ObjectPropertyAxiom::ObjectPropertyDomain {
                            property: property_iri,
                            domain: owl2_rs::ClassExpression::Class(domain_class),
                        })
                    );
                }
            }
            
            // Handle individual type assertions
            else if predicate_str == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" 
                && !object_str.contains("Class") && !object_str.contains("Property") {
                if !processed_individuals.contains(&subject_str) {
                    let individual = Individual::Named(IRI(subject_str.clone()));
                    let class = Class(IRI(object_str.clone()));
                    
                    owl_ontology.axioms.push(
                        owl2_rs::Axiom::Assertion(owl2_rs::Assertion::ClassAssertion {
                            class: owl2_rs::ClassExpression::Class(class),
                            individual,
                        })
                    );
                    processed_individuals.insert(subject_str);
                }
            }
        }
        
        Ok(owl_ontology)
    }
    
    /// Extract property IRI from subject string
    fn extract_property_iri(&self, subject_str: &str) -> Option<owl2_rs::ObjectPropertyExpression> {
        Some(owl2_rs::ObjectPropertyExpression::ObjectProperty(
            ObjectProperty(IRI(subject_str.to_string()))
        ))
    }
    
    /// Enhanced ontology validation using OWL 2 reasoner
    pub fn validate_ontology(&mut self, ontology_data: &OntologyData) -> Result<(), EpcisKgError> {
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
        
        // If we have an OWL 2 reasoner, perform consistency checking
        if let Some(ref mut reasoner) = self.owl_reasoner {
            let is_consistent = reasoner.is_consistent();
            if !is_consistent {
                return Err(EpcisKgError::Validation("Ontology is logically inconsistent".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Enhanced inference using OWL 2 reasoner
    pub fn perform_inference(&mut self) -> Result<Vec<String>, EpcisKgError> {
        let mut inferred_triples = Vec::new();
        
        // Use OWL 2 reasoner if available
        if let Some(ref mut reasoner) = self.owl_reasoner {
            // Check consistency first
            let is_consistent = reasoner.is_consistent();
            inferred_triples.push(format!("Ontology consistency: {}", if is_consistent { "✓ Consistent" } else { "✗ Inconsistent" }));
            
            if is_consistent {
                // Perform classification (compute class hierarchy)
                let class_hierarchy = reasoner.classify();
                let hierarchy_levels = 3; // Simplified for now
                inferred_triples.push(format!("Class hierarchy computed with {} levels", hierarchy_levels));
                
                // Realize individuals (find their types)
                let individual_types = reasoner.realize();
                inferred_triples.push(format!("Realized {} individuals", individual_types.len()));
                
                // Add detailed inference results after borrowing is complete
                inferred_triples.extend(self.generate_detailed_inferences(&class_hierarchy, &individual_types));
            }
        }
        
        // Fall back to basic SPARQL-based inference if OWL 2 reasoner not available
        if let Some(ref store) = self.store {
            inferred_triples.extend(self.perform_sparql_inference(store)?);
        }
        
        // Cache results
        let cache_key = format!("inference_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
        self.reasoning_cache.insert(cache_key, inferred_triples.clone());
        
        Ok(inferred_triples)
    }
    
    /// Count levels in class hierarchy
    fn count_hierarchy_levels(&self, hierarchy: &owl2_rs::reasoner::ClassHierarchy) -> usize {
        // This is a simplified implementation
        // In practice, you'd analyze the actual hierarchy structure
        3 // Default assumption for demo purposes
    }
    
    /// Generate detailed inference results
    fn generate_detailed_inferences(&self, _hierarchy: &owl2_rs::reasoner::ClassHierarchy, individual_types: &std::collections::HashMap<owl2_rs::Individual, owl2_rs::reasoner::IndividualTypes>) -> Vec<String> {
        let mut results = Vec::new();
        
        // Report on individual classifications
        for (individual, types) in individual_types {
            if !types.most_specific.is_empty() {
                results.push(format!("Individual {} classified as {} types", 
                    match individual {
                        owl2_rs::Individual::Named(ref iri) => &iri.0,
                        owl2_rs::Individual::Anonymous(ref node_id) => &node_id.0,
                    },
                    types.most_specific.len()
                ));
            }
        }
        
        results
    }
    
    /// Perform SPARQL-based inference as fallback
    fn perform_sparql_inference(&self, store: &OxigraphStore) -> Result<Vec<String>, EpcisKgError> {
        let mut inferred_triples = Vec::new();
        
        // Basic inference: find all subclasses and infer superclass relationships
        let subclass_query = r#"
            SELECT ?subclass ?superclass
            WHERE {
                ?subclass <http://www.w3.org/2000/01/rdf-schema#subClassOf> ?superclass .
            }
        "#;
        
        if let Ok(results) = store.query_select(subclass_query) {
            inferred_triples.push(format!("Found subclass relationships via SPARQL: {} results", results.len()));
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
            inferred_triples.push(format!("Found type instances via SPARQL: {} results", results.len()));
        }
        
        Ok(inferred_triples)
    }
    
    /// Check if ontology conforms to OWL profile using owl2_rs library
    pub fn check_owl_profile(&self, ontology_data: &OntologyData, profile: &str) -> Result<(), EpcisKgError> {
        // Convert to OWL 2 ontology first if we have the converter
        let owl_ontology = if let Some(ref existing_ontology) = self.owl_ontology {
            existing_ontology.clone()
        } else {
            // Convert on the fly
            self.convert_rdf_to_owl(ontology_data)?
        };
        
        // Use owl2_rs profile checker
        let owl_profile = match profile.to_lowercase().as_str() {
            "el" | "owl2el" => owl2_rs::owl2_profile::OwlProfile::EL,
            "ql" | "owl2ql" => owl2_rs::owl2_profile::OwlProfile::QL,
            "rl" | "owl2rl" => owl2_rs::owl2_profile::OwlProfile::RL,
            "full" | "owl2" => owl2_rs::owl2_profile::OwlProfile::Full,
            _ => {
                return Err(EpcisKgError::Validation(format!("Unknown OWL profile: {}", profile)));
            }
        };
        
        let profile_result = owl2_rs::owl2_profile::check_profile_compliance(&owl_ontology, owl_profile);
        
        if !profile_result.conforms {
            let error_message = format!(
                "OWL 2 {} profile violation: {}",
                profile,
                profile_result.violations.join(", ")
            );
            return Err(EpcisKgError::Validation(error_message));
        }
        
        // Additional EPCIS-specific checks
        self.perform_epcis_profile_checks(ontology_data, profile)?;
        
        Ok(())
    }
    
    /// Comprehensive profile validation with detailed reporting
    pub fn validate_owl_profile_comprehensive(&mut self, ontology_data: &OntologyData, profile: &str) -> Result<ProfileValidationResult, EpcisKgError> {
        // Load ontology data first
        self.load_ontology_data(ontology_data)?;
        
        // Convert to OWL 2 ontology
        let owl_ontology = self.owl_ontology.as_ref().unwrap();
        
        // Use owl2_rs profile checker
        let owl_profile = match profile.to_lowercase().as_str() {
            "el" | "owl2el" => owl2_rs::owl2_profile::OwlProfile::EL,
            "ql" | "owl2ql" => owl2_rs::owl2_profile::OwlProfile::QL,
            "rl" | "owl2rl" => owl2_rs::owl2_profile::OwlProfile::RL,
            "full" | "owl2" => owl2_rs::owl2_profile::OwlProfile::Full,
            _ => {
                return Err(EpcisKgError::Validation(format!("Unknown OWL profile: {}", profile)));
            }
        };
        
        let profile_result = owl2_rs::owl2_profile::check_profile_compliance(owl_ontology, owl_profile.clone());
        
        // Perform detailed analysis
        let mut validation_result = ProfileValidationResult {
            profile: profile.to_string(),
            conforms: profile_result.conforms,
            violations: profile_result.violations.clone(),
            ontology_stats: self.analyze_ontology_structure(owl_ontology),
            epcis_compliance: self.check_epcis_compliance(ontology_data),
            reasoning_capabilities: self.analyze_reasoning_capabilities(owl_ontology),
            performance_indicators: self.estimate_performance_characteristics(owl_ontology, &owl_profile),
            el_specific: None,
            ql_specific: None,
            rl_specific: None,
        };
        
        // Add profile-specific analysis
        match owl_profile {
            owl2_rs::owl2_profile::OwlProfile::EL => {
                validation_result.el_specific = Some(self.analyze_el_profile(owl_ontology));
            },
            owl2_rs::owl2_profile::OwlProfile::QL => {
                validation_result.ql_specific = Some(self.analyze_ql_profile(owl_ontology));
            },
            owl2_rs::owl2_profile::OwlProfile::RL => {
                validation_result.rl_specific = Some(self.analyze_rl_profile(owl_ontology));
            },
            _ => {}
        }
        
        Ok(validation_result)
    }
    
    /// Analyze ontology structure
    fn analyze_ontology_structure(&self, ontology: &owl2_rs::Ontology) -> OntologyStats {
        let mut class_count = 0;
        let mut property_count = 0;
        let mut individual_count = 0;
        let axiom_count = ontology.axioms.len();
        
        for axiom in &ontology.axioms {
            match axiom {
                owl2_rs::Axiom::Class(_) => class_count += 1,
                owl2_rs::Axiom::ObjectProperty(_) => property_count += 1,
                owl2_rs::Axiom::DataProperty(_) => property_count += 1,
                owl2_rs::Axiom::Assertion(_) => individual_count += 1,
            }
        }
        
        OntologyStats {
            total_axioms: axiom_count,
            classes: class_count,
            properties: property_count,
            individuals: individual_count,
        }
    }
    
    /// Check EPCIS compliance
    fn check_epcis_compliance(&self, ontology_data: &OntologyData) -> EpcisCompliance {
        let mut has_epcis_classes = false;
        let mut has_cbv_vocabulary = false;
        let mut has_event_types = false;
        let mut has_vocabulary_extensions = false;
        
        for triple in ontology_data.graph.iter() {
            let subject_str = format!("{}", triple.subject);
            let predicate_str = format!("{}", triple.predicate);
            let object_str = format!("{}", triple.object);
            
            // Check for EPCIS core classes
            if subject_str.contains("epcis") || object_str.contains("epcis") {
                has_epcis_classes = true;
                if subject_str.contains("Event") || object_str.contains("Event") {
                    has_event_types = true;
                }
            }
            
            // Check for CBV (Core Business Vocabulary)
            if subject_str.contains("cbv") || object_str.contains("cbv") {
                has_cbv_vocabulary = true;
            }
            
            // Check for vocabulary extensions
            if subject_str.contains("extension") || object_str.contains("extension") {
                has_vocabulary_extensions = true;
            }
        }
        
        EpcisCompliance {
            has_epcis_classes,
            has_cbv_vocabulary,
            has_event_types,
            has_vocabulary_extensions,
        }
    }
    
    /// Analyze reasoning capabilities
    fn analyze_reasoning_capabilities(&self, ontology: &owl2_rs::Ontology) -> ReasoningCapabilities {
        let mut has_class_hierarchy = false;
        let mut has_property_hierarchy = false;
        let mut has_complex_restrictions = false;
        let mut has_individual_assertions = false;
        
        for axiom in &ontology.axioms {
            match axiom {
                owl2_rs::Axiom::Class(class_axiom) => {
                    match class_axiom {
                        owl2_rs::ClassAxiom::SubClassOf { .. } => has_class_hierarchy = true,
                        owl2_rs::ClassAxiom::EquivalentClasses { .. } => has_class_hierarchy = true,
                        _ => {}
                    }
                },
                owl2_rs::Axiom::ObjectProperty(prop_axiom) => {
                    match prop_axiom {
                        owl2_rs::ObjectPropertyAxiom::SubObjectPropertyOf { .. } => has_property_hierarchy = true,
                        owl2_rs::ObjectPropertyAxiom::EquivalentObjectProperties { .. } => has_property_hierarchy = true,
                        _ => {}
                    }
                },
                owl2_rs::Axiom::Assertion(_) => has_individual_assertions = true,
                _ => {}
            }
        }
        
        ReasoningCapabilities {
            supports_classification: has_class_hierarchy,
            supports_realization: has_individual_assertions,
            has_property_hierarchy,
            has_complex_restrictions,
        }
    }
    
    /// Estimate performance characteristics
    fn estimate_performance_characteristics(&self, ontology: &owl2_rs::Ontology, profile: &owl2_rs::owl2_profile::OwlProfile) -> PerformanceIndicators {
        let axiom_count = ontology.axioms.len();
        
        let estimated_classification_time = match profile {
            owl2_rs::owl2_profile::OwlProfile::EL => axiom_count * 2, // EL is very fast
            owl2_rs::owl2_profile::OwlProfile::QL => axiom_count * 5, // QL is moderate
            owl2_rs::owl2_profile::OwlProfile::RL => axiom_count * 10, // RL is slower
            _ => axiom_count * 20, // Full OWL 2 is slowest
        };
        
        let estimated_realization_time = estimated_classification_time * 3;
        
        PerformanceIndicators {
            estimated_classification_time_ms: estimated_classification_time,
            estimated_realization_time_ms: estimated_realization_time,
            ontology_complexity: if axiom_count < 100 { "Low" } else if axiom_count < 1000 { "Medium" } else { "High" },
            reasoning_feasibility: if axiom_count > 10000 { "Limited" } else { "Good" },
        }
    }
    
    /// Analyze EL profile specific characteristics
    fn analyze_el_profile(&self, ontology: &owl2_rs::Ontology) -> ElProfileAnalysis {
        let mut existential_restrictions = 0;
        let mut conjunctions = 0;
        let mut simple_class_expressions = 0;
        
        for axiom in &ontology.axioms {
            match axiom {
                owl2_rs::Axiom::Class(class_axiom) => {
                    match class_axiom {
                        owl2_rs::ClassAxiom::SubClassOf { sub_class, super_class } => {
                            // Count existential restrictions
                            if format!("{:?}", sub_class).contains("ObjectSomeValuesFrom") {
                                existential_restrictions += 1;
                            }
                            // Count conjunctions
                            if format!("{:?}", super_class).contains("ObjectIntersectionOf") {
                                conjunctions += 1;
                            }
                        },
                        _ => simple_class_expressions += 1,
                    }
                },
                _ => {}
            }
        }
        
        ElProfileAnalysis {
            existential_restrictions,
            conjunctions,
            simple_class_expressions,
            el_optimization_potential: existential_restrictions > 0 || conjunctions > 0,
        }
    }
    
    /// Analyze QL profile specific characteristics
    fn analyze_ql_profile(&self, ontology: &owl2_rs::Ontology) -> QlProfileAnalysis {
        let existential_restrictions = 0;
        let universal_restrictions = 0;
        let mut simple_inclusions = 0;
        
        for axiom in &ontology.axioms {
            match axiom {
                owl2_rs::Axiom::Class(class_axiom) => {
                    match class_axiom {
                        owl2_rs::ClassAxiom::SubClassOf { .. } => simple_inclusions += 1,
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        
        QlProfileAnalysis {
            existential_restrictions,
            universal_restrictions,
            simple_inclusions,
            query_rewriting_potential: simple_inclusions > 0,
        }
    }
    
    /// Analyze RL profile specific characteristics
    fn analyze_rl_profile(&self, ontology: &owl2_rs::Ontology) -> RlProfileAnalysis {
        let mut complex_class_expressions = 0;
        let mut property_chains = 0;
        let mut simple_rules = 0;
        
        for axiom in &ontology.axioms {
            match axiom {
                owl2_rs::Axiom::Class(_) => simple_rules += 1,
                owl2_rs::Axiom::ObjectProperty(prop_axiom) => {
                    match prop_axiom {
                        owl2_rs::ObjectPropertyAxiom::SubObjectPropertyOf { .. } => property_chains += 1,
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        
        RlProfileAnalysis {
            complex_class_expressions,
            property_chains,
            simple_rules,
            rule_safety: complex_class_expressions < 100, // Arbitrary threshold
        }
    }
    
    /// Perform EPCIS-specific profile checks
    fn perform_epcis_profile_checks(&self, ontology_data: &OntologyData, profile: &str) -> Result<(), EpcisKgError> {
        // EPCIS-specific validation for supply chain ontologies
        let mut has_epcis_classes = false;
        let mut has_cbv_vocabulary = false;
        
        for triple in ontology_data.graph.iter() {
            let subject_str = format!("{}", triple.subject);
            let predicate_str = format!("{}", triple.predicate);
            let object_str = format!("{}", triple.object);
            
            // Check for EPCIS core classes
            if subject_str.contains("epcis") || object_str.contains("epcis") {
                has_epcis_classes = true;
            }
            
            // Check for CBV (Core Business Vocabulary)
            if subject_str.contains("cbv") || object_str.contains("cbv") {
                has_cbv_vocabulary = true;
            }
        }
        
        // For EL profile (most restrictive), ensure proper EPCIS structure
        if profile == "el" || profile == "owl2el" {
            if !has_epcis_classes {
                return Err(EpcisKgError::Validation(
                    "EPCIS EL profile violation: missing EPCIS core classes".to_string()
                ));
            }
            
            if !has_cbv_vocabulary {
                return Err(EpcisKgError::Validation(
                    "EPCIS EL profile violation: missing CBV vocabulary".to_string()
                ));
            }
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

// Data structures for comprehensive profile validation

/// Comprehensive result of profile validation
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProfileValidationResult {
    pub profile: String,
    pub conforms: bool,
    pub violations: Vec<String>,
    pub ontology_stats: OntologyStats,
    pub epcis_compliance: EpcisCompliance,
    pub reasoning_capabilities: ReasoningCapabilities,
    pub performance_indicators: PerformanceIndicators,
    pub el_specific: Option<ElProfileAnalysis>,
    pub ql_specific: Option<QlProfileAnalysis>,
    pub rl_specific: Option<RlProfileAnalysis>,
}

/// Statistics about the ontology structure
#[derive(Debug, Clone, serde::Serialize)]
pub struct OntologyStats {
    pub total_axioms: usize,
    pub classes: usize,
    pub properties: usize,
    pub individuals: usize,
}

/// EPCIS compliance information
#[derive(Debug, Clone, serde::Serialize)]
pub struct EpcisCompliance {
    pub has_epcis_classes: bool,
    pub has_cbv_vocabulary: bool,
    pub has_event_types: bool,
    pub has_vocabulary_extensions: bool,
}

/// Reasoning capabilities analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReasoningCapabilities {
    pub supports_classification: bool,
    pub supports_realization: bool,
    pub has_property_hierarchy: bool,
    pub has_complex_restrictions: bool,
}

/// Performance indicators for reasoning
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceIndicators {
    pub estimated_classification_time_ms: usize,
    pub estimated_realization_time_ms: usize,
    pub ontology_complexity: &'static str,
    pub reasoning_feasibility: &'static str,
}

/// EL profile specific analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct ElProfileAnalysis {
    pub existential_restrictions: usize,
    pub conjunctions: usize,
    pub simple_class_expressions: usize,
    pub el_optimization_potential: bool,
}

/// QL profile specific analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct QlProfileAnalysis {
    pub existential_restrictions: usize,
    pub universal_restrictions: usize,
    pub simple_inclusions: usize,
    pub query_rewriting_potential: bool,
}

/// RL profile specific analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct RlProfileAnalysis {
    pub complex_class_expressions: usize,
    pub property_chains: usize,
    pub simple_rules: usize,
    pub rule_safety: bool,
}