use epcis_knowledge_graph::api::sparql::SparqlEndpoint;
use epcis_knowledge_graph::ontology::loader::OntologyLoader;
use epcis_knowledge_graph::storage::oxigraph_store::OxigraphStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing SPARQL Endpoint...");
    
    // Create a new in-memory store
    let mut store = OxigraphStore::new_memory()?;
    
    // Load ontologies
    let loader = OntologyLoader::new();
    let epcis_ontology = loader.load_ontology("ontologies/epcis2.ttl")?;
    let cbv_ontology = loader.load_ontology("ontologies/cbv.ttl")?;
    
    // Store ontologies
    store.store_ontology_data(&epcis_ontology)?;
    store.store_ontology_data(&cbv_ontology)?;
    
    // Create SPARQL endpoint
    let endpoint = SparqlEndpoint::new(store);
    
    // Test store statistics
    println!("\nTesting store statistics...");
    let stats = endpoint.get_store_statistics()?;
    println!("Store Statistics: {}", stats);
    
    // Test SELECT query
    println!("\nTesting SELECT query...");
    let select_query = r#"
        SELECT ?s ?p ?o
        WHERE {
            ?s a <urn:epcglobal:cbv:BizStep> ;
               ?p ?o .
        }
        LIMIT 3
    "#;
    
    let select_results = endpoint.execute_query(select_query).await?;
    println!("SELECT Results: {}", select_results);
    
    // Test ASK query
    println!("\nTesting ASK query...");
    let ask_query = r#"
        ASK WHERE {
            ?s a <urn:epcglobal:epcis:ObjectEvent> .
        }
    "#;
    
    let ask_results = endpoint.execute_query(ask_query).await?;
    println!("ASK Results: {}", ask_results);
    
    // Test CONSTRUCT query
    println!("\nTesting CONSTRUCT query...");
    let construct_query = r#"
        CONSTRUCT {
            ?s ?p ?o .
        }
        WHERE {
            ?s a <urn:epcglobal:cbv:Disposition> ;
               ?p ?o .
        }
        LIMIT 2
    "#;
    
    let construct_results = endpoint.execute_query(construct_query).await?;
    println!("CONSTRUCT Results: {}", construct_results);
    
    println!("\nAll SPARQL endpoint tests completed successfully!");
    Ok(())
}