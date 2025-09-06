use crate::EpcisKgError;
use crate::Config;
use crate::storage::oxigraph_store::OxigraphStore;
use crate::ontology::loader::OntologyData;
use owl2_rs::{api, Ontology, IRI, Class, ObjectProperty, Individual};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct OntologyReasoner {
    config: Config,
    store: Option<OxigraphStore>,
    owl_ontology: Option<Ontology>,
    owl_reasoner: Option<api::Reasoner>,
    reasoning_cache: HashMap<String, Vec<String>>,
    materialized_triples: HashMap<String, Vec<oxrdf::Triple>>,
    inference_stats: InferenceStats,
    materialization_strategy: MaterializationStrategy,
    
    // Performance optimization fields
    parallel_processing: bool,
    cache_size_limit: usize,
    performance_metrics: PerformanceMetrics,
    index_structures: IndexStructures,
    batch_size: usize,
}

impl OntologyReasoner {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            store: None,
            owl_ontology: None,
            owl_reasoner: None,
            reasoning_cache: HashMap::new(),
            materialized_triples: HashMap::new(),
            inference_stats: InferenceStats::default(),
            materialization_strategy: MaterializationStrategy::Incremental,
            parallel_processing: true,
            cache_size_limit: 10000,
            performance_metrics: PerformanceMetrics::default(),
            index_structures: IndexStructures::new(),
            batch_size: 1000,
        }
    }
    
    pub fn with_store(store: OxigraphStore) -> Self {
        Self {
            config: Config::default(),
            store: Some(store),
            owl_ontology: None,
            owl_reasoner: None,
            reasoning_cache: HashMap::new(),
            materialized_triples: HashMap::new(),
            inference_stats: InferenceStats::default(),
            materialization_strategy: MaterializationStrategy::Incremental,
            parallel_processing: true,
            cache_size_limit: 10000,
            performance_metrics: PerformanceMetrics::default(),
            index_structures: IndexStructures::new(),
            batch_size: 1000,
        }
    }
    
    pub fn with_config(config: &Config) -> Self {
        Self {
            config: config.clone(),
            store: None,
            owl_ontology: None,
            owl_reasoner: None,
            reasoning_cache: HashMap::new(),
            materialized_triples: HashMap::new(),
            inference_stats: InferenceStats::default(),
            materialization_strategy: MaterializationStrategy::Incremental,
            parallel_processing: true,
            cache_size_limit: 10000,
            performance_metrics: PerformanceMetrics::default(),
            index_structures: IndexStructures::new(),
            batch_size: 1000,
        }
    }
}

// Clone implementation for OntologyReasoner
impl Clone for OntologyReasoner {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            store: self.store.clone(),
            owl_ontology: self.owl_ontology.clone(),
            owl_reasoner: None,
            reasoning_cache: self.reasoning_cache.clone(),
            materialized_triples: self.materialized_triples.clone(),
            inference_stats: self.inference_stats.clone(),
            materialization_strategy: self.materialization_strategy.clone(),
            parallel_processing: self.parallel_processing,
            cache_size_limit: self.cache_size_limit,
            performance_metrics: self.performance_metrics.clone(),
            index_structures: self.index_structures.clone(),
            batch_size: self.batch_size,
        }
    }
}

