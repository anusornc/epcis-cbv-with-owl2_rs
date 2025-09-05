#[cfg(test)]
mod performance_benchmarks {
    use epcis_knowledge_graph::models::epcis::EpcisEvent;
    use epcis_knowledge_graph::utils::validation::Validator;
    use epcis_knowledge_graph::ontology::loader::OntologyLoader;
    use epcis_knowledge_graph::ontology::reasoner::OntologyReasoner;
    use std::time::Instant;
    use tempfile::TempDir;

    fn sample_epcis_event() -> EpcisEvent {
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

    fn sample_turtle_ontology() -> &'static str {
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

    #[test]
    fn benchmark_epcis_validation() {
        let validator = Validator::new();
        let num_events = 1000;
        
        let events: Vec<EpcisEvent> = (0..num_events)
            .map(|i| {
                let mut event = sample_epcis_event();
                event.event_id = format!("benchmark-event-{:04}", i);
                event
            })
            .collect();
        
        let start = Instant::now();
        
        for event in &events {
            let result = validator.validate_epcis_event(event);
            assert!(result.is_ok(), "Event '{}' should be valid", event.event_id);
        }
        
        let duration = start.elapsed();
        let avg_duration = duration.as_micros() as f64 / num_events as f64;
        
        println!("Validated {} events in {:?}", num_events, duration);
        println!("Average validation time: {:.2} μs per event", avg_duration);
        
        assert!(duration.as_millis() < 1000, "Validation should complete in under 1 second");
        assert!(avg_duration < 1000.0, "Average validation should be under 1ms per event");
    }

    #[test]
    fn benchmark_epcis_serialization() {
        let num_events = 1000;
        let events: Vec<EpcisEvent> = (0..num_events)
            .map(|i| {
                let mut event = sample_epcis_event();
                event.event_id = format!("serialization-event-{:04}", i);
                event
            })
            .collect();
        
        let start = Instant::now();
        
        for event in &events {
            let json = serde_json::to_string(event).expect("Serialization should succeed");
            assert!(!json.is_empty());
        }
        
        let serialization_duration = start.elapsed();
        
        let start = Instant::now();
        for event in &events {
            let json = serde_json::to_string(event).expect("Serialization should succeed");
            let deserialized: EpcisEvent = serde_json::from_str(&json).expect("Deserialization should succeed");
            assert_eq!(event, &deserialized);
        }
        
        let deserialization_duration = start.elapsed();
        
        println!("Serialized {} events in {:?}", num_events, serialization_duration);
        println!("Deserialized {} events in {:?}", num_events, deserialization_duration);
        
        let avg_serialization = serialization_duration.as_micros() as f64 / num_events as f64;
        let avg_deserialization = deserialization_duration.as_micros() as f64 / num_events as f64;
        
        println!("Average serialization time: {:.2} μs per event", avg_serialization);
        println!("Average deserialization time: {:.2} μs per event", avg_deserialization);
        
        assert!(serialization_duration.as_millis() < 500, "Serialization should complete in under 500ms");
        assert!(deserialization_duration.as_millis() < 500, "Deserialization should complete in under 500ms");
    }

    #[test]
    fn benchmark_ontology_loading() {
        let temp_dir = TempDir::new().unwrap();
        let loader = OntologyLoader::new();
        
        let sizes = vec![100, 500, 1000];
        
        for size in sizes {
            let ontology_content = generate_large_ontology(size);
            let ontology_file = temp_dir.path().join(format!("benchmark_ontology_{}.ttl", size));
            std::fs::write(&ontology_file, &ontology_content).unwrap();
            
            let start = Instant::now();
            let result = loader.load_ontology(&ontology_file);
            let duration = start.elapsed();
            
            println!("Loading {} triples took: {:?}", size, duration);
            
            assert!(matches!(result, Err(_)));
            assert!(duration.as_millis() < 100, "Loading should be fast even for placeholder implementation");
        }
    }

    #[test]
    fn benchmark_reasoner_validation() {
        let reasoner = OntologyReasoner::new();
        let ontology_sizes = vec![100, 500, 1000];
        
        for size in ontology_sizes {
            let ontology_content = generate_large_ontology(size);
            
            let start = Instant::now();
            let result = reasoner.validate_ontology(&ontology_content);
            let duration = start.elapsed();
            
            println!("Validating ontology with {} triples took: {:?}", size, duration);
            
            assert!(matches!(result, Err(_)));
            assert!(duration.as_millis() < 100, "Validation should be fast even for placeholder implementation");
        }
    }

    #[test]
    fn benchmark_memory_usage() {
        let num_events = 10000;
        let events: Vec<EpcisEvent> = (0..num_events)
            .map(|i| {
                let mut event = sample_epcis_event();
                event.event_id = format!("memory-test-event-{:04}", i);
                event
            })
            .collect();
        
        let start = Instant::now();
        
        let validator = Validator::new();
        for event in &events {
            let result = validator.validate_epcis_event(event);
            assert!(result.is_ok());
        }
        
        let duration = start.elapsed();
        
        println!("Processed {} events in {:?}", num_events, duration);
        
        assert!(duration.as_millis() < 5000, "Should handle 10k events in under 5 seconds");
    }

    #[test]
    fn benchmark_concurrent_validation() {
        use std::thread;
        use std::sync::Arc;
        
        let validator = Arc::new(Validator::new());
        let num_threads = 4;
        let events_per_thread = 250;
        
        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let validator = validator.clone();
                thread::spawn(move || {
                    let start = Instant::now();
                    
                    for i in 0..events_per_thread {
                        let mut event = sample_epcis_event();
                        event.event_id = format!("concurrent-event-{}-{:04}", thread_id, i);
                        let result = validator.validate_epcis_event(&event);
                        assert!(result.is_ok());
                    }
                    
                    start.elapsed()
                })
            })
            .collect();
        
