use super::{GeneratorConfig, GenerationResult, DataGenerator};
use crate::data_gen::entities::{LocationGenerator, ProductGenerator, BusinessEntityGenerator};
use crate::data_gen::events::EventGenerator;
use crate::data_gen::utils::formatters::{TurtleFormatter, DataFormatter};
use std::time::Instant;
use std::fs;

/// Main data generator for EPCIS knowledge graph
pub struct EpcisDataGenerator {
    location_gen: LocationGenerator,
    product_gen: ProductGenerator,
    business_gen: BusinessEntityGenerator,
    event_gen: EventGenerator,
}

impl EpcisDataGenerator {
    pub fn new() -> Self {
        Self {
            location_gen: LocationGenerator::new(),
            product_gen: ProductGenerator::new(),
            business_gen: BusinessEntityGenerator::new(),
            event_gen: EventGenerator::new(),
        }
    }

    /// Generate dataset based on configuration
    pub fn generate_dataset(&self, config: &GeneratorConfig) -> Result<GenerationResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = Instant::now();
        
        // Validate configuration
        self.validate_config(config)?;
        
        // Create output directory
        fs::create_dir_all(&config.output_path)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        
        // Calculate distribution of triples
        let total_triples = config.scale.triple_count();
        let (location_count, product_count, event_count) = self.calculate_distribution(total_triples);
        
        println!("Generating dataset with {} triples:", total_triples);
        println!("  - Locations: {}", location_count);
        println!("  - Products: {}", product_count);
        println!("  - Events: {}", event_count);
        
        // Generate entities
        let locations = self.location_gen.generate_supply_chain_network(location_count)?;
        let products = self.product_gen.generate_product_catalog(product_count)?;
        let business_entities = self.business_gen.generate_business_entities(20)?;
        
        // Generate events
        let events = self.event_gen.generate_supply_chain_events(
            &products, &locations, &business_entities, event_count
        )?;
        
        // Convert to RDF triples
        let mut all_triples = Vec::new();
        
        // Add ontology triples first
        all_triples.extend(self.generate_ontology_triples());
        
        // Add entity triples
        all_triples.extend(self.generate_entity_triples(&locations, &products, &business_entities));
        
        // Add event triples
        all_triples.extend(self.generate_event_triples(&events));
        
        // Format and save output
        let formatter = TurtleFormatter::new();
        let formatted_data = formatter.format_triples(&all_triples);
        