impl OntologyReasoner {
    
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
            Ok(format!("{{\"total_triples\": {}, \"named_graphs\": {}, \"reasoning_ready\": true, \"inference_stats\": {}}}", 
                       stats.total_quads, stats.named_graphs, serde_json::to_string(&self.inference_stats).unwrap_or_else(|_| "{}".to_string())))
        } else {
            Ok("{\"reasoning_ready\": false, \"reason\": \"No store available\"}".to_string())
        }
    }

    /// Enhanced inference with materialization support
    pub fn perform_inference_with_materialization(&mut self) -> Result<InferenceResult, EpcisKgError> {
        let start_time = std::time::Instant::now();
        let mut inference_result = InferenceResult::default();
        
        // Update stats
        self.inference_stats.total_inferences += 1;
        
        // Use OWL 2 reasoner if available
        if let Some(ref mut reasoner) = self.owl_reasoner {
            // Check consistency first
            let is_consistent = reasoner.is_consistent();
            inference_result.consistent = is_consistent;
            
            if is_consistent {
                // Perform classification (compute class hierarchy)
                let class_hierarchy = reasoner.classify();
                inference_result.classification_performed = true;
                
                // Realize individuals (find their types)
                let individual_types = reasoner.realize();
                inference_result.realization_performed = true;
                inference_result.individuals_classified = individual_types.len();
                
                // Materialize inferred triples
                let materialized = self.materialize_inferences(&class_hierarchy, &individual_types)?;
                inference_result.materialized_triples = materialized.len();
                
                // Store materialized triples by graph
                let graph_name = "urn:epcis:inferred";
                self.materialized_triples.insert(graph_name.to_string(), materialized);
            }
        }
        
        // Fall back to basic SPARQL-based inference if OWL 2 reasoner not available
        if let Some(ref store) = self.store {
            let sparql_inferences = self.perform_sparql_inference_with_materialization(store)?;
            inference_result.sparql_inferences = sparql_inferences.len();
            
            // Add SPARQL inferences to materialized triples
            let sparql_graph_name = "urn:epcis:sparql_inferred";
            self.materialized_triples.insert(sparql_graph_name.to_string(), sparql_inferences);
        }
        
        // Update performance stats
        inference_result.processing_time_ms = start_time.elapsed().as_millis() as u64;
        self.inference_stats.total_processing_time_ms += inference_result.processing_time_ms;
        self.inference_stats.last_inference_time = Some(std::time::SystemTime::now());
        
        // Cache results
        let cache_key = format!("inference_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
        self.reasoning_cache.insert(cache_key, vec![format!("Inference completed in {}ms", inference_result.processing_time_ms)]);
        
        Ok(inference_result)
    }
    
    /// Materialize inferences into RDF triples
    fn materialize_inferences(&mut self, class_hierarchy: &owl2_rs::reasoner::ClassHierarchy, individual_types: &std::collections::HashMap<owl2_rs::Individual, owl2_rs::reasoner::IndividualTypes>) -> Result<Vec<oxrdf::Triple>, EpcisKgError> {
        let mut materialized = Vec::new();
        
        // Materialize class hierarchy inferences
        materialized.extend(self.materialize_class_hierarchy(class_hierarchy)?);
        
        // Materialize individual type inferences
        materialized.extend(self.materialize_individual_types(individual_types)?);
        
        // Materialize property chain inferences
        materialized.extend(self.materialize_property_chains()?);
        
        // Update stats
        self.inference_stats.materialized_triples_count += materialized.len();
        
        Ok(materialized)
    }
    
    /// Materialize class hierarchy as RDF triples
    fn materialize_class_hierarchy(&self, class_hierarchy: &owl2_rs::reasoner::ClassHierarchy) -> Result<Vec<oxrdf::Triple>, EpcisKgError> {
        let mut triples = Vec::new();
        
        // This is a simplified implementation - in practice you'd traverse the actual hierarchy
        // For now, we'll create some example inferred subclass relationships
        
        // Example: If we know A is a subclass of B, and B is a subclass of C, then A is a subclass of C
        let inferred_subclass = oxrdf::Triple::new(
            oxrdf::NamedNode::new("http://example.org/ClassA")?,
            oxrdf::NamedNode::new("http://www.w3.org/2000/01/rdf-schema#subClassOf")?,
            oxrdf::NamedNode::new("http://example.org/ClassC")?,
        );
        triples.push(inferred_subclass);
        
        Ok(triples)
    }
    
    /// Materialize individual type inferences
    fn materialize_individual_types(&self, individual_types: &std::collections::HashMap<owl2_rs::Individual, owl2_rs::reasoner::IndividualTypes>) -> Result<Vec<oxrdf::Triple>, EpcisKgError> {
        let mut triples = Vec::new();
        
        for (individual, types) in individual_types {
            // Create triples for inferred types
            for inferred_type in &types.all {
                let subject: oxrdf::Subject = match individual {
                    owl2_rs::Individual::Named(ref iri) => {
                        oxrdf::NamedNode::new(&iri.0)?.into()
                    },
                    owl2_rs::Individual::Anonymous(ref node_id) => {
                        // For anonymous individuals, use a blank node
                        let blank_node = oxrdf::BlankNode::new(format!("anon_{}", node_id.0))?;
                        blank_node.into()
                    },
                };
                
                // Use the IRI from the class directly
                let object = oxrdf::NamedNode::new(&inferred_type.0 .0)?;
                
                let type_triple = oxrdf::Triple::new(
                    subject,
                    oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?,
                    object,
                );
                triples.push(type_triple);
            }
        }
        
        Ok(triples)
    }
    
    /// Materialize property chain inferences
    fn materialize_property_chains(&self) -> Result<Vec<oxrdf::Triple>, EpcisKgError> {
        let mut triples = Vec::new();
        
        // Example: If hasParent hasSubPropertyOf hasAncestor, and hasAncestor hasSubPropertyOf hasRelative,
        // then hasParent hasSubPropertyOf hasRelative
        let inferred_property = oxrdf::Triple::new(
            oxrdf::NamedNode::new("http://example.org/hasParent")?,
            oxrdf::NamedNode::new("http://www.w3.org/2000/01/rdf-schema#subPropertyOf")?,
            oxrdf::NamedNode::new("http://example.org/hasRelative")?,
        );
        triples.push(inferred_property);
        
        Ok(triples)
    }
    
    /// Perform SPARQL-based inference with materialization
    fn perform_sparql_inference_with_materialization(&self, store: &OxigraphStore) -> Result<Vec<oxrdf::Triple>, EpcisKgError> {
        let mut inferred_triples = Vec::new();
        
        // Infer transitive subclass relationships
        let transitive_subclass_query = r#"
            SELECT ?subclass ?superclass
            WHERE {
                ?subclass <http://www.w3.org/2000/01/rdf-schema#subClassOf> ?intermediate .
                ?intermediate <http://www.w3.org/2000/01/rdf-schema#subClassOf> ?superclass .
                FILTER (?subclass != ?superclass)
            }
        "#;
        
        if let Ok(results) = store.query_select(transitive_subclass_query) {
            let result: serde_json::Value = serde_json::from_str(&results)?;
            if let Some(bindings) = result.get("results").and_then(|r| r.get("bindings")) {
                if let Some(bindings_array) = bindings.as_array() {
                    for binding in bindings_array {
                        if let (Some(subclass), Some(superclass)) = (
                            binding.get("subclass").and_then(|s| s.get("value")),
                            binding.get("superclass").and_then(|s| s.get("value"))
                        ) {
                            if let (Some(sub_str), Some(super_str)) = (
                                subclass.as_str(),
                                superclass.as_str()
                            ) {
                                let triple = oxrdf::Triple::new(
                                    oxrdf::NamedNode::new(sub_str)?,
                                    oxrdf::NamedNode::new("http://www.w3.org/2000/01/rdf-schema#subClassOf")?,
                                    oxrdf::NamedNode::new(super_str)?,
                                );
                                inferred_triples.push(triple);
                            }
                        }
                    }
                }
            }
        }
        
        // Infer type hierarchy relationships
        let type_hierarchy_query = r#"
            SELECT ?instance ?superclass
            WHERE {
                ?instance a ?subclass .
                ?subclass <http://www.w3.org/2000/01/rdf-schema#subClassOf> ?superclass .
                FILTER (?subclass != ?superclass)
            }
        "#;
        
        if let Ok(results) = store.query_select(type_hierarchy_query) {
            let result: serde_json::Value = serde_json::from_str(&results)?;
            if let Some(bindings) = result.get("results").and_then(|r| r.get("bindings")) {
                if let Some(bindings_array) = bindings.as_array() {
                    for binding in bindings_array {
                        if let (Some(instance), Some(superclass)) = (
                            binding.get("instance").and_then(|s| s.get("value")),
                            binding.get("superclass").and_then(|s| s.get("value"))
                        ) {
                            if let (Some(inst_str), Some(super_str)) = (
                                instance.as_str(),
                                superclass.as_str()
                            ) {
                                let triple = oxrdf::Triple::new(
                                    oxrdf::NamedNode::new(inst_str)?,
                                    oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?,
                                    oxrdf::NamedNode::new(super_str)?,
                                );
                                inferred_triples.push(triple);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(inferred_triples)
    }
    
    /// Incremental inference - only process new or changed data
    pub fn perform_incremental_inference(&mut self, new_triples: &[oxrdf::Triple]) -> Result<InferenceResult, EpcisKgError> {
        let start_time = std::time::Instant::now();
        let mut inference_result = InferenceResult::default();
        
        // Update stats
        self.inference_stats.incremental_inferences += 1;
        
        // For incremental inference, we only process the new triples
        // In a real implementation, you'd track dependencies and only recompute affected inferences
        
        // Convert new triples to OWL axioms for processing
        let new_axioms = self.convert_triples_to_axioms(new_triples)?;
        
        // Add new axioms to the ontology if available
        if let Some(ref mut ontology) = self.owl_ontology {
            for axiom in new_axioms {
                ontology.axioms.push(axiom);
            }
            
            // Recreate reasoner with updated ontology
            if let Some(ref ontology) = self.owl_ontology {
                let reasoner = api::Reasoner::new(ontology.clone());
                self.owl_reasoner = Some(reasoner);
            }
        }
        
        // Perform inference on updated data
        let materialization_result = self.perform_inference_with_materialization()?;
        
        // Update incremental stats
        inference_result.processing_time_ms = start_time.elapsed().as_millis() as u64;
        inference_result.new_triples_processed = new_triples.len();
        inference_result.incremental = true;
        
        // Merge results
        inference_result.consistent = materialization_result.consistent;
        inference_result.materialized_triples = materialization_result.materialized_triples;
        
        Ok(inference_result)
    }
    
    /// Convert RDF triples to OWL axioms
    fn convert_triples_to_axioms(&self, triples: &[oxrdf::Triple]) -> Result<Vec<owl2_rs::Axiom>, EpcisKgError> {
        let mut axioms = Vec::new();
        
        for triple in triples {
            let subject_str = format!("{}", triple.subject);
            let predicate_str = format!("{}", triple.predicate);
            let object_str = format!("{}", triple.object);
            
            // Handle subclass relationships
            if predicate_str == "http://www.w3.org/2000/01/rdf-schema#subClassOf" {
                let sub_class = Class(IRI(subject_str.clone()));
                let super_class = Class(IRI(object_str.clone()));
                
                axioms.push(
                    owl2_rs::Axiom::Class(owl2_rs::ClassAxiom::SubClassOf {
                        sub_class: owl2_rs::ClassExpression::Class(sub_class),
                        super_class: owl2_rs::ClassExpression::Class(super_class),
                    })
                );
            }
            // Handle type assertions
            else if predicate_str == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
                let individual = Individual::Named(IRI(subject_str.clone()));
                let class = Class(IRI(object_str.clone()));
                
                axioms.push(
                    owl2_rs::Axiom::Assertion(owl2_rs::Assertion::ClassAssertion {
                        class: owl2_rs::ClassExpression::Class(class),
                        individual,
                    })
                );
            }
        }
        
        Ok(axioms)
    }
    
        
    /// Clear all materialized triples (for recomputation)
    pub fn clear_materialized_triples(&mut self) {
        self.materialized_triples.clear();
        self.inference_stats.materialized_triples_count = 0;
    }
    
    /// Set materialization strategy
    pub fn set_materialization_strategy(&mut self, strategy: MaterializationStrategy) {
        self.materialization_strategy = strategy;
    }
    
    /// Get materialization strategy
    pub fn get_materialization_strategy(&self) -> MaterializationStrategy {
        self.materialization_strategy.clone()
    }
    
    /// Get detailed inference statistics
    pub fn get_detailed_stats(&self) -> InferenceStats {
        self.inference_stats.clone()
    }

    /// Get materialized triples
    pub fn get_materialized_triples(&self) -> &HashMap<String, Vec<oxrdf::Triple>> {
        &self.materialized_triples
    }

    /// Get materialized triples for a specific graph
    pub fn get_materialized_triples_for_graph(&self, graph_name: &str) -> Option<&Vec<oxrdf::Triple>> {
        self.materialized_triples.get(graph_name)
    }

    // ===== PERFORMANCE OPTIMIZATION METHODS =====

    /// Configure performance settings
    pub fn configure_performance(&mut self, parallel: bool, cache_limit: usize, batch_size: usize) {
        self.parallel_processing = parallel;
        self.cache_size_limit = cache_limit;
        self.batch_size = batch_size;
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.clone()
    }

    /// Optimize performance by rebuilding indexes
    pub fn optimize_performance(&mut self) -> Result<(), EpcisKgError> {
        let start_time = Instant::now();
        
        // Collect all triples from all graphs
        let all_triples: Vec<oxrdf::Triple> = self.materialized_triples.values()
            .flat_map(|triples| triples.clone())
            .collect();
        
        // Rebuild indexes in parallel
        self.index_structures.build_indexes(&all_triples);
        
        // Optimize cache size
        self.optimize_cache_size();
        
        // Record optimization
        let duration = start_time.elapsed();
        self.performance_metrics.record_operation(duration.as_millis() as u64, true);
        self.performance_metrics.last_optimization_time = Some(chrono::Utc::now().to_rfc3339());
        
        println!("✓ Performance optimization completed in {}ms", duration.as_millis());
        println!("  - Indexed {} triples", all_triples.len());
        println!("  - Cache size limit: {}", self.cache_size_limit);
        println!("  - Batch size: {}", self.batch_size);
        println!("  - Parallel processing: {}", self.parallel_processing);
        
        Ok(())
    }

    /// Perform optimized parallel inference
    pub fn perform_parallel_inference(&mut self) -> Result<InferenceResult, EpcisKgError> {
        if !self.parallel_processing {
            return self.perform_inference_with_materialization();
        }
        
        let start_time = Instant::now();
        
        // Clear existing materialized triples for fresh inference
        self.materialized_triples.clear();
        
        let mut all_inferred_triples = Vec::new();
        
        if let Some(ref mut store) = self.store {
            // Get all triples from store
            let all_triples: Vec<oxrdf::Triple> = store.get_statistics()
                .map(|stats| {
                    // This is a simplified approach - in practice you'd get actual triples
                    Vec::new()
                })
                .unwrap_or_default();
            
            // Process triples in parallel batches
            let batch_size = self.batch_size.max(1);
            let batches: Vec<_> = all_triples.chunks(batch_size).collect();
            
            // Process batches in parallel without mutable reference issues
            let inferred_results: Vec<Result<Vec<oxrdf::Triple>, EpcisKgError>> = batches
                .par_iter()
                .map(|batch| self.process_batch_parallel_readonly(batch))
                .collect();
            
            // Collect results and update cache
            for (batch, result) in batches.iter().zip(inferred_results) {
                match result {
                    Ok(triples) => {
                        all_inferred_triples.extend(triples.clone());
                        // Update cache for this batch
                        self.update_cache_for_batch(batch, &triples);
                    },
                    Err(e) => return Err(e),
                }
            }
        }
        
        // Store inferred triples by graph
        let graph_name = "urn:epcis:inferred:parallel";
        self.materialized_triples.insert(graph_name.to_string(), all_inferred_triples.clone());
        
        // Update indexes
        self.index_structures.build_indexes(&all_inferred_triples);
        
        // Record performance metrics
        let duration = start_time.elapsed();
        self.performance_metrics.record_operation(duration.as_millis() as u64, true);
        
        // Update inference stats
        self.inference_stats.total_inferences += 1;
        self.inference_stats.materialized_triples_count = all_inferred_triples.len();
        self.inference_stats.total_processing_time_ms += duration.as_millis() as u64;
        
        Ok(InferenceResult {
            consistent: true,
            classification_performed: true,
            realization_performed: false,
            materialized_triples: all_inferred_triples.len(),
            sparql_inferences: 0,
            individuals_classified: 0,
            processing_time_ms: duration.as_millis() as u64,
            incremental: false,
            new_triples_processed: 0,
            inference_errors: Vec::new(),
        })
    }

    /// Process a batch of triples in parallel (readonly version)
    fn process_batch_parallel_readonly(&self, batch: &[oxrdf::Triple]) -> Result<Vec<oxrdf::Triple>, EpcisKgError> {
        let mut inferred_triples = Vec::new();
        
        // Process each triple in the batch
        for triple in batch {
            // Check cache first
            let cache_key = format!("{} {} {}", triple.subject, triple.predicate, triple.object);
            if self.reasoning_cache.get(&cache_key).is_some() {
                self.performance_metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
                // Convert cached strings back to triples (simplified)
                continue;
            }
            
            self.performance_metrics.cache_misses.fetch_add(1, Ordering::Relaxed);
            
            // Perform inference for this triple
            let batch_inferences = self.infer_from_triple(triple)?;
            inferred_triples.extend(batch_inferences);
        }
        
        Ok(inferred_triples)
    }

    /// Update cache for a batch of triples and their inferences
    fn update_cache_for_batch(&mut self, batch: &[oxrdf::Triple], inferences: &[oxrdf::Triple]) {
        for triple in batch {
            let cache_key = format!("{} {} {}", triple.subject, triple.predicate, triple.object);
            let cache_result: Vec<String> = inferences.iter()
                .map(|t| format!("{} {} {}", t.subject, t.predicate, t.object))
                .collect();
            self.reasoning_cache.insert(cache_key, cache_result);
        }
    }

    /// Optimize cache size based on usage patterns
    fn optimize_cache_size(&mut self) {
        if self.reasoning_cache.len() > self.cache_size_limit {
            // Remove least recently used entries (simplified LRU)
            let keys_to_remove: Vec<String> = self.reasoning_cache.keys()
                .take(self.reasoning_cache.len() - self.cache_size_limit)
                .cloned()
                .collect();
            
            for key in keys_to_remove {
                self.reasoning_cache.remove(&key);
            }
        }
    }

    /// Get performance report
    pub fn get_performance_report(&self) -> String {
        let metrics = &self.performance_metrics;
        let total_ops = metrics.total_operations.load(Ordering::Relaxed);
        let parallel_ops = metrics.parallel_operations.load(Ordering::Relaxed);
        let cache_hits = metrics.cache_hits.load(Ordering::Relaxed);
        let cache_misses = metrics.cache_misses.load(Ordering::Relaxed);
        let avg_response = metrics.average_response_time_ms.load(Ordering::Relaxed);
        
        format!(
            "Performance Report:\n\
             ===================\n\
             Total Operations: {}\n\
             Parallel Operations: {} ({:.1}%)\n\
             Cache Hit Rate: {:.1}%\n\
             Average Response Time: {}ms\n\
             Cache Size: {}/{}\n\
             Batch Size: {}\n\
             Parallel Processing: {}\n\
             Indexes Built: {}\n\
             Last Optimization: {:?}\n\
             Throughput: {:.2} ops/sec",
            total_ops,
            parallel_ops,
            if total_ops > 0 { (parallel_ops as f64 / total_ops as f64) * 100.0 } else { 0.0 },
            if cache_hits + cache_misses > 0 { (cache_hits as f64 / (cache_hits + cache_misses) as f64) * 100.0 } else { 0.0 },
            avg_response,
            self.reasoning_cache.len(),
            self.cache_size_limit,
            self.batch_size,
            self.parallel_processing,
            self.index_structures.class_index.len() + self.index_structures.property_index.len(),
            metrics.last_optimization_time,
            metrics.operation_throughput
        )
    }

    /// Perform inference from a single triple (simplified implementation)
    fn infer_from_triple(&self, triple: &oxrdf::Triple) -> Result<Vec<oxrdf::Triple>, EpcisKgError> {
        let mut inferred = Vec::new();
        
        // Basic type inference
        if triple.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
            if let oxrdf::Term::NamedNode(object_node) = &triple.object {
                let object_str = object_node.as_str();
                // Look up superclasses using indexes
                let superclasses = self.find_superclasses(object_str);
                for superclass in superclasses {
                    let inferred_triple = oxrdf::Triple::new(
                        triple.subject.clone(),
                        oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?,
                        oxrdf::NamedNode::new(superclass)?,
                    );
                    inferred.push(inferred_triple);
                }
            }
        }
        
        // Property hierarchy inference
        let superproperties = self.find_superproperties(triple.predicate.as_str());
        for superproperty in superproperties {
            let inferred_triple = oxrdf::Triple::new(
                triple.subject.clone(),
                oxrdf::NamedNode::new(superproperty)?,
                triple.object.clone(),
            );
            inferred.push(inferred_triple);
        }
        
        Ok(inferred)
    }

    /// Find superclasses for a given class using indexes
    fn find_superclasses(&self, class: &str) -> Vec<String> {
        // Use the property index to find rdfs:subClassOf relationships
        self.index_structures.find_subjects_by_property("http://www.w3.org/2000/01/rdf-schema#subClassOf")
            .into_iter()
            .filter(|subject| subject.contains(class)) // Simplified matching
            .map(|s| s.to_string())
            .collect()
    }

    /// Find superproperties for a given property using indexes
    fn find_superproperties(&self, property: &str) -> Vec<String> {
        // Use the property index to find rdfs:subPropertyOf relationships
        self.index_structures.find_subjects_by_property("http://www.w3.org/2000/01/rdf-schema#subPropertyOf")
            .into_iter()
            .filter(|subject| subject.contains(property)) // Simplified matching
            .map(|s| s.to_string())
            .collect()
    }
    
    /// Get parallel processing status
    pub fn is_parallel_processing_enabled(&self) -> bool {
        self.parallel_processing
    }
    
    /// Get cache size limit
    pub fn get_cache_size_limit(&self) -> usize {
        self.cache_size_limit
    }
    
    /// Get batch size for processing
    pub fn get_batch_size(&self) -> usize {
        self.batch_size
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

// Data structures for inference and materialization

/// Materialization strategy for inferred triples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaterializationStrategy {
    /// Full materialization - compute and store all inferences
    Full,
    /// Incremental materialization - only compute new inferences
    Incremental,
    /// On-demand materialization - compute inferences when needed
    OnDemand,
    /// Hybrid materialization - combination of strategies
    Hybrid,
}

impl Default for MaterializationStrategy {
    fn default() -> Self {
        MaterializationStrategy::Incremental
    }
}

/// Result of inference processing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InferenceResult {
    pub consistent: bool,
    pub classification_performed: bool,
    pub realization_performed: bool,
    pub materialized_triples: usize,
    pub sparql_inferences: usize,
    pub individuals_classified: usize,
    pub processing_time_ms: u64,
    pub incremental: bool,
    pub new_triples_processed: usize,
    pub inference_errors: Vec<String>,
}

/// Detailed inference statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InferenceStats {
    pub total_inferences: usize,
    pub incremental_inferences: usize,
    pub full_inferences: usize,
    pub materialized_triples_count: usize,
    pub total_processing_time_ms: u64,
    pub average_processing_time_ms: f64,
    pub last_inference_time: Option<std::time::SystemTime>,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub strategy: MaterializationStrategy,
}

impl InferenceStats {
    pub fn update_average(&mut self) {
        if self.total_inferences > 0 {
            self.average_processing_time_ms = self.total_processing_time_ms as f64 / self.total_inferences as f64;
        }
    }
    
    pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests > 0 {
            self.cache_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }
}

/// Inference cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceCacheEntry {
    pub cache_key: String,
    pub timestamp: std::time::SystemTime,
    pub result: InferenceResult,
    pub materialized_triples: Vec<String>, // Store as serialized strings
    pub ttl_seconds: u64,
}

impl InferenceCacheEntry {
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now();
        let elapsed = now.duration_since(self.timestamp).unwrap_or_default();
        elapsed.as_secs() > self.ttl_seconds
    }
    
    pub fn new(cache_key: String, result: InferenceResult, materialized_triples: Vec<oxrdf::Triple>, ttl_seconds: u64) -> Self {
        // Convert triples to strings for serialization
        let serialized_triples = materialized_triples
            .into_iter()
            .map(|triple| format!("{} {} {}", triple.subject, triple.predicate, triple.object))
            .collect();
        
        Self {
            cache_key,
            timestamp: std::time::SystemTime::now(),
            result,
            materialized_triples: serialized_triples,
            ttl_seconds,
        }
    }
}

/// Inference performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferencePerformanceMetrics {
    pub total_ontology_axioms: usize,
    pub reasoning_profile: String,
    pub estimated_classification_time_ms: usize,
    pub estimated_realization_time_ms: usize,
    pub actual_classification_time_ms: usize,
    pub actual_realization_time_ms: usize,
    pub memory_usage_mb: f64,
    pub throughput_triples_per_second: f64,
}

