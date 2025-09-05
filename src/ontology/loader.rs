use crate::EpcisKgError;
use crate::Config;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use oxrdf::{Graph, NamedNodeRef, TermRef};
use oxttl::TurtleParser;

/// Represents loaded ontology data with parsing statistics
pub struct OntologyData {
    pub graph: Graph,
    pub triples_count: usize,
    pub source_file: String,
}

pub struct OntologyLoader {
    config: Config,
}

impl OntologyLoader {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
    
    pub fn with_config(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }
    
    /// Load ontology from a Turtle file
    pub fn load_ontology<P: AsRef<Path>>(&self, path: P) -> Result<OntologyData, EpcisKgError> {
        let path = path.as_ref();
        let source_file = path.to_string_lossy().to_string();
        
        // Open and read the file
        let mut file = File::open(path)
            .map_err(|e| EpcisKgError::Ontology(format!("Failed to open ontology file: {}", e)))?;
        
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| EpcisKgError::Ontology(format!("Failed to read ontology file: {}", e)))?;
        
        self.parse_turtle_content(&content, source_file)
    }
    
    /// Load multiple ontology files
    pub fn load_ontologies<P: AsRef<Path>>(&self, paths: &[P]) -> Result<Vec<OntologyData>, EpcisKgError> {
        let mut results = Vec::new();
        
        for path in paths {
            let ontology_data = self.load_ontology(path)?;
            results.push(ontology_data);
        }
        
        Ok(results)
    }
    
    /// Parse Turtle content from bytes
    fn parse_turtle_content(&self, content: &[u8], source_file: String) -> Result<OntologyData, EpcisKgError> {
        let mut graph = Graph::default();
        let mut triples_count = 0;
        
        // Parse Turtle content using oxttl
        let parser = TurtleParser::new();
        let reader = std::io::Cursor::new(content);
        for triple_result in parser.for_reader(reader) {
            let triple = triple_result
                .map_err(|e| EpcisKgError::Ontology(format!("Turtle parsing error: {}", e)))?;
            
            // The triple from oxttl is already in the correct format for oxrdf
            graph.insert(triple.as_ref());
            triples_count += 1;
        }
        
        Ok(OntologyData {
            graph,
            triples_count,
            source_file,
        })
    }
    
    /// Load EPCIS 2.0 ontology from default location
    pub async fn load_epcis(&self) -> Result<OntologyData, EpcisKgError> {
        // Look for EPCIS ontology in standard locations
        let possible_paths = vec![
            "ontologies/epcis2.ttl",
            "ontologies/epcis.ttl",
            "epcis2.ttl",
        ];
        
        for path_str in possible_paths {
            let path = Path::new(path_str);
            if path.exists() {
                let ontology_data = self.load_ontology(path)?;
                
                // Validate that this is actually an EPCIS ontology
                self.validate_epcis_structure(&ontology_data)?;
                
                return Ok(ontology_data);
            }
        }
        
        Err(EpcisKgError::Ontology(
            "EPCIS ontology file not found. Expected at ontologies/epcis2.ttl".to_string(),
        ))
    }
    
    /// Get EPCIS-specific vocabulary information from loaded ontology
    pub fn get_epcis_vocabulary(&self, ontology_data: &OntologyData) -> EpcisVocabulary {
        let mut event_types = Vec::new();
        let mut business_steps = Vec::new();
        let mut dispositions = Vec::new();
        let mut properties = Vec::new();
        
        let rdf_type = NamedNodeRef::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap();
        let rdfs_class = NamedNodeRef::new("http://www.w3.org/2000/01/rdf-schema#Class").unwrap();
        let owl_class = NamedNodeRef::new("http://www.w3.org/2002/07/owl#Class").unwrap();
        let epcis_event = NamedNodeRef::new("urn:epcglobal:epcis:Event").unwrap();
        let epcis_object_event = NamedNodeRef::new("urn:epcglobal:epcis:ObjectEvent").unwrap();
        let epcis_aggregation_event = NamedNodeRef::new("urn:epcglobal:epcis:AggregationEvent").unwrap();
        let epcis_quantity_event = NamedNodeRef::new("urn:epcglobal:epcis:QuantityEvent").unwrap();
        let epcis_transaction_event = NamedNodeRef::new("urn:epcglobal:epcis:TransactionEvent").unwrap();
        let epcis_transformation_event = NamedNodeRef::new("urn:epcglobal:epcis:TransformationEvent").unwrap();
        let cbv_biz_step = NamedNodeRef::new("urn:epcglobal:cbv:BizStep").unwrap();
        let cbv_disposition = NamedNodeRef::new("urn:epcglobal:cbv:Disposition").unwrap();
        
        for triple in ontology_data.graph.iter() {
            if triple.predicate == rdf_type {
                match (triple.subject, triple.object) {
                    (oxrdf::SubjectRef::NamedNode(subject), oxrdf::TermRef::NamedNode(object)) => {
                        // Check for EPCIS event types and CBV vocabulary
                        if object == epcis_event {
                            event_types.push(EpcisEventType::Event(subject.as_str().to_string()));
                        } else if object == epcis_object_event {
                            event_types.push(EpcisEventType::ObjectEvent(subject.as_str().to_string()));
                        } else if object == epcis_aggregation_event {
                            event_types.push(EpcisEventType::AggregationEvent(subject.as_str().to_string()));
                        } else if object == epcis_quantity_event {
                            event_types.push(EpcisEventType::QuantityEvent(subject.as_str().to_string()));
                        } else if object == epcis_transaction_event {
                            event_types.push(EpcisEventType::TransactionEvent(subject.as_str().to_string()));
                        } else if object == epcis_transformation_event {
                            event_types.push(EpcisEventType::TransformationEvent(subject.as_str().to_string()));
                        } else if object == cbv_biz_step {
                            business_steps.push(subject.as_str().to_string());
                        } else if object == cbv_disposition {
                            dispositions.push(subject.as_str().to_string());
                        }
                    }
                    _ => {}
                }
            } else {
                // Extract EPCIS properties
                if triple.predicate.as_str().starts_with("urn:epcglobal:epcis:") {
                    properties.push(triple.predicate.as_str().to_string());
                }
            }
        }
        
        EpcisVocabulary {
            event_types,
            business_steps,
            dispositions,
            properties,
            total_triples: ontology_data.triples_count,
        }
    }
    
    /// Load CBV (Core Business Vocabulary) ontology from default location
    pub async fn load_cbv(&self) -> Result<OntologyData, EpcisKgError> {
        // Look for CBV ontology in standard locations
        let possible_paths = vec![
            "ontologies/cbv.ttl",
            "ontologies/cbv_core.ttl",
            "cbv.ttl",
        ];
        
        for path_str in possible_paths {
            let path = Path::new(path_str);
            if path.exists() {
                return self.load_ontology(path);
            }
        }
        
        Err(EpcisKgError::Ontology(
            "CBV ontology file not found. Expected at ontologies/cbv.ttl".to_string(),
        ))
    }
    
    /// Get CBV-specific vocabulary information from loaded ontology
    pub fn get_cbv_vocabulary(&self, ontology_data: &OntologyData) -> CbvVocabulary {
        let mut business_steps = Vec::new();
        let mut dispositions = Vec::new();
        let mut business_locations = Vec::new();
        let mut business_transactions = Vec::new();
        let mut sensor_readings = Vec::new();
        let mut actions = Vec::new();
        
        let rdf_type = NamedNodeRef::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap();
        let rdfs_class = NamedNodeRef::new("http://www.w3.org/2000/01/rdf-schema#Class").unwrap();
        let owl_class = NamedNodeRef::new("http://www.w3.org/2002/07/owl#Class").unwrap();
        let cbv_biz_step = NamedNodeRef::new("urn:epcglobal:cbv:BizStep").unwrap();
        let cbv_disposition = NamedNodeRef::new("urn:epcglobal:cbv:Disposition").unwrap();
        let cbv_business_location = NamedNodeRef::new("urn:epcglobal:cbv:BusinessLocation").unwrap();
        let cbv_business_transaction = NamedNodeRef::new("urn:epcglobal:cbv:BusinessTransaction").unwrap();
        let cbv_sensor_reading = NamedNodeRef::new("urn:epcglobal:cbv:SensorReading").unwrap();
        let epcis_action = NamedNodeRef::new("urn:epcglobal:epcis:action").unwrap();
        
        for triple in ontology_data.graph.iter() {
            if triple.predicate == rdf_type {
                match (triple.subject, triple.object) {
                    (oxrdf::SubjectRef::NamedNode(subject), oxrdf::TermRef::NamedNode(object)) => {
                        // Check for CBV vocabulary types
                        if object == cbv_biz_step {
                            business_steps.push(subject.as_str().to_string());
                        } else if object == cbv_disposition {
                            dispositions.push(subject.as_str().to_string());
                        } else if object == cbv_business_location {
                            business_locations.push(subject.as_str().to_string());
                        } else if object == cbv_business_transaction {
                            business_transactions.push(subject.as_str().to_string());
                        } else if object == cbv_sensor_reading {
                            sensor_readings.push(subject.as_str().to_string());
                        } else if object == epcis_action {
                            actions.push(subject.as_str().to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        
        CbvVocabulary {
            business_steps,
            dispositions,
            business_locations,
            business_transactions,
            sensor_readings,
            actions,
            total_triples: ontology_data.triples_count,
        }
    }
    
    /// Validate that the loaded ontology contains expected EPCIS vocabulary
    pub fn validate_epcis_structure(&self, ontology_data: &OntologyData) -> Result<(), EpcisKgError> {
        let epcis_uri = NamedNodeRef::new("urn:epcglobal:epcis:").unwrap();
        let rdf_type = NamedNodeRef::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap();
        let object_event_class = NamedNodeRef::new("urn:epcglobal:epcis:ObjectEvent").unwrap();
        
        // Check for ObjectEvent class definition
        let has_object_event = ontology_data.graph.iter().any(|triple| {
            triple.subject == oxrdf::SubjectRef::NamedNode(object_event_class) && 
            triple.predicate == rdf_type
        });
        
        if !has_object_event {
            return Err(EpcisKgError::Validation(
                "EPCIS ontology missing ObjectEvent class definition".to_string(),
            ));
        }
        
        // Check for EPCIS namespace usage
        let has_epcis_namespace = ontology_data.graph.iter().any(|triple| {
            match (triple.subject, triple.predicate, triple.object) {
                (oxrdf::SubjectRef::NamedNode(subject), _, _) => 
                    subject.as_str().starts_with(epcis_uri.as_str()),
                (_, _, oxrdf::TermRef::NamedNode(object)) => 
                    object.as_str().starts_with(epcis_uri.as_str()),
                (_, predicate, _) => 
                    predicate.as_str().starts_with(epcis_uri.as_str()),
            }
        });
        
        if !has_epcis_namespace {
            return Err(EpcisKgError::Validation(
                "EPCIS ontology does not use EPCIS namespace".to_string(),
            ));
        }
        
        Ok(())
    }
    
    /// Get statistics about loaded ontology
    pub fn get_statistics(&self, ontology_data: &OntologyData) -> OntologyStats {
        let mut classes = 0;
        let mut properties = 0;
        let mut individuals = 0;
        
        let rdf_type = NamedNodeRef::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap();
        let rdfs_class = NamedNodeRef::new("http://www.w3.org/2000/01/rdf-schema#Class").unwrap();
        let rdf_property = NamedNodeRef::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#Property").unwrap();
        let owl_class = NamedNodeRef::new("http://www.w3.org/2002/07/owl#Class").unwrap();
        let owl_object_property = NamedNodeRef::new("http://www.w3.org/2002/07/owl#ObjectProperty").unwrap();
        let owl_datatype_property = NamedNodeRef::new("http://www.w3.org/2002/07/owl#DatatypeProperty").unwrap();
        
        for triple in ontology_data.graph.iter() {
            if triple.predicate == rdf_type {
                match triple.object {
                    TermRef::NamedNode(obj) if obj == rdfs_class || obj == owl_class => classes += 1,
                    TermRef::NamedNode(obj) if obj == rdf_property || obj == owl_object_property || obj == owl_datatype_property => properties += 1,
                    _ => individuals += 1,
                }
            }
        }
        
        OntologyStats {
            total_triples: ontology_data.triples_count,
            classes,
            properties,
            individuals,
            source_file: ontology_data.source_file.clone(),
        }
    }
}

/// Statistics about loaded ontology
#[derive(Debug, Clone)]
pub struct OntologyStats {
    pub total_triples: usize,
    pub classes: usize,
    pub properties: usize,
    pub individuals: usize,
    pub source_file: String,
}

/// EPCIS event types
#[derive(Debug, Clone, PartialEq)]
pub enum EpcisEventType {
    Event(String),
    ObjectEvent(String),
    AggregationEvent(String),
    QuantityEvent(String),
    TransactionEvent(String),
    TransformationEvent(String),
}

/// EPCIS vocabulary information extracted from ontology
#[derive(Debug, Clone)]
pub struct EpcisVocabulary {
    pub event_types: Vec<EpcisEventType>,
    pub business_steps: Vec<String>,
    pub dispositions: Vec<String>,
    pub properties: Vec<String>,
    pub total_triples: usize,
}

/// CBV vocabulary information extracted from ontology
#[derive(Debug, Clone)]
pub struct CbvVocabulary {
    pub business_steps: Vec<String>,
    pub dispositions: Vec<String>,
    pub business_locations: Vec<String>,
    pub business_transactions: Vec<String>,
    pub sensor_readings: Vec<String>,
    pub actions: Vec<String>,
    pub total_triples: usize,
}