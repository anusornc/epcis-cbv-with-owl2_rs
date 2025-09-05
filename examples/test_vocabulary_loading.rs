use epcis_knowledge_graph::ontology::loader::{OntologyLoader, EpcisVocabulary, CbvVocabulary};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing EPCIS/CBV Vocabulary Loading ===\n");
    
    let loader = OntologyLoader::new();
    
    // Test EPCIS vocabulary loading
    println!("1. Testing EPCIS 2.0 Vocabulary Loading");
    println!("   ----------------------------------------");
    
    match loader.load_epcis().await {
        Ok(epcis_data) => {
            println!("   ✓ EPCIS ontology loaded successfully");
            println!("   📊 Source: {} ({} triples)", epcis_data.source_file, epcis_data.triples_count);
            
            // Extract EPCIS vocabulary
            let epcis_vocab = loader.get_epcis_vocabulary(&epcis_data);
            println!("   📈 EPCIS Vocabulary Summary:");
            println!("     - Event Types: {}", epcis_vocab.event_types.len());
            println!("     - Business Steps: {}", epcis_vocab.business_steps.len());
            println!("     - Dispositions: {}", epcis_vocab.dispositions.len());
            println!("     - Properties: {}", epcis_vocab.properties.len());
            
            // Display some event types
            if !epcis_vocab.event_types.is_empty() {
                println!("   🎯 Sample Event Types:");
                for event_type in epcis_vocab.event_types.iter().take(3) {
                    match event_type {
                        epcis_knowledge_graph::ontology::loader::EpcisEventType::ObjectEvent(uri) => {
                            println!("     - ObjectEvent: {}", uri.split(':').last().unwrap_or("unknown"));
                        }
                        epcis_knowledge_graph::ontology::loader::EpcisEventType::AggregationEvent(uri) => {
                            println!("     - AggregationEvent: {}", uri.split(':').last().unwrap_or("unknown"));
                        }
                        epcis_knowledge_graph::ontology::loader::EpcisEventType::QuantityEvent(uri) => {
                            println!("     - QuantityEvent: {}", uri.split(':').last().unwrap_or("unknown"));
                        }
                        epcis_knowledge_graph::ontology::loader::EpcisEventType::TransactionEvent(uri) => {
                            println!("     - TransactionEvent: {}", uri.split(':').last().unwrap_or("unknown"));
                        }
                        epcis_knowledge_graph::ontology::loader::EpcisEventType::TransformationEvent(uri) => {
                            println!("     - TransformationEvent: {}", uri.split(':').last().unwrap_or("unknown"));
                        }
                        epcis_knowledge_graph::ontology::loader::EpcisEventType::Event(uri) => {
                            println!("     - Event: {}", uri.split(':').last().unwrap_or("unknown"));
                        }
                    }
                }
            }
            
            // Display some properties
            if !epcis_vocab.properties.is_empty() {
                println!("   🔧 Sample Properties:");
                for prop in epcis_vocab.properties.iter().take(5) {
                    println!("     - {}", prop.split(':').last().unwrap_or("unknown"));
                }
            }
        }
        Err(e) => println!("   ✗ Failed to load EPCIS ontology: {}", e),
    }
    
    println!();
    
    // Test CBV vocabulary loading
    println!("2. Testing CBV Vocabulary Loading");
    println!("   --------------------------------");
    
    match loader.load_cbv().await {
        Ok(cbv_data) => {
            println!("   ✓ CBV ontology loaded successfully");
            println!("   📊 Source: {} ({} triples)", cbv_data.source_file, cbv_data.triples_count);
            
            // Extract CBV vocabulary
            let cbv_vocab = loader.get_cbv_vocabulary(&cbv_data);
            println!("   📈 CBV Vocabulary Summary:");
            println!("     - Business Steps: {}", cbv_vocab.business_steps.len());
            println!("     - Dispositions: {}", cbv_vocab.dispositions.len());
            println!("     - Business Locations: {}", cbv_vocab.business_locations.len());
            println!("     - Business Transactions: {}", cbv_vocab.business_transactions.len());
            println!("     - Sensor Readings: {}", cbv_vocab.sensor_readings.len());
            println!("     - Actions: {}", cbv_vocab.actions.len());
            
            // Display some business steps
            if !cbv_vocab.business_steps.is_empty() {
                println!("   🏭 Sample Business Steps:");
                for step in cbv_vocab.business_steps.iter().take(5) {
                    println!("     - {}", step.split(':').last().unwrap_or("unknown"));
                }
            }
            
            // Display some dispositions
            if !cbv_vocab.dispositions.is_empty() {
                println!("   📦 Sample Dispositions:");
                for disp in cbv_vocab.dispositions.iter().take(5) {
                    println!("     - {}", disp.split(':').last().unwrap_or("unknown"));
                }
            }
            
            // Display some business locations
            if !cbv_vocab.business_locations.is_empty() {
                println!("   📍 Sample Business Locations:");
                for loc in cbv_vocab.business_locations.iter().take(5) {
                    println!("     - {}", loc.split(':').last().unwrap_or("unknown"));
                }
            }
            
            // Display some actions
            if !cbv_vocab.actions.is_empty() {
                println!("   ⚡ Sample Actions:");
                for action in cbv_vocab.actions.iter().take(3) {
                    println!("     - {}", action.split(':').last().unwrap_or("unknown"));
                }
            }
        }
        Err(e) => println!("   ✗ Failed to load CBV ontology: {}", e),
    }
    
    println!();
    
    // Test combined loading
    println!("3. Testing Combined Ontology Loading");
    println!("   -----------------------------------");
    
    let epcis_result = loader.load_epcis().await;
    let cbv_result = loader.load_cbv().await;
    
    if let (Ok(epcis_data), Ok(cbv_data)) = (epcis_result, cbv_result) {
        let total_triples = epcis_data.triples_count + cbv_data.triples_count;
        println!("   ✓ Combined loading successful");
        println!("   📊 Total triples across both ontologies: {}", total_triples);
        
        // Get combined vocabulary statistics
        let epcis_vocab = loader.get_epcis_vocabulary(&epcis_data);
        let cbv_vocab = loader.get_cbv_vocabulary(&cbv_data);
        
        println!("   📈 Combined Vocabulary Statistics:");
        println!("     - Total Event Types: {}", epcis_vocab.event_types.len());
        println!("     - Total Business Steps: {} (EPCIS) + {} (CBV) = {}", 
                 epcis_vocab.business_steps.len(), cbv_vocab.business_steps.len(),
                 epcis_vocab.business_steps.len() + cbv_vocab.business_steps.len());
        println!("     - Total Dispositions: {} (EPCIS) + {} (CBV) = {}", 
                 epcis_vocab.dispositions.len(), cbv_vocab.dispositions.len(),
                 epcis_vocab.dispositions.len() + cbv_vocab.dispositions.len());
        println!("     - Total Properties: {}", epcis_vocab.properties.len());
        println!("     - Total Business Locations: {}", cbv_vocab.business_locations.len());
        println!("     - Total Business Transactions: {}", cbv_vocab.business_transactions.len());
        println!("     - Total Sensor Readings: {}", cbv_vocab.sensor_readings.len());
        println!("     - Total Actions: {} (CBV only)", cbv_vocab.actions.len());
    } else {
        println!("   ✗ Combined loading failed - one or both ontologies could not be loaded");
    }
    
    println!("\n=== Test Complete ===");
    Ok(())
}