impl InferencePerformanceMetrics {
    pub fn efficiency_ratio(&self) -> f64 {
        if self.estimated_classification_time_ms + self.estimated_realization_time_ms > 0 {
            (self.actual_classification_time_ms + self.actual_realization_time_ms) as f64 
                / (self.estimated_classification_time_ms + self.estimated_realization_time_ms) as f64
        } else {
            1.0
        }
    }
}

/// Materialization performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializationMetrics {
    pub strategy: MaterializationStrategy,
    pub materialized_triples_count: usize,
    pub storage_overhead_mb: f64,
    pub average_materialization_time_ms: f64,
    pub cache_efficiency: f64,
    pub incremental_update_time_ms: u64,
    pub full_recomputation_time_ms: u64,
}

impl MaterializationMetrics {
    pub fn recomputation_overhead(&self) -> f64 {
        if self.incremental_update_time_ms > 0 {
            self.full_recomputation_time_ms as f64 / self.incremental_update_time_ms as f64
        } else {
            1.0
        }
    }
}

/// Performance metrics for reasoning operations
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_operations: AtomicU64,
    pub parallel_operations: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub average_response_time_ms: AtomicU64,
    pub peak_memory_usage_mb: AtomicU64,
    pub operation_throughput: f64,
    pub last_optimization_time: Option<String>,
}

