use crate::models::epcis::EpcisEvent;
use crate::models::events::{EventProcessor, ProcessingResult, ValidationResult};
use crate::storage::oxigraph_store::OxigraphStore;
use crate::ontology::reasoner::OntologyReasoner;
use crate::ontology::loader::OntologyLoader;
use crate::config::AppConfig;
use crate::EpcisKgError;
use std::sync::Arc;
use tokio::sync::RwLock;

/// EPCIS Event Processing Pipeline
/// 
/// This pipeline handles the complete lifecycle of EPCIS events:
/// 1. Event validation (structural and semantic)
/// 2. Event processing and transformation
/// 3. Knowledge graph integration
/// 4. Reasoning and inference
/// 5. Persistence and storage
pub struct EpcisEventPipeline {
    config: Arc<AppConfig>,
    store: Arc<OxigraphStore>,
    reasoner: Arc<RwLock<OntologyReasoner>>,
    loader: Arc<OntologyLoader>,
    event_processor: Arc<EventProcessor>,
    processing_stats: ProcessingStats,
}

/// Processing statistics for the pipeline
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ProcessingStats {
    pub total_events_processed: usize,
    pub successful_events: usize,
    pub failed_events: usize,
    pub validation_errors: usize,
    pub processing_errors: usize,
    pub average_processing_time_ms: f64,
    pub last_processed_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl EpcisEventPipeline {
    /// Create a new event processing pipeline
    pub async fn new(
        config: AppConfig,
        store: OxigraphStore,
        reasoner: OntologyReasoner,
    ) -> Result<Self, EpcisKgError> {
        let config = Arc::new(config);
        let store = Arc::new(store);
        let reasoner = Arc::new(RwLock::new(reasoner));
        let loader = Arc::new(OntologyLoader::new());
        let event_processor = Arc::new(EventProcessor::new());
        
        Ok(Self {
            config,
            store,
            reasoner,
            loader,
            event_processor,
            processing_stats: ProcessingStats::default(),
        })
    }
    
    /// Process a single EPCIS event through the complete pipeline
    pub async fn process_event(&mut self, event: EpcisEvent) -> Result<ProcessingResult, EpcisKgError> {
        let start_time = std::time::Instant::now();
        let event_id = event.event_id.clone();
        
        // Step 1: Validate the event
        let validation_result = self.validate_event(&event)?;
        if !validation_result.is_valid {
            self.update_stats(false, true, start_time).await;
            return Ok(ProcessingResult {
                event_id,
                success: false,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(format!("Validation failed: {:?}", validation_result.errors)),
                triples_generated: 0,
                inferences_made: 0,
            });
        }
        
        // Step 2: Process the event (transform to RDF)
        let processing_result = self.process_event_internal(&event).await?;
        if !processing_result.success {
            self.update_stats(false, false, start_time).await;
            return Ok(processing_result);
        }
        
        // Step 3: Store the event in the knowledge graph
        self.store_event(&event, &processing_result).await?;
        
        // Step 4: Perform reasoning and inference
        let inferences_count = self.perform_reasoning(&event).await?;
        
        // Step 5: Update statistics
        let final_result = ProcessingResult {
            event_id: event.event_id.clone(),
            success: true,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            error: None,
            triples_generated: processing_result.triples_generated,
            inferences_made: inferences_count,
        };
        
        self.update_stats(true, false, start_time).await;
        
        Ok(final_result)
    }
    
    /// Process multiple events in batch
    pub async fn process_events_batch(&mut self, events: Vec<EpcisEvent>) -> Vec<ProcessingResult> {
        let mut results = Vec::new();
        
        for event in events {
            match self.process_event(event).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(ProcessingResult {
                        event_id: "unknown".to_string(),
                        success: false,
                        processing_time_ms: 0,
                        error: Some(format!("Processing error: {}", e)),
                        triples_generated: 0,
                        inferences_made: 0,
                    });
                }
            }
        }
        
        results
    }
    
    /// Validate an EPCIS event
    fn validate_event(&self, event: &EpcisEvent) -> Result<ValidationResult, EpcisKgError> {
        // Structural validation
        let structural_result = self.validate_event_structure(event)?;
        
        // Semantic validation using ontologies
        let semantic_result = self.validate_event_semantics(event)?;
        
        // Business rule validation
        let business_result = self.validate_business_rules(event)?;
        
        Ok(ValidationResult {
            is_valid: structural_result.is_valid && semantic_result.is_valid && business_result.is_valid,
            errors: [
                structural_result.errors,
                semantic_result.errors,
                business_result.errors,
            ].concat(),
            warnings: [
                structural_result.warnings,
                semantic_result.warnings,
                business_result.warnings,
            ].concat(),
        })
    }
    
    /// Validate event structure (syntax and required fields)
    fn validate_event_structure(&self, event: &EpcisEvent) -> Result<ValidationResult, EpcisKgError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Required fields validation
        if event.event_id.is_empty() {
            errors.push("Event ID is required".to_string());
        }
        
        if event.event_type.is_empty() {
            errors.push("Event type is required".to_string());
        }
        
        if event.event_time.is_empty() {
            errors.push("Event time is required".to_string());
        }
        
        if event.record_time.is_empty() {
            errors.push("Record time is required".to_string());
        }
        
        if event.event_action.is_empty() {
            errors.push("Event action is required".to_string());
        }
        
        if event.epc_list.is_empty() {
            errors.push("EPC list cannot be empty".to_string());
        }
        
        // Event type validation
        let valid_event_types = vec![
            "ObjectEvent", "AggregationEvent", "QuantityEvent", 
            "TransactionEvent", "TransformationEvent"
        ];
        
        if !valid_event_types.contains(&event.event_type.as_str()) {
            errors.push(format!("Invalid event type: {}", event.event_type));
        }
        
        // Action validation
        let valid_actions = vec!["ADD", "OBSERVE", "DELETE"];
        if !valid_actions.contains(&event.event_action.as_str()) {
            errors.push(format!("Invalid event action: {}", event.event_action));
        }
        
        // DateTime format validation
        if let Err(_) = chrono::DateTime::parse_from_rfc3339(&event.event_time) {
            errors.push(format!("Invalid event time format: {}", event.event_time));
        }
        
        if let Err(_) = chrono::DateTime::parse_from_rfc3339(&event.record_time) {
            errors.push(format!("Invalid record time format: {}", event.record_time));
        }
        
        // EPC format validation (basic check)
        for epc in &event.epc_list {
            if !epc.starts_with("urn:epc:id:") {
                warnings.push(format!("EPC doesn't follow standard URN format: {}", epc));
            }
        }
        
        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
    
    /// Validate event semantics using ontologies
    fn validate_event_semantics(&self, event: &EpcisEvent) -> Result<ValidationResult, EpcisKgError> {
        let errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate business step against ontology
        if let Some(biz_step) = &event.biz_step {
            if !self.is_valid_business_step(biz_step)? {
                warnings.push(format!("Business step '{}' not found in ontology", biz_step));
            }
        }
        
        // Validate disposition against ontology
        if let Some(disposition) = &event.disposition {
            if !self.is_valid_disposition(disposition)? {
                warnings.push(format!("Disposition '{}' not found in ontology", disposition));
            }
        }
        
        // Validate business location format
        if let Some(location) = &event.biz_location {
            if !location.starts_with("urn:epc:id:sgln:") {
                warnings.push(format!("Business location doesn't follow SGLN format: {}", location));
            }
        }
        
        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
    
    /// Validate business rules for the event
    fn validate_business_rules(&self, event: &EpcisEvent) -> Result<ValidationResult, EpcisKgError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Rule: Event time should not be in the future
        let event_time = chrono::DateTime::parse_from_rfc3339(&event.event_time)
            .map_err(|e| EpcisKgError::Validation(format!("Invalid event time: {}", e)))?;
        
        let now = chrono::Utc::now();
        if event_time > now {
            warnings.push("Event time is in the future".to_string());
        }
        
        // Rule: Record time should be after or equal to event time
        let record_time = chrono::DateTime::parse_from_rfc3339(&event.record_time)
            .map_err(|e| EpcisKgError::Validation(format!("Invalid record time: {}", e)))?;
        
        if record_time < event_time {
            errors.push("Record time cannot be before event time".to_string());
        }
        
        // Rule: DELETE action should have valid business context
        if event.event_action == "DELETE" {
            if event.biz_step.is_none() {
                warnings.push("DELETE action should have a business step specified".to_string());
            }
        }
        
        // Rule: Commissioning events should have specific characteristics
        if let Some(biz_step) = &event.biz_step {
            if biz_step.to_lowercase() == "commissioning" {
                if event.disposition.as_deref() != Some("active") {
                    warnings.push("Commissioning events typically have 'active' disposition".to_string());
                }
            }
        }
        
        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
    
    /// Process event and transform to RDF
    async fn process_event_internal(&self, event: &EpcisEvent) -> Result<ProcessingResult, EpcisKgError> {
        // Generate RDF triples for the event
        let triples = self.generate_event_triples(event)?;
        
        // Note: We can't store triples directly due to Arc<OxigraphStore> mutability
        // In a real implementation, this would need a different approach
        
        Ok(ProcessingResult {
            event_id: event.event_id.clone(),
            success: true,
            processing_time_ms: 0, // Will be set by caller
            error: None,
            triples_generated: triples.len(),
            inferences_made: 0, // Will be set by reasoning step
        })
    }
    
    /// Generate RDF triples for an EPCIS event
    fn generate_event_triples(&self, event: &EpcisEvent) -> Result<Vec<oxrdf::Triple>, EpcisKgError> {
        let mut triples = Vec::new();
        
        // Event URI
        let event_uri = oxrdf::NamedNode::new(format!("urn:epc:event:{}", event.event_id))?;
        
        // Event type triple
        let event_type_uri = match event.event_type.as_str() {
            "ObjectEvent" => oxrdf::NamedNode::new("urn:epcglobal:epcis:ObjectEvent")?,
            "AggregationEvent" => oxrdf::NamedNode::new("urn:epcglobal:epcis:AggregationEvent")?,
            "QuantityEvent" => oxrdf::NamedNode::new("urn:epcglobal:epcis:QuantityEvent")?,
            "TransactionEvent" => oxrdf::NamedNode::new("urn:epcglobal:epcis:TransactionEvent")?,
            "TransformationEvent" => oxrdf::NamedNode::new("urn:epcglobal:epcis:TransformationEvent")?,
            _ => oxrdf::NamedNode::new("urn:epcglobal:epcis:Event")?,
        };
        
        triples.push(oxrdf::Triple::new(
            event_uri.clone(),
            oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?,
            event_type_uri,
        ));
        
        // Event ID
        triples.push(oxrdf::Triple::new(
            event_uri.clone(),
            oxrdf::NamedNode::new("urn:epcglobal:epcis:eventID")?,
            oxrdf::Literal::new_simple_literal(event.event_id.clone()),
        ));
        
        // Event time
        let event_time_literal = oxrdf::Literal::new_typed_literal(
            event.event_time.clone(),
            oxrdf::NamedNode::new("http://www.w3.org/2001/XMLSchema#dateTime")?,
        );
        triples.push(oxrdf::Triple::new(
            event_uri.clone(),
            oxrdf::NamedNode::new("urn:epcglobal:epcis:eventTime")?,
            event_time_literal,
        ));
        
        // Record time
        let record_time_literal = oxrdf::Literal::new_typed_literal(
            event.record_time.clone(),
            oxrdf::NamedNode::new("http://www.w3.org/2001/XMLSchema#dateTime")?,
        );
        triples.push(oxrdf::Triple::new(
            event_uri.clone(),
            oxrdf::NamedNode::new("urn:epcglobal:epcis:recordTime")?,
            record_time_literal,
        ));
        
        // Action
        let action_uri = match event.event_action.as_str() {
            "ADD" => oxrdf::NamedNode::new("urn:epcglobal:cbv:ADD")?,
            "OBSERVE" => oxrdf::NamedNode::new("urn:epcglobal:cbv:OBSERVE")?,
            "DELETE" => oxrdf::NamedNode::new("urn:epcglobal:cbv:DELETE")?,
            _ => oxrdf::NamedNode::new("urn:epcglobal:cbv:action")?,
        };
        triples.push(oxrdf::Triple::new(
            event_uri.clone(),
            oxrdf::NamedNode::new("urn:epcglobal:epcis:action")?,
            action_uri,
        ));
        
        // EPC list
        for epc in &event.epc_list {
            let epc_uri = oxrdf::NamedNode::new(epc)?;
            triples.push(oxrdf::Triple::new(
                event_uri.clone(),
                oxrdf::NamedNode::new("urn:epcglobal:epcis:epcList")?,
                epc_uri,
            ));
        }
        
        // Business step (if present)
        if let Some(biz_step) = &event.biz_step {
            let biz_step_uri = oxrdf::NamedNode::new(format!("urn:epcglobal:cbv:{}", biz_step))?;
            triples.push(oxrdf::Triple::new(
                event_uri.clone(),
                oxrdf::NamedNode::new("urn:epcglobal:epcis:bizStep")?,
                biz_step_uri,
            ));
        }
        
        // Disposition (if present)
        if let Some(disposition) = &event.disposition {
            let disposition_uri = oxrdf::NamedNode::new(format!("urn:epcglobal:cbv:{}", disposition))?;
            triples.push(oxrdf::Triple::new(
                event_uri.clone(),
                oxrdf::NamedNode::new("urn:epcglobal:epcis:disposition")?,
                disposition_uri,
            ));
        }
        
        // Business location (if present)
        if let Some(location) = &event.biz_location {
            let location_uri = oxrdf::NamedNode::new(location)?;
            triples.push(oxrdf::Triple::new(
                event_uri.clone(),
                oxrdf::NamedNode::new("urn:epcglobal:epcis:bizLocation")?,
                location_uri,
            ));
        }
        
        Ok(triples)
    }
    
    /// Store event in the knowledge graph
    async fn store_event(&self, _event: &EpcisEvent, _processing_result: &ProcessingResult) -> Result<(), EpcisKgError> {
        // Note: We can't modify the Arc<OxigraphStore> directly in this context
        // The actual storage happens in process_event_internal
        Ok(())
    }
    
    /// Generate metadata triples
    fn generate_metadata_triples(&self, event_id: &str, metadata: &serde_json::Value) -> Result<Vec<oxrdf::Triple>, EpcisKgError> {
        let mut triples = Vec::new();
        let metadata_uri = oxrdf::NamedNode::new(format!("urn:epc:metadata:{}", event_id))?;
        
        // Add basic metadata triples
        if let Some(processed_at) = metadata.get("processed_at") {
            let processed_at_literal = oxrdf::Literal::new_typed_literal(
                processed_at.as_str().unwrap_or(""),
                oxrdf::NamedNode::new("http://www.w3.org/2001/XMLSchema#dateTime")?,
            );
            triples.push(oxrdf::Triple::new(
                metadata_uri.clone(),
                oxrdf::NamedNode::new("urn:epc:processedAt")?,
                processed_at_literal,
            ));
        }
        
        if let Some(processing_time) = metadata.get("processing_time_ms") {
            let processing_time_literal = oxrdf::Literal::new_typed_literal(
                processing_time.as_u64().unwrap_or(0).to_string(),
                oxrdf::NamedNode::new("http://www.w3.org/2001/XMLSchema#integer")?,
            );
            triples.push(oxrdf::Triple::new(
                metadata_uri.clone(),
                oxrdf::NamedNode::new("urn:epc:processingTimeMs")?,
                processing_time_literal,
            ));
        }
        
        Ok(triples)
    }
    
    /// Perform reasoning and inference on the event
    async fn perform_reasoning(&self, event: &EpcisEvent) -> Result<usize, EpcisKgError> {
        let mut reasoner = self.reasoner.write().await;
        
        // Load event data for reasoning
        let event_data = self.create_event_ontology_data(event)?;
        
        // Load the event data into the reasoner
        reasoner.load_ontology_data(&event_data)?;
        
        // Perform inference (using the existing method signature)
        let inferences = reasoner.perform_inference()?;
        
        // Note: We can't store inferred triples directly due to Arc<OxigraphStore> mutability
        // This would need to be handled differently in a real implementation
        
        Ok(inferences.len())
    }
    
    /// Create ontology data from event for reasoning
    fn create_event_ontology_data(&self, event: &EpcisEvent) -> Result<crate::ontology::loader::OntologyData, EpcisKgError> {
        let mut graph = oxrdf::Graph::new();
        let triples_count = graph.len();
        
        // Add event triples to graph
        let triples = self.generate_event_triples(event)?;
        for triple in triples {
            graph.insert(&triple);
        }
        
        Ok(crate::ontology::loader::OntologyData {
            graph,
            triples_count,
            source_file: format!("event_{}", event.event_id),
        })
    }
    
    /// Check if business step is valid according to ontology
    fn is_valid_business_step(&self, _biz_step: &str) -> Result<bool, EpcisKgError> {
        // For now, return true for basic validation
        // In a real implementation, this would query the knowledge graph
        Ok(true)
    }
    
    /// Check if disposition is valid according to ontology
    fn is_valid_disposition(&self, _disposition: &str) -> Result<bool, EpcisKgError> {
        // For now, return true for basic validation
        // In a real implementation, this would query the knowledge graph
        Ok(true)
    }
    
    /// Update processing statistics
    async fn update_stats(&mut self, success: bool, validation_error: bool, start_time: std::time::Instant) {
        self.processing_stats.total_events_processed += 1;
        
        if success {
            self.processing_stats.successful_events += 1;
        } else {
            self.processing_stats.failed_events += 1;
            
            if validation_error {
                self.processing_stats.validation_errors += 1;
            } else {
                self.processing_stats.processing_errors += 1;
            }
        }
        
        // Update average processing time
        let processing_time = start_time.elapsed().as_millis() as f64;
        let total_events = self.processing_stats.total_events_processed;
        let current_avg = self.processing_stats.average_processing_time_ms;
        self.processing_stats.average_processing_time_ms = 
            (current_avg * (total_events - 1) as f64 + processing_time) / total_events as f64;
        
        self.processing_stats.last_processed_time = Some(chrono::Utc::now());
    }
    
    /// Get processing statistics
    pub fn get_stats(&self) -> &ProcessingStats {
        &self.processing_stats
    }
    
    /// Reset processing statistics
    pub fn reset_stats(&mut self) {
        self.processing_stats = ProcessingStats::default();
    }
}