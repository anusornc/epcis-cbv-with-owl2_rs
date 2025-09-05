use serde::{Deserialize, Serialize};
use epcis_knowledge_graph::models::epcis::EpcisEvent;

pub mod test_data {
    use super::*;

    pub fn sample_epcis_event() -> EpcisEvent {
        EpcisEvent {
            event_id: "test-event-001".to_string(),
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
        }
    }

    pub fn invalid_epcis_event_empty_id() -> EpcisEvent {
        let mut event = sample_epcis_event();
        event.event_id = String::new();
        event
    }

    pub fn invalid_epcis_event_empty_epc_list() -> EpcisEvent {
        let mut event = sample_epcis_event();
        event.epc_list = Vec::new();
        event
    }

    pub fn invalid_epcis_event_invalid_action() -> EpcisEvent {
        let mut event = sample_epcis_event();
        event.event_action = "INVALID_ACTION".to_string();
        event
    }

    pub fn sample_turtle_ontology() -> &'static str {
        r#"
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix epcis: <urn:epcglobal:epcis:> .
        @prefix cbv: <urn:epcglobal:cbv:> .
        @prefix ex: <http://example.com/> .
        
        ex:Product a rdfs:Class ;
            rdfs:label "Product" ;
            rdfs:comment "A product in the supply chain" .
            
        ex:hasEPC a rdf:Property ;
            rdfs:domain ex:Product ;
            rdfs:range xsd:string ;
            rdfs:label "has EPC" .
            
        ex:locatedAt a rdf:Property ;
            rdfs:domain ex:Product ;
            rdfs:range ex:Location ;
            rdfs:label "located at" .
            
        ex:Location a rdfs:Class ;
            rdfs:label "Location" .
        "#
    }

    pub fn sample_sparql_query() -> &'static str {
        r#"
        PREFIX ex: <http://example.com/>
        SELECT ?product ?location WHERE {
            ?product a ex:Product .
            ?product ex:locatedAt ?location .
        }
        LIMIT 10
        "#
    }

    pub fn sample_epcis_json_event() -> &'static str {
        r#"{
            "event_id": "test-event-001",
            "event_type": "ObjectEvent",
            "event_time": "2024-01-01T00:00:00Z",
            "record_time": "2024-01-01T00:00:00Z",
            "event_action": "ADD",
            "epc_list": [
                "urn:epc:id:sgtin:123456.789.100",
                "urn:epc:id:sgtin:123456.789.101"
            ],
            "biz_step": "commissioning",
            "disposition": "active",
            "biz_location": "urn:epc:id:sgln:123456.789.0"
        }"#
    }
}

pub mod temp_dir {
    use tempfile::TempDir;
    use std::path::PathBuf;

    pub fn create_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp directory")
    }

    pub fn create_temp_file_with_content(dir: &PathBuf, filename: &str, content: &str) -> PathBuf {
        let file_path = dir.join(filename);
        std::fs::write(&file_path, content).expect("Failed to write temp file");
        file_path
    }
}

pub mod assertions {
    use epcis_knowledge_graph::models::epcis::EpcisEvent;

    pub fn assert_epcis_event_valid(event: &EpcisEvent) {
        assert!(!event.event_id.is_empty(), "Event ID should not be empty");
        assert!(!event.epc_list.is_empty(), "EPC list should not be empty");
        assert!(
            ["ADD", "OBSERVE", "DELETE"].contains(&event.event_action.as_str()),
            "Event action should be valid"
        );
    }

    pub fn assert_epcis_event_serialization_roundtrip(event: &EpcisEvent) {
        let json = serde_json::to_string(event).expect("Failed to serialize event");
        let deserialized: EpcisEvent = serde_json::from_str(&json).expect("Failed to deserialize event");
        assert_eq!(event, &deserialized, "Event should be equal after serialization roundtrip");
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestConfig {
    pub database_path: String,
    pub server_port: u16,
    pub log_level: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            database_path: "./test_data".to_string(),
            server_port: 8081,
            log_level: "debug".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub description: String,
    pub input: String,
    pub expected_output: String,
    pub should_pass: bool,
}

impl TestCase {
    pub fn new(name: &str, description: &str, input: &str, expected_output: &str, should_pass: bool) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            input: input.to_string(),
            expected_output: expected_output.to_string(),
            should_pass,
        }
    }
}

pub fn validation_test_cases() -> Vec<TestCase> {
    vec![
        TestCase::new(
            "Valid Event",
            "A complete valid EPCIS event should pass validation",
            test_data::sample_epcis_json_event(),
            "Validation successful",
            true,
        ),
        TestCase::new(
            "Empty Event ID",
            "Event with empty ID should fail validation",
            r#"{"event_id": "", "event_type": "ObjectEvent", "event_time": "2024-01-01T00:00:00Z", "record_time": "2024-01-01T00:00:00Z", "event_action": "ADD", "epc_list": ["test"]}"#,
            "Event ID cannot be empty",
            false,
        ),
        TestCase::new(
            "Empty EPC List",
            "Event with empty EPC list should fail validation",
            r#"{"event_id": "test", "event_type": "ObjectEvent", "event_time": "2024-01-01T00:00:00Z", "record_time": "2024-01-01T00:00:00Z", "event_action": "ADD", "epc_list": []}"#,
            "EPC list cannot be empty",
            false,
        ),
        TestCase::new(
            "Invalid Action",
            "Event with invalid action should fail validation",
            r#"{"event_id": "test", "event_type": "ObjectEvent", "event_time": "2024-01-01T00:00:00Z", "record_time": "2024-01-01T00:00:00Z", "event_action": "INVALID", "epc_list": ["test"]}"#,
            "Invalid event action",
            false,
        ),
    ]
}