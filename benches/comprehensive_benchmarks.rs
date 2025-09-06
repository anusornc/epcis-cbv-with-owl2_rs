use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use epcis_knowledge_graph::ontology::reasoner::{OntologyReasoner, MaterializationStrategy};
use epcis_knowledge_graph::storage::oxigraph_store::OxigraphStore;
use epcis_knowledge_graph::models::epcis::EpcisEvent;
use epcis_knowledge_graph::pipeline::EpcisEventPipeline;
use tempfile::TempDir;

fn benchmark_ontology_reasoner_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ontology_reasoner_creation");
    
    group.bench_function("default_creation", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().to_str().unwrap();
            let store = OxigraphStore::new(black_box(db_path)).unwrap();
            OntologyReasoner::with_store(store)
        })
    });
    
    group.finish();
}

fn benchmark_inference_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("inference_performance");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    group.bench_function("basic_inference", |b| {
        b.iter(|| {
            let result = reasoner.perform_inference();
            black_box(result)
        })
    });
    
    // Test with different materialization strategies
    for strategy in [
        MaterializationStrategy::Full,
        MaterializationStrategy::Incremental,
        MaterializationStrategy::OnDemand,
        MaterializationStrategy::Hybrid,
    ] {
        group.bench_with_input(
            BenchmarkId::new("materialization_strategy", format!("{:?}", strategy)),
            &strategy,
            |b, strategy| {
                b.iter(|| {
                    reasoner.set_materialization_strategy(strategy.clone());
                    let result = reasoner.perform_inference_with_materialization();
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_performance_configuration(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_configuration");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    group.bench_function("configure_performance", |b| {
        b.iter(|| {
            reasoner.configure_performance(
                black_box(true),
                black_box(10000),
                black_box(1000)
            )
        })
    });
    
    group.bench_function("optimize_performance", |b| {
        b.iter(|| {
            let result = reasoner.optimize_performance();
            black_box(result)
        })
    });
    
    group.finish();
}

fn benchmark_parallel_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_processing");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    // Sequential processing
    group.bench_function("sequential_inference", |b| {
        b.iter(|| {
            reasoner.configure_performance(false, 10000, 1000);
            let result = reasoner.perform_inference();
            black_box(result)
        })
    });
    
    // Parallel processing
    group.bench_function("parallel_inference", |b| {
        b.iter(|| {
            reasoner.configure_performance(true, 10000, 1000);
            let result = reasoner.perform_parallel_inference();
            black_box(result)
        })
    });
    
    group.finish();
}

fn benchmark_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    // Benchmark basic reasoning operations (simplified since cache methods are not available)
    group.bench_function("reasoning_operations", |b| {
        b.iter(|| {
            let result = reasoner.perform_inference();
            black_box(result)
        })
    });
    
    // Benchmark performance metrics
    group.bench_function("performance_metrics", |b| {
        b.iter(|| {
            let metrics = reasoner.get_performance_metrics();
            black_box(metrics)
        })
    });
    
    group.finish();
}

fn benchmark_materialization_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("materialization_operations");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    group.bench_function("clear_materialized_triples", |b| {
        b.iter(|| {
            reasoner.clear_materialized_triples();
        })
    });
    
    group.bench_function("get_materialized_triples", |b| {
        b.iter(|| {
            let triples = reasoner.get_materialized_triples();
            black_box(triples)
        })
    });
    
    group.finish();
}