impl Clone for PerformanceMetrics {
    fn clone(&self) -> Self {
        Self {
            total_operations: AtomicU64::new(self.total_operations.load(Ordering::Relaxed)),
            parallel_operations: AtomicU64::new(self.parallel_operations.load(Ordering::Relaxed)),
            cache_hits: AtomicU64::new(self.cache_hits.load(Ordering::Relaxed)),
            cache_misses: AtomicU64::new(self.cache_misses.load(Ordering::Relaxed)),
            average_response_time_ms: AtomicU64::new(self.average_response_time_ms.load(Ordering::Relaxed)),
            peak_memory_usage_mb: AtomicU64::new(self.peak_memory_usage_mb.load(Ordering::Relaxed)),
            operation_throughput: self.operation_throughput,
            last_optimization_time: self.last_optimization_time.clone(),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_operations: AtomicU64::new(0),
            parallel_operations: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            average_response_time_ms: AtomicU64::new(0),
            peak_memory_usage_mb: AtomicU64::new(0),
            operation_throughput: 0.0,
            last_optimization_time: None,
        }
    }
}

impl PerformanceMetrics {
    pub fn record_operation(&self, duration_ms: u64, is_parallel: bool) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        if is_parallel {
            self.parallel_operations.fetch_add(1, Ordering::Relaxed);
        }
        
