mod test_utils;

use test_utils::{test_data, assertions, validation_test_cases};
use epcis_knowledge_graph::models::epcis::EpcisEvent;
use epcis_knowledge_graph::utils::validation::Validator;

#[test]
fn test_validation_with_sample_event() {
    let event = test_data::sample_epcis_event();
    let validator = Validator::new();
    
    let result = validator.validate_epcis_event(&event);
    assert!(result.is_ok(), "Sample event should be valid");
    
    let warnings = result.unwrap();
    assert!(warnings.is_empty(), "Sample event should have no warnings");
}

#[test]
fn test_validation_invalid_events() {
    let test_cases = validation_test_cases();
    
    for test_case in test_cases {
        println!("Running test case: {}", test_case.name);
        
        let event: EpcisEvent = serde_json::from_str(&test_case.input)
            .expect(&format!("Failed to parse event for test case: {}", test_case.name));
        
        let validator = Validator::new();
        let result = validator.validate_epcis_event(&event);
        
        if test_case.should_pass {
            assert!(result.is_ok(), "Test case '{}' should pass: {:?}", test_case.name, result);
        } else {
            assert!(result.is_err(), "Test case '{}' should fail: {:?}", test_case.name, result);
            
            if let Err(error) = result {
                let error_msg = error.to_string();
                assert!(
                    error_msg.contains(&test_case.expected_output),
                    "Error message should contain '{}'. Got: '{}'",
                    test_case.expected_output,
                    error_msg
                );
            }
        }
    }
}

#[test]
fn test_validation_edge_cases() {
    let validator = Validator::new();
    
    // Test event with whitespace-only ID
    let mut event = test_data::sample_epcis_event();
    event.event_id = "   ".to_string();
    let result = validator.validate_epcis_event(&event);
    assert!(result.is_ok(), "Whitespace-only ID should be valid (no explicit trim)");
    
    // Test event with special characters in ID
    event.event_id = "test-event_001@company.com".to_string();
    let result = validator.validate_epcis_event(&event);
    assert!(result.is_ok(), "Special characters in ID should be valid");
    
    // Test event with very long ID
    event.event_id = "a".repeat(1000);
    let result = validator.validate_epcis_event(&event);
    assert!(result.is_ok(), "Very long ID should be valid");
}

#[test]
fn test_validation_multiple_epcs() {
    let validator = Validator::new();
    
    // Test event with many EPCs
    let mut event = test_data::sample_epcis_event();
    event.epc_list = (0..1000)
        .map(|i| format!("urn:epc:id:sgtin:123456.789.{}", i))
        .collect();
    
    let result = validator.validate_epcis_event(&event);
    assert!(result.is_ok(), "Event with many EPCs should be valid");
    assert_eq!(event.epc_list.len(), 1000);
}

#[test]
fn test_validation_event_types() {
    let validator = Validator::new();
    
    let event_types = vec![
        "ObjectEvent",
        "AggregationEvent", 
        "QuantityEvent",
        "TransactionEvent",
        "TransformationEvent",
    ];
    
    for event_type in event_types {
        let mut event = test_data::sample_epcis_event();
        event.event_type = event_type.to_string();
        
        let result = validator.validate_epcis_event(&event);
        assert!(result.is_ok(), "Event type '{}' should be valid", event_type);
    }
}

#[test]
fn test_validation_all_actions() {
    let validator = Validator::new();
    
    let valid_actions = vec!["ADD", "OBSERVE", "DELETE"];
    
    for action in valid_actions {
        let mut event = test_data::sample_epcis_event();
        event.event_action = action.to_string();
        
        let result = validator.validate_epcis_event(&event);
        assert!(result.is_ok(), "Action '{}' should be valid", action);
    }
}

#[test]
fn test_validation_serialization_compatibility() {
    let event = test_data::sample_epcis_event();
    assertions::assert_epcis_event_serialization_roundtrip(&event);
}

