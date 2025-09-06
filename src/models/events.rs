use crate::models::epcis::EpcisEvent;
use crate::EpcisKgError;
use serde::{Serialize, Deserialize};

/// Result of event processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub event_id: String,
    pub success: bool,
    pub processing_time_ms: u64,
    pub error: Option<String>,
    pub triples_generated: usize,
    pub inferences_made: usize,
}

/// Result of event validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Event processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventProcessingStats {
    pub total_events: usize,
    pub successful_events: usize,
    pub failed_events: usize,
    pub average_processing_time_ms: f64,
    pub last_event_time: Option<String>,
}

/// Event processor for EPCIS events
pub struct EventProcessor {
    config: Option<crate::config::AppConfig>,
}

impl EventProcessor {
    /// Create a new event processor
    pub fn new() -> Self {
        Self {
            config: None,
        }
    }
    
    /// Create a new event processor with configuration
    pub fn with_config(config: crate::config::AppConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
    
    /// Validate an EPCIS event
    pub fn validate_event(&self, event: &EpcisEvent) -> Result<ValidationResult, EpcisKgError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Basic validation
        if event.event_id.is_empty() {
            errors.push("Event ID is required".to_string());
        }
        
        if event.event_type.is_empty() {
            errors.push("Event type is required".to_string());
        }
        
        if event.event_time.is_empty() {
            errors.push("Event time is required".to_string());
        }
        
        if event.epc_list.is_empty() {
            errors.push("EPC list cannot be empty".to_string());
        }
        
        // Validate event type
        let valid_types = vec![
            "ObjectEvent", "AggregationEvent", "QuantityEvent", 
            "TransactionEvent", "TransformationEvent"
        ];
        
        if !valid_types.contains(&event.event_type.as_str()) {
            errors.push(format!("Invalid event type: {}", event.event_type));
        }
        
        // Validate action
        let valid_actions = vec!["ADD", "OBSERVE", "DELETE"];
        if !valid_actions.contains(&event.event_action.as_str()) {
            errors.push(format!("Invalid action: {}", event.event_action));
        }
        
        // DateTime validation
        if let Err(_) = chrono::DateTime::parse_from_rfc3339(&event.event_time) {
            errors.push(format!("Invalid event time format: {}", event.event_time));
        }
        
        if let Err(_) = chrono::DateTime::parse_from_rfc3339(&event.record_time) {
            errors.push(format!("Invalid record time format: {}", event.record_time));
        }
        
        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
    
    /// Process an EPCIS event (basic processing)
    pub fn process_event(&self, event: &EpcisEvent) -> Result<ProcessingResult, EpcisKgError> {
        let start_time = std::time::Instant::now();
        
        // Validate the event first
        let validation = self.validate_event(event)?;
        if !validation.is_valid {
            return Ok(ProcessingResult {
                event_id: event.event_id.clone(),
                success: false,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(format!("Validation failed: {:?}", validation.errors)),
                triples_generated: 0,
                inferences_made: 0,
            });
        }
        
        // Basic processing logic
        let triples_count = self.estimate_triples_count(event);
        
        Ok(ProcessingResult {
            event_id: event.event_id.clone(),
            success: true,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            error: None,
            triples_generated: triples_count,
            inferences_made: 0,
        })
    }
    
    /// Estimate the number of triples that will be generated for an event
    fn estimate_triples_count(&self, event: &EpcisEvent) -> usize {
        let mut count = 5; // Basic triples: type, id, eventTime, recordTime, action
        
        count += event.epc_list.len(); // One triple per EPC
        
        if event.biz_step.is_some() {
            count += 1;
        }
        
        if event.disposition.is_some() {
            count += 1;
        }
        
        if event.biz_location.is_some() {
            count += 1;
        }
        
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::epcis::EpcisEvent;

    #[test]
    fn test_event_processor_creation() {
        let processor = EventProcessor::new();
        assert!(processor.config.is_none());
    }
    
    #[test]
    fn test_valid_event_validation() {
        let processor = EventProcessor::new();
        let event = EpcisEvent {
            event_id: "test-001".to_string(),
            event_type: "ObjectEvent".to_string(),
            event_time: "2024-01-01T00:00:00Z".to_string(),
            record_time: "2024-01-01T00:00:00Z".to_string(),
            event_action: "ADD".to_string(),
            epc_list: vec!["urn:epc:id:sgtin:123456.789.100".to_string()],
            biz_step: Some("commissioning".to_string()),
            disposition: Some("active".to_string()),
            biz_location: Some("urn:epc:id:sgln:123456.789.0".to_string()),
        };
        
        let result = processor.validate_event(&event).unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }
    
    #[test]
    fn test_invalid_event_validation() {
        let processor = EventProcessor::new();
        let event = EpcisEvent {
            event_id: "".to_string(), // Invalid: empty
            event_type: "InvalidType".to_string(), // Invalid: not in list
            event_time: "invalid-time".to_string(), // Invalid: wrong format
            record_time: "2024-01-01T00:00:00Z".to_string(),
            event_action: "INVALID".to_string(), // Invalid: not in list
            epc_list: vec![], // Invalid: empty
            biz_step: None,
            disposition: None,
            biz_location: None,
        };
        
        let result = processor.validate_event(&event).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        assert!(result.errors.iter().any(|e| e.contains("Event ID is required")));
        assert!(result.errors.iter().any(|e| e.contains("Invalid event type")));
        assert!(result.errors.iter().any(|e| e.contains("Invalid event time format")));
        assert!(result.errors.iter().any(|e| e.contains("Invalid action")));
        assert!(result.errors.iter().any(|e| e.contains("EPC list cannot be empty")));
    }
    
    #[test]
    fn test_event_processing() {
        let processor = EventProcessor::new();
        let event = EpcisEvent {
            event_id: "test-001".to_string(),
            event_type: "ObjectEvent".to_string(),
            event_time: "2024-01-01T00:00:00Z".to_string(),
            record_time: "2024-01-01T00:00:00Z".to_string(),
            event_action: "ADD".to_string(),
            epc_list: vec!["urn:epc:id:sgtin:123456.789.100".to_string()],
            biz_step: Some("commissioning".to_string()),
            disposition: Some("active".to_string()),
            biz_location: Some("urn:epc:id:sgln:123456.789.0".to_string()),
        };
        
        let result = processor.process_event(&event).unwrap();
        assert!(result.success);
        assert_eq!(result.event_id, "test-001");
        assert!(result.error.is_none());
        assert_eq!(result.triples_generated, 9); // 5 basic + 1 EPC + 1 biz_step + 1 disposition + 1 location
        assert_eq!(result.inferences_made, 0);
    }
    
    #[test]
    fn test_triples_count_estimation() {
        let processor = EventProcessor::new();
        
        // Minimal event
        let minimal_event = EpcisEvent {
            event_id: "minimal".to_string(),
            event_type: "ObjectEvent".to_string(),
            event_time: "2024-01-01T00:00:00Z".to_string(),
            record_time: "2024-01-01T00:00:00Z".to_string(),
            event_action: "ADD".to_string(),
            epc_list: vec!["urn:epc:id:sgtin:123456.789.100".to_string()],
            biz_step: None,
            disposition: None,
            biz_location: None,
        };
        
        assert_eq!(processor.estimate_triples_count(&minimal_event), 6); // 5 basic + 1 EPC
        
        // Full event
        let full_event = EpcisEvent {
            event_id: "full".to_string(),
            event_type: "ObjectEvent".to_string(),
            event_time: "2024-01-01T00:00:00Z".to_string(),
            record_time: "2024-01-01T00:00:00Z".to_string(),
            event_action: "ADD".to_string(),
            epc_list: vec![
                "urn:epc:id:sgtin:123456.789.100".to_string(),
                "urn:epc:id:sgtin:123456.789.101".to_string(),
            ],
            biz_step: Some("commissioning".to_string()),
            disposition: Some("active".to_string()),
            biz_location: Some("urn:epc:id:sgln:123456.789.0".to_string()),
        };
        
        assert_eq!(processor.estimate_triples_count(&full_event), 10); // 5 basic + 2 EPCs + 1 biz_step + 1 disposition + 1 location
    }
}