        let output_file = config.output_path.join(format!("epcis_data_{}.ttl", config.scale.triple_count()));
        fs::write(&output_file, formatted_data)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        
        let generation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(GenerationResult {
            triple_count: all_triples.len(),
            event_count: events.len(),
            location_count: locations.len(),
            product_count: products.len(),
            generation_time_ms: generation_time,
            output_files: vec![output_file.to_string_lossy().to_string()],
        })
    }
    
    fn calculate_distribution(&self, total_triples: usize) -> (usize, usize, usize) {
        // Distribution: 20% locations, 15% products, 65% events
        let location_triples = (total_triples as f64 * 0.20) as usize;
        let product_triples = (total_triples as f64 * 0.15) as usize;
        let event_triples = total_triples - location_triples - product_triples;
        
        // Estimate entity counts based on average triples per entity
        let location_count = location_triples / 4; // ~4 triples per location
        let product_count = product_triples / 5; // ~5 triples per product
        let event_count = event_triples / 8; // ~8 triples per event
        
        (location_count.max(10), product_count.max(20), event_count.max(50))
    }
    
    fn generate_ontology_triples(&self) -> Vec<oxrdf::Triple> {
        vec![
            // EPCIS namespace declarations
            oxrdf::Triple::new(
                oxrdf::NamedNode::new("urn:epcglobal:epcis:").unwrap(),
                oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
                oxrdf::NamedNode::new("http://www.w3.org/2002/07/owl#Ontology").unwrap(),
            ),
            // CBV namespace declarations
            oxrdf::Triple::new(
                oxrdf::NamedNode::new("urn:epcglobal:cbv:").unwrap(),
                oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
                oxrdf::NamedNode::new("http://www.w3.org/2002/07/owl#Ontology").unwrap(),
            ),
        ]
    }
    
    fn generate_entity_triples(
        &self,
        locations: &[crate::data_gen::entities::Location],
        products: &[crate::data_gen::entities::Product],
        _business_entities: &[crate::data_gen::entities::BusinessEntity],
    ) -> Vec<oxrdf::Triple> {
        let mut triples = Vec::new();
        
        // Location triples
        for location in locations {
            triples.push(oxrdf::Triple::new(
                oxrdf::NamedNode::new(&location.uri).unwrap(),
                oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
                oxrdf::NamedNode::new("http://example.com/Location").unwrap(),
            ));
            triples.push(oxrdf::Triple::new(
                oxrdf::NamedNode::new(&location.uri).unwrap(),
                oxrdf::NamedNode::new("http://example.com/name").unwrap(),
                <oxrdf::Literal as Into<oxrdf::Term>>::into(oxrdf::Literal::new_simple_literal(location.name.clone())),
            ));
            triples.push(oxrdf::Triple::new(
                oxrdf::NamedNode::new(&location.uri).unwrap(),
                oxrdf::NamedNode::new("http://example.com/locationType").unwrap(),
                <oxrdf::Literal as Into<oxrdf::Term>>::into(oxrdf::Literal::new_simple_literal(location.location_type.clone())),
            ));
        }
        
        // Product triples
        for product in products {
            triples.push(oxrdf::Triple::new(
                oxrdf::NamedNode::new(&product.uri).unwrap(),
                oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
                oxrdf::NamedNode::new("http://example.com/Product").unwrap(),
            ));
            triples.push(oxrdf::Triple::new(
                oxrdf::NamedNode::new(&product.uri).unwrap(),
                oxrdf::NamedNode::new("http://example.com/name").unwrap(),
                <oxrdf::Literal as Into<oxrdf::Term>>::into(oxrdf::Literal::new_simple_literal(product.name.clone())),
            ));
            triples.push(oxrdf::Triple::new(
                oxrdf::NamedNode::new(&product.uri).unwrap(),
                oxrdf::NamedNode::new("http://example.com/epc").unwrap(),
                <oxrdf::Literal as Into<oxrdf::Term>>::into(oxrdf::Literal::new_simple_literal(product.epc.clone())),
            ));
        }
        
        triples
    }
    
    fn generate_event_triples(&self, events: &[crate::data_gen::events::EpcisEvent]) -> Vec<oxrdf::Triple> {
        let mut triples = Vec::new();
        
        for event in events {
            triples.push(oxrdf::Triple::new(
                oxrdf::NamedNode::new(&event.uri).unwrap(),
                oxrdf::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
                oxrdf::NamedNode::new("urn:epcglobal:epcis:ObjectEvent").unwrap(),
            ));
            triples.push(oxrdf::Triple::new(
                oxrdf::NamedNode::new(&event.uri).unwrap(),
                oxrdf::NamedNode::new("urn:epcglobal:epcis:eventTime").unwrap(),
                <oxrdf::Literal as Into<oxrdf::Term>>::into(oxrdf::Literal::new_simple_literal(event.event_time.clone())),
            ));
            triples.push(oxrdf::Triple::new(
                oxrdf::NamedNode::new(&event.uri).unwrap(),
                oxrdf::NamedNode::new("urn:epcglobal:epcis:action").unwrap(),
                <oxrdf::Literal as Into<oxrdf::Term>>::into(oxrdf::Literal::new_simple_literal(event.action.clone())),
            ));
        }
        
        triples
    }
}

impl DataGenerator for EpcisDataGenerator {
    fn generate(&self, config: &GeneratorConfig) -> Result<GenerationResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.generate_dataset(config)?)
    }
    
    fn validate_config(&self, config: &GeneratorConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if config.output_path.as_os_str().is_empty() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Output path cannot be empty")) as Box<dyn std::error::Error + Send + Sync>);
        }
        
        if config.scale.triple_count() == 0 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Triple count must be greater than 0")) as Box<dyn std::error::Error + Send + Sync>);
        }
        
        Ok(())
    }
}