#[test]
fn test_validation_event_creation_time_order() {
    let mut event = test_data::sample_epcis_event();
    
    // Valid: event_time <= record_time
    event.event_time = "2024-01-01T10:00:00Z".to_string();
    event.record_time = "2024-01-01T10:01:00Z".to_string();
    
    let validator = Validator::new();
    let result = validator.validate_epcis_event(&event);
    assert!(result.is_ok(), "Event with event_time <= record_time should be valid");
    
    // Currently, our validator doesn't check time order, but this documents the expectation
    // If we add time validation in the future, this test will need updating
}

#[test]
fn test_validation_biz_step_values() {
    let validator = Validator::new();
    
    let biz_steps = vec![
        Some("commissioning".to_string()),
        Some("encoding".to_string()),
        Some("packing".to_string()),
        Some("shipping".to_string()),
        Some("receiving".to_string()),
        Some("inspecting".to_string()),
        None, // Optional field
    ];
    
    for biz_step in biz_steps {
        let mut event = test_data::sample_epcis_event();
        event.biz_step = biz_step;
        
        let result = validator.validate_epcis_event(&event);
        assert!(result.is_ok(), "Biz step '{:?}' should be valid", event.biz_step);
    }
}

#[test]
fn test_validation_disposition_values() {
    let validator = Validator::new();
    
    let dispositions = vec![
        Some("active".to_string()),
        Some("inactive".to_string()),
        Some("expired".to_string()),
        Some("damaged".to_string()),
        None, // Optional field
    ];
    
    for disposition in dispositions {
        let mut event = test_data::sample_epcis_event();
        event.disposition = disposition;
        
        let result = validator.validate_epcis_event(&event);
        assert!(result.is_ok(), "Disposition '{:?}' should be valid", event.disposition);
    }
}

#[test]
fn test_validation_location_formats() {
    let validator = Validator::new();
    
    let locations = vec![
        Some("urn:epc:id:sgln:123456.789.0".to_string()),
        Some("urn:epc:id:sgln:123456.789.1".to_string()),
        Some("http://example.com/location/warehouse1".to_string()),
        None, // Optional field
    ];
    
    for location in locations {
        let mut event = test_data::sample_epcis_event();
        event.biz_location = location;
        
        let result = validator.validate_epcis_event(&event);
        assert!(result.is_ok(), "Location '{:?}' should be valid", event.biz_location);
    }
}

#[test]
fn test_validation_performance() {
    let validator = Validator::new();
    
    // Test validation performance with 1000 events
    let events: Vec<EpcisEvent> = (0..1000)
        .map(|i| {
            let mut event = test_data::sample_epcis_event();
            event.event_id = format!("event-{:04}", i);
            event
        })
        .collect();
    
    let start = std::time::Instant::now();
    
    for event in &events {
        let result = validator.validate_epcis_event(event);
        assert!(result.is_ok(), "Event '{}' should be valid", event.event_id);
    }
    
    let duration = start.elapsed();
    println!("Validated 1000 events in {:?}", duration);
    
    // Assert that validation is reasonably fast (should be much less than 1 second for 1000 events)
    assert!(duration.as_millis() < 1000, "Validation should be fast");
}

#[test]
fn test_validation_error_accumulation() {
    let validator = Validator::new();
    
    // Create an event with multiple validation errors
    let mut event = test_data::sample_epcis_event();
    event.event_id = String::new(); // Empty ID
    event.epc_list = Vec::new(); // Empty EPC list
    event.event_action = "INVALID".to_string(); // Invalid action
    
    let result = validator.validate_epcis_event(&event);
    assert!(result.is_err());
    
    let error_msg = result.unwrap_err().to_string();
    
    // Check that all validation errors are reported
    assert!(error_msg.contains("Event ID cannot be empty"));
    assert!(error_msg.contains("EPC list cannot be empty"));
    assert!(error_msg.contains("Invalid event action"));
}

#[test]
fn test_validation_unicode_support() {
    let validator = Validator::new();
    
    // Test event with Unicode characters
    let mut event = test_data::sample_epcis_event();
    event.event_id = "测试-事件-001".to_string(); // Chinese
    event.biz_step = Some("réception".to_string()); // French
    event.disposition = Some("activo".to_string()); // Spanish
    
    let result = validator.validate_epcis_event(&event);
    assert!(result.is_ok(), "Event with Unicode characters should be valid");
}