fn benchmark_epcis_event_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("epcis_event_processing");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    // Create sample events
    let sample_events = vec![
        EpcisEvent {
            event_id: "event-001".to_string(),
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
        },
        EpcisEvent {
            event_id: "event-002".to_string(),
            event_type: "ObjectEvent".to_string(),
            event_time: "2024-01-15T11:00:00Z".to_string(),
            record_time: "2024-01-15T11:01:00Z".to_string(),
            event_action: "OBSERVE".to_string(),
            epc_list: vec![
                "urn:epc:id:sgtin:0614141.107346.2020".to_string(),
            ],
            biz_step: Some("encoding".to_string()),
            disposition: Some("in_progress".to_string()),
            biz_location: Some("urn:epc:id:sgln:0614141.00777.1".to_string()),
        },
    ];
    
    for event_count in [1, 10, 100] {
        group.bench_with_input(
            BenchmarkId::new("process_events", event_count),
            &event_count,
            |b, &count| {
                let events: Vec<_> = sample_events.iter().cycle().take(count as usize).cloned().collect();
                
                b.iter(|| {
                    // Create new store and reasoner for each iteration
                    let temp_dir = TempDir::new().unwrap();
                    let db_path = temp_dir.path().to_str().unwrap();
                    let store = OxigraphStore::new(db_path).unwrap();
                    let reasoner = OntologyReasoner::with_store(store.clone());
                    
                    let config = epcis_knowledge_graph::Config::default();
                    let mut pipeline = futures::executor::block_on(
                        EpcisEventPipeline::new(config, store, reasoner)
                    ).unwrap();
                    
                    let result = futures::executor::block_on(pipeline.process_events_batch(events.clone()));
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_sparql_query_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparql_query_simulation");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    let store = OxigraphStore::new(db_path).unwrap();
    
    // Test different query complexities
    let simple_query = "SELECT * WHERE { ?s ?p ?o } LIMIT 10";
    let complex_query = "
        PREFIX epcis: <urn:epcglobal:epcis:> 
        PREFIX cbv: <urn:epcglobal:cbv:>
        SELECT ?event ?product ?location ?time
        WHERE {
            ?event a epcis:ObjectEvent ;
                  epcis:eventTime ?time ;
                  epcis:epcList ?epc ;
                  epcis:bizStep ?step ;
                  epcis:disposition ?disp .
            ?epc epcis:hasProduct ?product .
            ?event epcis:bizLocation ?location .
            FILTER (?time > '2024-01-01T00:00:00Z'^^xsd:dateTime)
        }
        ORDER BY DESC(?time)
        LIMIT 100
    ";
    
    group.bench_function("simple_query", |b| {
        b.iter(|| {
            let result = store.query_select(black_box(simple_query));
            black_box(result)
        })
    });
    
    group.bench_function("complex_query", |b| {
        b.iter(|| {
            let result = store.query_select(black_box(complex_query));
            black_box(result)
        })
    });
    
    group.finish();
}

fn benchmark_statistics_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics_operations");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    let store = OxigraphStore::new(db_path).unwrap();
    let mut reasoner = OntologyReasoner::with_store(store);
    
    group.bench_function("reasoner_statistics", |b| {
        b.iter(|| {
            let stats = reasoner.get_detailed_stats();
            black_box(stats)
        })
    });
    
    group.bench_function("performance_metrics", |b| {
        b.iter(|| {
            let metrics = reasoner.get_performance_metrics();
            black_box(metrics)
        })
    });
    
    group.bench_function("materialization_operations", |b| {
        b.iter(|| {
            reasoner.clear_materialized_triples();
            black_box(())
        })
    });
    
    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // Test memory usage with large datasets
    for item_count in [1000, 10000, 100000] {
        group.bench_with_input(
            BenchmarkId::new("large_dataset_operations", item_count),
            &item_count,
            |b, &count| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();
                    let db_path = temp_dir.path().to_str().unwrap();
                    let store = OxigraphStore::new(db_path).unwrap();
                    let mut reasoner = OntologyReasoner::with_store(store);
                    
                    // Simulate large dataset operations
                    for i in 0..count {
                        let _key = format!("inference_{}", i);
                        let _results = vec![format!("result_{}", i)];
                        // Note: cache_inference method not available, perform other operations
                    }
                    
                    // Perform operations
                    let stats = reasoner.get_detailed_stats();
                    black_box(stats);
                    
                    // Clear to free memory
                    reasoner.clear_materialized_triples();
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    
    // Test scalability with increasing numbers of operations
    for operation_count in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("bulk_operations", operation_count),
            &operation_count,
            |b, &count| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();
                    let db_path = temp_dir.path().to_str().unwrap();
                    let store = OxigraphStore::new(db_path).unwrap();
                    let mut reasoner = OntologyReasoner::with_store(store);
                    
                    // Configure for performance
                    reasoner.configure_performance(true, count * 10, count / 10);
                    
                    // Perform bulk operations
                    for i in 0..count {
                        let _key = format!("bulk_key_{}", i);
                        let _results = vec![format!("bulk_result_{}", i)];
                        // Note: cache_inference method not available, perform other operations
                    }
                    
                    // Perform inference
                    let result = reasoner.perform_inference();
                    black_box(result);
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_ontology_reasoner_creation,
    benchmark_inference_performance,
    benchmark_performance_configuration,
    benchmark_parallel_processing,
    benchmark_cache_operations,
    benchmark_materialization_operations,
    benchmark_epcis_event_processing,
    benchmark_sparql_query_simulation,
    benchmark_statistics_operations,
    benchmark_memory_usage,
    benchmark_scalability
);
criterion_main!(benches);