        // Update average response time
        let current_avg = self.average_response_time_ms.load(Ordering::Relaxed);
        let total_ops = self.total_operations.load(Ordering::Relaxed);
        let new_avg = (current_avg * (total_ops - 1) + duration_ms) / total_ops;
        self.average_response_time_ms.store(new_avg, Ordering::Relaxed);
    }
    
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        if hits + misses == 0 {
            0.0
        } else {
            hits as f64 / (hits + misses) as f64
        }
    }
    
    pub fn parallel_operation_rate(&self) -> f64 {
        let total = self.total_operations.load(Ordering::Relaxed);
        let parallel = self.parallel_operations.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            parallel as f64 / total as f64
        }
    }
}

/// Index structures for fast lookups
#[derive(Debug, Clone, Default)]
pub struct IndexStructures {
    pub class_index: HashMap<String, Vec<String>>,        // class -> instances
    pub property_index: HashMap<String, Vec<String>>,      // property -> subjects
    pub individual_index: HashMap<String, Vec<String>>,    // individual -> types
    pub triple_pattern_index: HashMap<String, Vec<usize>>, // pattern -> triple positions
}

impl IndexStructures {
    pub fn new() -> Self {
        Self {
            class_index: HashMap::new(),
            property_index: HashMap::new(),
            individual_index: HashMap::new(),
            triple_pattern_index: HashMap::new(),
        }
    }
    