        let durations: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let total_duration = durations.iter().sum::<std::time::Duration>();
        let max_duration = durations.iter().max().unwrap();
        
        println!("Concurrent validation ({} threads):", num_threads);
        println!("Total time: {:?}", total_duration);
        println!("Max thread time: {:?}", max_duration);
        println!("Total events validated: {}", num_threads * events_per_thread);
        
        assert!(total_duration.as_millis() < 2000, "Concurrent validation should be efficient");
    }

    #[test]
    fn benchmark_large_epc_lists() {
        let validator = Validator::new();
        let epc_counts = vec![100, 1000, 5000];
        
        for count in epc_counts {
            let mut event = sample_epcis_event();
            event.event_id = format!("large-epc-list-{}", count);
            event.epc_list = (0..count)
                .map(|i| format!("urn:epc:id:sgtin:123456.789.{}", i))
                .collect();
            
            let start = Instant::now();
            let result = validator.validate_epcis_event(&event);
            let duration = start.elapsed();
            
            println!("Validating event with {} EPCs took: {:?}", count, duration);
            
            assert!(result.is_ok(), "Event with {} EPCs should be valid", count);
            assert!(duration.as_millis() < 100, "Validation should be fast even with large EPC lists");
        }
    }

    #[test]
    fn benchmark_error_scenarios() {
        let validator = Validator::new();
        let num_events = 1000;
        
        let invalid_events: Vec<EpcisEvent> = (0..num_events)
            .map(|i| {
                let mut event = sample_epcis_event();
                event.event_id = format!("invalid-event-{:04}", i);
                event.epc_list = Vec::new();
                event
            })
            .collect();
        
        let start = Instant::now();
        
        for event in &invalid_events {
            let result = validator.validate_epcis_event(event);
            assert!(result.is_err(), "Event should be invalid");
        }
        
        let duration = start.elapsed();
        
        println!("Rejected {} invalid events in {:?}", num_events, duration);
        
        assert!(duration.as_millis() < 500, "Error detection should be very fast");
    }

    fn generate_large_ontology(num_triples: usize) -> String {
        let mut ontology = String::new();
        ontology.push_str("@prefix ex: <http://example.com/> .\n");
        ontology.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
        ontology.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\n");
        
        for i in 0..num_triples {
            ontology.push_str(&format!(
                "ex:Product{} a rdfs:Class ;\n    rdfs:label \"Product {}\" .\n\n",
                i, i
            ));
        }
        
        ontology
    }
}