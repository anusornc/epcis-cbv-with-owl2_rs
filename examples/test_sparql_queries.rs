use epcis_knowledge_graph::ontology::loader::OntologyLoader;
use epcis_knowledge_graph::storage::oxigraph_store::OxigraphStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Oxigraph Store with SPARQL queries...");
    
    // Create a new in-memory store
    let mut store = OxigraphStore::new_memory()?;
    
    // Load EPCIS ontology
    let loader = OntologyLoader::new();
    let epcis_ontology = loader.load_ontology("ontologies/epcis2.ttl")?;
    
    // Store the ontology in our Oxigraph store
    store.store_ontology_data(&epcis_ontology)?;
    
    // Load CBV ontology
    let cbv_ontology = loader.load_ontology("ontologies/cbv.ttl")?;
    store.store_ontology_data(&cbv_ontology)?;
    
    // Get store statistics
    let stats = store.get_statistics()?;
    println!("Store Statistics:");
    println!("  Total triples: {}", stats.total_quads);
    println!("  Named graphs: {}", stats.named_graphs);
    println!("  Storage path: {}", stats.storage_path);
    
    // Test a simple SPARQL SELECT query
    println!("\nTesting SPARQL SELECT query...");
    let select_query = r#"
        SELECT ?s ?p ?o
        WHERE {
            ?s ?p ?o .
        }
        LIMIT 5
    "#;
    
    let select_results = store.query_select(select_query)?;
    println!("SELECT Results:");
    println!("{}", select_results);
    
    // Test a SPARQL ASK query
    println!("\nTesting SPARQL ASK query...");
    let ask_query = r#"
        ASK WHERE {
            ?s a <urn:epcglobal:epcis:ObjectEvent> .
        }
    "#;
    
    let ask_result = store.query_ask(ask_query)?;
    println!("ASK Result: {}", ask_result);
    
    // Test a SPARQL CONSTRUCT query
    println!("\nTesting SPARQL CONSTRUCT query...");
    let construct_query = r#"
        CONSTRUCT {
            ?s ?p ?o .
        }
        WHERE {
            ?s a <urn:epcglobal:epcis:ObjectEvent> ;
               ?p ?o .
        }
        LIMIT 3
    "#;
    
    let construct_results = store.query_construct(construct_query)?;
    println!("CONSTRUCT Results:");
    println!("{}", construct_results);
    
    // Test export functionality
    println!("\nTesting export functionality...");
    let turtle_export = store.export_turtle()?;
    println!("Exported Turtle (first 500 chars):");
    println!("{}", &turtle_export[..turtle_export.len().min(500)]);
    
    println!("\nAll tests completed successfully!");
    Ok(())
}