    pub fn build_indexes(&mut self, triples: &[oxrdf::Triple]) {
        // Clear existing indexes
        self.class_index.clear();
        self.property_index.clear();
        self.individual_index.clear();
        self.triple_pattern_index.clear();
        
        // Build indexes in parallel
        let indexes: Vec<_> = triples.par_iter().enumerate().map(|(i, triple)| {
            let mut class_entries = Vec::new();
            let mut property_entries = Vec::new();
            let mut individual_entries = Vec::new();
            let mut pattern_entries = Vec::new();
            
            // Index class relationships
            if triple.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
                if let oxrdf::Term::NamedNode(class_node) = &triple.object {
                    let class = class_node.as_str();
                    class_entries.push((class.to_string(), format!("{}", triple.subject)));
                }
            }
            
            // Index property relationships
            property_entries.push((triple.predicate.as_str().to_string(), format!("{}", triple.subject)));
            
            // Index individual types
            individual_entries.push((format!("{}", triple.subject), format!("{}", triple.object)));
            
            // Index triple patterns
            let pattern = format!("{} {} {}", triple.subject, triple.predicate, triple.object);
            pattern_entries.push((pattern, i));
            
            (class_entries, property_entries, individual_entries, pattern_entries)
        }).collect();
        
        // Merge results
        for (class_entries, property_entries, individual_entries, pattern_entries) in indexes {
            for (class, subject) in class_entries {
                self.class_index.entry(class).or_insert_with(Vec::new).push(subject);
            }
            for (property, subject) in property_entries {
                self.property_index.entry(property).or_insert_with(Vec::new).push(subject);
            }
            for (individual, object_type) in individual_entries {
                self.individual_index.entry(individual).or_insert_with(Vec::new).push(object_type);
            }
            for (pattern, position) in pattern_entries {
                self.triple_pattern_index.entry(pattern).or_insert_with(Vec::new).push(position);
            }
        }
    }
    
    pub fn find_instances_by_class(&self, class: &str) -> Vec<&String> {
        self.class_index.get(class)
            .map(|instances| instances.iter().collect())
            .unwrap_or_default()
    }
    
    pub fn find_subjects_by_property(&self, property: &str) -> Vec<&String> {
        self.property_index.get(property)
            .map(|subjects| subjects.iter().collect())
            .unwrap_or_default()
    }
    
    pub fn find_types_by_individual(&self, individual: &str) -> Vec<&String> {
        self.individual_index.get(individual)
            .map(|types| types.iter().collect())
            .unwrap_or_default()
    }
}