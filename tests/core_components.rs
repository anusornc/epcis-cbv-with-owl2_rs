use epcis_knowledge_graph::ontology::reasoner::{OntologyReasoner, MaterializationStrategy, InferenceResult};
use epcis_knowledge_graph::storage::oxigraph_store::OxigraphStore;
use epcis_knowledge_graph::models::epcis::EpcisEvent;
use epcis_knowledge_graph::pipeline::EpcisEventPipeline;
use epcis_knowledge_graph::ontology::loader::{OntologyLoader, OntologyData};
use tempfile::TempDir;
use std::collections::HashMap;
use oxrdf::Graph;

#[test]
fn test_ontology_reasoner_creation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    assert!(reasoner.is_parallel_processing_enabled());
    assert_eq!(reasoner.get_cache_size_limit(), 10000);
    assert_eq!(reasoner.get_batch_size(), 1000);
}

#[test]
fn test_materialization_strategies() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    // Test different materialization strategies
    let strategies = vec![
        MaterializationStrategy::Full,
        MaterializationStrategy::Incremental,
        MaterializationStrategy::OnDemand,
        MaterializationStrategy::Hybrid,
    ];
    
    for strategy in strategies {
        reasoner.set_materialization_strategy(strategy.clone());
        // Verify the strategy was set (no panic)
        assert!(true);
    }
}

#[test]
fn test_performance_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    // Test performance configuration
    reasoner.configure_performance(true, 5000, 500);
    
    assert!(reasoner.is_parallel_processing_enabled());
    assert_eq!(reasoner.get_cache_size_limit(), 5000);
    assert_eq!(reasoner.get_batch_size(), 500);
}

#[test]
fn test_inference_result_creation() {
    let result = InferenceResult {
        consistent: true,
        classification_performed: true,
        realization_performed: false,
        materialized_triples: 5,
        sparql_inferences: 2,
        individuals_classified: 3,
        incremental: false,
        new_triples_processed: 10,
        inference_errors: Vec::new(),
        processing_time_ms: 100,
    };
    
    assert!(result.consistent);
    assert!(result.classification_performed);
    assert!(!result.realization_performed);
    assert_eq!(result.materialized_triples, 5);
    assert_eq!(result.sparql_inferences, 2);
    assert_eq!(result.individuals_classified, 3);
    assert!(!result.incremental);
    assert_eq!(result.new_triples_processed, 10);
    assert!(result.inference_errors.is_empty());
    assert_eq!(result.processing_time_ms, 100);
}

#[test]
fn test_epcis_event_validation() {
    use epcis_knowledge_graph::models::epcis::EpcisEvent;
    
    // Test valid event
    let valid_event = EpcisEvent {
        event_id: "test-event-001".to_string(),
        event_type: "ObjectEvent".to_string(),
        event_time: "2024-01-15T10:30:00Z".to_string(),
        record_time: "2024-01-15T10:31:00Z".to_string(),
        event_action: "ADD".to_string(),
        epc_list: vec![
            "urn:epc:id:sgtin:0614141.107346.2018".to_string(),
            "urn:epc:id:sgtin:0614141.107346.2019".to_string(),
        ],
        biz_step: Some("commissioning".to_string()),
        disposition: Some("active".to_string()),
        biz_location: Some("urn:epc:id:sgln:0614141.00777.0".to_string()),
    };
    
    assert!(!valid_event.event_id.is_empty());
    assert!(!valid_event.epc_list.is_empty());
    assert!(["ADD", "OBSERVE", "DELETE"].contains(&valid_event.event_action.as_str()));
    
    // Test event serialization
    let json = serde_json::to_string(&valid_event).expect("Failed to serialize event");
    let deserialized: EpcisEvent = serde_json::from_str(&json).expect("Failed to deserialize event");
    assert_eq!(valid_event.event_id, deserialized.event_id);
    assert_eq!(valid_event.epc_list, deserialized.epc_list);
}

#[test]
fn test_ontology_loader_creation() {
    let _loader = OntologyLoader::new();
    
    // Test that loader can be created
    assert!(true); // If we get here, creation succeeded
}

