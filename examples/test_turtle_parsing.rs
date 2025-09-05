use epcis_knowledge_graph::ontology::loader::OntologyLoader;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Turtle file parsing functionality...");
    
    // Create an ontology loader
    let loader = OntologyLoader::new();
    
    // Test loading the EPCIS ontology
    println!("\n=== Loading EPCIS 2.0 Ontology ===");
    let epcis_path = Path::new("ontologies/epcis2.ttl");
    
    if epcis_path.exists() {
        match loader.load_ontology(epcis_path) {
            Ok(ontology_data) => {
                println!("✓ Successfully loaded EPCIS ontology");
                println!("  - Source file: {}", ontology_data.source_file);
                println!("  - Triples count: {}", ontology_data.triples_count);
                
                // Test validation
                match loader.validate_epcis_structure(&ontology_data) {
                    Ok(_) => println!("✓ EPCIS ontology validation passed"),
                    Err(e) => println!("✗ EPCIS ontology validation failed: {}", e),
                }
                
                // Get statistics
                let stats = loader.get_statistics(&ontology_data);
                println!("  - Statistics: {} triples, {} classes, {} properties, {} individuals", 
                         stats.total_triples, stats.classes, stats.properties, stats.individuals);
            }
            Err(e) => println!("✗ Failed to load EPCIS ontology: {}", e),
        }
    } else {
        println!("✗ EPCIS ontology file not found at: {}", epcis_path.display());
    }
    
    // Test loading the CBV ontology
    println!("\n=== Loading CBV Ontology ===");
    let cbv_path = Path::new("ontologies/cbv.ttl");
    
    if cbv_path.exists() {
        match loader.load_ontology(cbv_path) {
            Ok(ontology_data) => {
                println!("✓ Successfully loaded CBV ontology");
                println!("  - Source file: {}", ontology_data.source_file);
                println!("  - Triples count: {}", ontology_data.triples_count);
                
                // Get statistics
                let stats = loader.get_statistics(&ontology_data);
                println!("  - Statistics: {} triples, {} classes, {} properties, {} individuals", 
                         stats.total_triples, stats.classes, stats.properties, stats.individuals);
            }
            Err(e) => println!("✗ Failed to load CBV ontology: {}", e),
        }
    } else {
        println!("✗ CBV ontology file not found at: {}", cbv_path.display());
    }
    
    // Test loading multiple ontologies
    println!("\n=== Loading Multiple Ontologies ===");
    let paths = vec![epcis_path, cbv_path];
    let existing_paths: Vec<_> = paths.iter().filter(|p| p.exists()).collect();
    
    if !existing_paths.is_empty() {
        match loader.load_ontologies(&existing_paths) {
            Ok(ontology_data_list) => {
                println!("✓ Successfully loaded {} ontologies", ontology_data_list.len());
                for (i, data) in ontology_data_list.iter().enumerate() {
                    println!("  - Ontology {}: {} triples from {}", 
                             i + 1, data.triples_count, data.source_file);
                }
            }
            Err(e) => println!("✗ Failed to load multiple ontologies: {}", e),
        }
    } else {
        println!("✗ No ontology files found");
    }
    
    println!("\n=== Test Complete ===");
    Ok(())
}