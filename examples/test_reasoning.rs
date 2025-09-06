use epcis_knowledge_graph::ontology::loader::OntologyLoader;
use epcis_knowledge_graph::ontology::reasoner::OntologyReasoner;
use epcis_knowledge_graph::storage::oxigraph_store::OxigraphStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Ontology Reasoning...");
    
    // Create a new in-memory store
    let mut store = OxigraphStore::new_memory()?;
    
    // Load ontologies
    let loader = OntologyLoader::new();
    let epcis_ontology = loader.load_ontology("ontologies/epcis2.ttl")?;
    let cbv_ontology = loader.load_ontology("ontologies/cbv.ttl")?;
    
    // Store ontologies
    store.store_ontology_data(&epcis_ontology)?;
    store.store_ontology_data(&cbv_ontology)?;
    
    // Create reasoner with store
    let mut reasoner = OntologyReasoner::with_store(store);
    
    // Test ontology validation
    println!("\nTesting ontology validation...");
    match reasoner.validate_ontology(&epcis_ontology) {
        Ok(()) => println!("✓ EPCIS ontology validation passed"),
        Err(e) => println!("✗ EPCIS ontology validation failed: {}", e),
    }
    
    match reasoner.validate_ontology(&cbv_ontology) {
        Ok(()) => println!("✓ CBV ontology validation passed"),
        Err(e) => println!("✗ CBV ontology validation failed: {}", e),
    }
    
    // Test OWL profile checking
    println!("\nTesting OWL profile checking...");
    
    // Test EL profile
    match reasoner.check_owl_profile(&cbv_ontology, "EL") {
        Ok(()) => println!("✓ OWL 2 EL profile check passed"),
        Err(e) => println!("✗ OWL 2 EL profile check failed: {}", e),
    }
    
    // Test QL profile
    match reasoner.check_owl_profile(&cbv_ontology, "QL") {
        Ok(()) => println!("✓ OWL 2 QL profile check passed"),
        Err(e) => println!("✗ OWL 2 QL profile check failed: {}", e),
    }
    
    // Test RL profile
    match reasoner.check_owl_profile(&cbv_ontology, "RL") {
        Ok(()) => println!("✓ OWL 2 RL profile check passed"),
        Err(e) => println!("✗ OWL 2 RL profile check failed: {}", e),
    }
    
    // Test inference
    println!("\nTesting inference...");
    match reasoner.perform_inference() {
        Ok(inferences) => {
            println!("✓ Inference completed successfully");
            for (i, inference) in inferences.iter().enumerate() {
                println!("  {}: {}", i + 1, inference);
            }
        },
        Err(e) => println!("✗ Inference failed: {}", e),
    }
    
    // Test reasoning statistics
    println!("\nTesting reasoning statistics...");
    match reasoner.get_reasoning_stats() {
        Ok(stats) => println!("Reasoning stats: {}", stats),
        Err(e) => println!("✗ Failed to get reasoning stats: {}", e),
    }
    
    // Test vocabulary extraction
    println!("\nTesting vocabulary extraction...");
    let epcis_vocab = loader.get_epcis_vocabulary(&epcis_ontology);
    let cbv_vocab = loader.get_cbv_vocabulary(&cbv_ontology);
    
    println!("EPCIS Vocabulary:");
    println!("  Event types: {}", epcis_vocab.event_types.len());
    println!("  Business steps: {}", epcis_vocab.business_steps.len());
    println!("  Dispositions: {}", epcis_vocab.dispositions.len());
    println!("  Properties: {}", epcis_vocab.properties.len());
    
    println!("CBV Vocabulary:");
    println!("  Business steps: {}", cbv_vocab.business_steps.len());
    println!("  Dispositions: {}", cbv_vocab.dispositions.len());
    println!("  Business locations: {}", cbv_vocab.business_locations.len());
    println!("  Business transactions: {}", cbv_vocab.business_transactions.len());
    println!("  Sensor readings: {}", cbv_vocab.sensor_readings.len());
    println!("  Actions: {}", cbv_vocab.actions.len());
    
    // Test ontology statistics
    println!("\nTesting ontology statistics...");
    let epcis_stats = loader.get_statistics(&epcis_ontology);
    let cbv_stats = loader.get_statistics(&cbv_ontology);
    
    println!("EPCIS Statistics:");
    println!("  Total triples: {}", epcis_stats.total_triples);
    println!("  Classes: {}", epcis_stats.classes);
    println!("  Properties: {}", epcis_stats.properties);
    println!("  Individuals: {}", epcis_stats.individuals);
    println!("  Source: {}", epcis_stats.source_file);
    
    println!("CBV Statistics:");
    println!("  Total triples: {}", cbv_stats.total_triples);
    println!("  Classes: {}", cbv_stats.classes);
    println!("  Properties: {}", cbv_stats.properties);
    println!("  Individuals: {}", cbv_stats.individuals);
    println!("  Source: {}", cbv_stats.source_file);
    
    println!("\nAll reasoning tests completed successfully!");
    Ok(())
}