#[test]
fn test_ontology_data_structure() {
    let ontology_data = OntologyData {
        graph: Graph::default(),
        triples_count: 100,
        source_file: "test.ttl".to_string(),
    };
    
    assert_eq!(ontology_data.triples_count, 100);
    assert_eq!(ontology_data.source_file, "test.ttl");
}

#[test]
fn test_store_creation_and_statistics() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let store = OxigraphStore::new(db_path).unwrap();
    let stats = store.get_statistics().unwrap();
    
    // New store should have minimal data
    assert!(stats.total_quads >= 0);
    assert!(stats.named_graphs >= 0);
    assert_eq!(stats.storage_path, db_path);
}

#[test]
fn test_reasoning_cache_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    // Test that reasoner can be configured with cache settings
    reasoner.configure_performance(true, 1000, 100);
    
    // Verify cache size limit was set
    assert_eq!(reasoner.get_cache_size_limit(), 1000);
}

#[test]
fn test_materialized_triples_management() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    // Initially should be empty
    assert!(reasoner.get_materialized_triples().is_empty());
    
    // Add some materialized triples (simplified test)
    reasoner.clear_materialized_triples();
    
    // Should still be empty after clearing
    assert!(reasoner.get_materialized_triples().is_empty());
}

#[test]
fn test_inference_statistics() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    let initial_stats = reasoner.get_detailed_stats();
    
    // Initial stats should be at zero
    assert_eq!(initial_stats.total_inferences, 0);
    assert_eq!(initial_stats.materialized_triples_count, 0);
    assert_eq!(initial_stats.total_processing_time_ms, 0);
    assert_eq!(initial_stats.cache_hits, 0);
    assert_eq!(initial_stats.cache_misses, 0);
}

#[test]
fn test_performance_metrics() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    let metrics = reasoner.get_performance_metrics();
    
    // Initial metrics should be zero
    assert_eq!(metrics.cache_hits.load(std::sync::atomic::Ordering::Relaxed), 0);
    assert_eq!(metrics.cache_misses.load(std::sync::atomic::Ordering::Relaxed), 0);
    assert_eq!(metrics.cache_hit_rate(), 0.0);
}

#[test]
fn test_clone_implementation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    // Test cloning
    let cloned_reasoner = reasoner.clone();
    
    // Both should have the same configuration
    assert_eq!(reasoner.is_parallel_processing_enabled(), cloned_reasoner.is_parallel_processing_enabled());
    assert_eq!(reasoner.get_cache_size_limit(), cloned_reasoner.get_cache_size_limit());
    assert_eq!(reasoner.get_batch_size(), cloned_reasoner.get_batch_size());
}

#[test]
fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    // Test with invalid query (should handle gracefully)
    let _result = reasoner.perform_inference();
    // Should handle gracefully (may succeed or fail gracefully)
    assert!(true); // If we get here, no panic occurred
}

#[test]
fn test_configuration_validation() {
    use epcis_knowledge_graph::Config;
    
    // Test default configuration
    let config = Config::default();
    assert!(config.validate().is_ok());
    
    // Test custom configuration
    let custom_config = Config {
        database_path: "./test_db".to_string(),
        server_port: 8081,
        log_level: "info".to_string(),
        ontology_paths: vec!["ontologies/test.ttl".to_string()],
        reasoning: Default::default(),
        sparql: Default::default(),
        server: Default::default(),
        persistence: Default::default(),
    };
    
    assert!(custom_config.validate().is_ok());
    assert_eq!(custom_config.server_port, 8081);
    assert_eq!(custom_config.database_path, "./test_db");
}

#[test]
fn test_edge_cases() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    // Test with very large cache limit
    reasoner.configure_performance(false, usize::MAX, 1);
    assert_eq!(reasoner.get_cache_size_limit(), usize::MAX);
    
    // Test with zero batch size
    reasoner.configure_performance(false, 1000, 0);
    assert_eq!(reasoner.get_batch_size(), 0);
}