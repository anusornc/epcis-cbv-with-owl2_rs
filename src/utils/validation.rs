use crate::models::epcis::EpcisEvent;
use crate::EpcisKgError;

pub struct Validator;

impl Validator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn validate_epcis_event(&self, event: &EpcisEvent) -> Result<Vec<String>, EpcisKgError> {
        let mut errors = Vec::new();
        
        if event.event_id.is_empty() {
            errors.push("Event ID cannot be empty".to_string());
        }
        
        if event.epc_list.is_empty() {
            errors.push("EPC list cannot be empty".to_string());
        }
        
        if !["ADD", "OBSERVE", "DELETE"].contains(&event.event_action.as_str()) {
            errors.push(format!("Invalid event action: {}", event.event_action));
        }
        
        if errors.is_empty() {
            Ok(vec![])
        } else {
            Err(EpcisKgError::Validation(errors.join("; ")))
        }
    }
    
    pub fn validate_ontology_compliance(&self, _data: &str) -> Result<(), EpcisKgError> {
        todo!("Implement ontology compliance validation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::epcis::EpcisEvent;

    #[test]
    fn test_validator_creation() {
        let validator = Validator::new();
        // Test that validator can be created
        assert!(true); // If we get here, creation succeeded
    }

    #[test]
    fn test_valid_event_validation() {
        let validator = Validator::new();
        let event = EpcisEvent {
            event_id: "test-event-001".to_string(),
            event_type: "ObjectEvent".to_string(),
            event_time: "2024-01-01T00:00:00Z".to_string(),
            record_time: "2024-01-01T00:00:00Z".to_string(),
            event_action: "ADD".to_string(),
            epc_list: vec!["urn:epc:id:sgtin:123456.789.100".to_string()],
            biz_step: Some("commissioning".to_string()),
            disposition: Some("active".to_string()),
            biz_location: Some("urn:epc:id:sgln:123456.789.0".to_string()),
        };

        let result = validator.validate_epcis_event(&event);
        assert!(result.is_ok());
        let warnings = result.unwrap();
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_invalid_event_empty_id() {
        let validator = Validator::new();
        let mut event = EpcisEvent::default();
        event.event_id = "".to_string();
        event.epc_list = vec!["urn:epc:id:sgtin:123456.789.100".to_string()];

        let result = validator.validate_epcis_event(&event);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, EpcisKgError::Validation(_)));
        assert!(error.to_string().contains("Event ID cannot be empty"));
    }

    #[test]
    fn test_invalid_event_empty_epc_list() {
        let validator = Validator::new();
        let mut event = EpcisEvent::default();
        event.event_id = "test-event".to_string();
        event.epc_list = vec![];

        let result = validator.validate_epcis_event(&event);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("EPC list cannot be empty"));
    }

    #[test]
    fn test_invalid_event_action() {
        let validator = Validator::new();
        let mut event = EpcisEvent::default();
        event.event_id = "test-event".to_string();
        event.epc_list = vec!["urn:epc:id:sgtin:123456.789.100".to_string()];
        event.event_action = "INVALID".to_string();

        let result = validator.validate_epcis_event(&event);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid event action: INVALID"));
    }

    #[test]
    fn test_multiple_validation_errors() {
        let validator = Validator::new();
        let event = EpcisEvent {
            event_id: "".to_string(),
            event_type: "ObjectEvent".to_string(),
            event_time: "2024-01-01T00:00:00Z".to_string(),
            record_time: "2024-01-01T00:00:00Z".to_string(),
            event_action: "INVALID".to_string(),
            epc_list: vec![],
            biz_step: None,
            disposition: None,
            biz_location: None,
        };

        let result = validator.validate_epcis_event(&event);
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        
        // Should contain multiple error messages
        assert!(error_msg.contains("Event ID cannot be empty"));
        assert!(error_msg.contains("EPC list cannot be empty"));
        assert!(error_msg.contains("Invalid event action: INVALID"));
    }
}