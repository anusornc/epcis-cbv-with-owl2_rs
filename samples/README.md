# EPCIS Knowledge Graph Sample Data

This directory contains pre-generated sample datasets for the EPCIS Knowledge Graph system. These datasets demonstrate realistic supply chain scenarios with varying scales and complexity.

## Available Datasets

### Small Dataset (`epcis_data_small.ttl`)
- **Triples**: ~150 triples
- **Entities**: 3 business entities, 3 locations, 3 products
- **Events**: 6 supply chain events
- **Use Case**: Simple demonstration and testing
- **Journey**: Basic manufacturing → shipping → warehouse → retail → sale

### Medium Dataset (`epcis_data_medium.ttl`)
- **Triples**: ~1,000+ triples
- **Entities**: 20 business entities, 30 locations, 10+ products
- **Events**: 20+ supply chain events
- **Use Case**: Development testing and small-scale demonstrations
- **Features**:
  - Multiple manufacturers, distributors, and retailers
  - Various location types (factories, warehouses, distribution centers, retail stores)
  - Complete supply chain events (manufacturing, quality control, shipping, receiving, retail)
  - Business transactions and product journeys
  - Return and recycling scenarios

### Large Dataset (`epcis_data_large.ttl`)
- **Triples**: ~10,000+ triples
- **Entities**: 100+ business entities, 200+ locations, 50+ products
- **Events**: 100+ supply chain events
- **Use Case**: Performance testing and large-scale demonstrations
- **Features**:
  - Complex multi-tier supply chain networks
  - International shipping and logistics
  - Advanced scenarios (aggregation, transformation, transactions)
  - Real-world event patterns and timing

### Extra Large Dataset (`epcis_data_xlarge.ttl`)
- **Triples**: ~100,000+ triples
- **Entities**: 500+ business entities, 1000+ locations, 200+ products
- **Events**: 500+ supply chain events
- **Use Case**: Stress testing and production-scale validation
- **Features**:
  - Enterprise-scale supply chain operations
  - High-frequency event processing
  - Complex reasoning scenarios
  - Performance benchmarking

## Data Structure

### Business Entities
- **Manufacturers**: TechCorp, Global Electronics, Smart Devices, Innovation Labs
- **Distributors**: Global Distributors, Regional Logistics, Speedy Delivery
- **Retailers**: Retail Solutions, City Electronics, Tech Superstore
- **Service Providers**: Quality Control, Logistics, Warehouse Operators

### Locations
- **Factories**: Manufacturing facilities with capacity and coordinates
- **Warehouses**: Distribution centers with storage capacity
- **Retail Stores**: Customer-facing locations with shelf space
- **Logistics**: Ports, airports, cross-docking facilities
- **Specialized**: QC labs, packaging centers, recycling facilities

### Products
- **Categories**: Electronics, smartphones, laptops, accessories
- **Attributes**: EPC codes, manufacturing dates, expiration dates, weights
- **Tracking**: Complete journey through supply chain

### Events
- **Manufacturing**: Production, quality control, testing, packaging
- **Logistics**: Shipping, transport, receiving, storage
- **Retail**: Pricing, displaying, selling, customer pickup
- **Service**: Returns, repairs, recycling, inspection

## Usage Examples

### Load Sample Data with CLI
```bash
# Start server with small sample data
cargo run -- serve --use-samples-data --samples-scale small

# Load medium sample data into existing database
cargo run -- load-samples --scale medium

# Generate custom sample data
cargo run -- generate --scale large --output-path ./custom_data/
```

### Load Sample Data Programmatically
```bash
# Load small dataset
curl -X POST -H 'Content-Type: text/turtle' \
  --data-binary @samples/epcis_data_small.ttl \
  http://localhost:8080/api/v1/load

# Load medium dataset
curl -X POST -H 'Content-Type: text/turtle' \
  --data-binary @samples/epcis_data_medium.ttl \
  http://localhost:8080/api/v1/load
```

### Query Sample Data
```sparql
# Get all products
SELECT ?product ?name ?manufacturer WHERE {
  ?product rdf:type ex:Product .
  ?product ex:name ?name .
  ?product ex:manufacturer ?manufacturer .
}

# Get supply chain journey for a product
SELECT ?event ?bizStep ?time ?location WHERE {
  ex:product1 ex:hasEvent ?event .
  ?event epcis:bizStep ?bizStep .
  ?event epcis:eventTime ?time .
  OPTIONAL { ?event epcis:bizLocation ?location }
}
ORDER BY ?time

# Get all business entities
SELECT ?entity ?name ?type WHERE {
  ?entity rdf:type ex:BusinessEntity .
  ?entity ex:name ?name .
  ?entity ex:entityType ?type .
}
```

## Sample Scenarios

### 1. Complete Product Journey
- **Product**: Smartphone Model X
- **Journey**: Factory → Quality Control → Testing → Packaging → Shipping → Warehouse → Retail Store → Sale
- **Events**: 14 events tracking the complete lifecycle

### 2. Multi-Channel Distribution
- **Scenario**: Product distributed through multiple channels
- **Path**: Manufacturer → Distributor → Multiple Retailers
- **Events**: Manufacturing, shipping, receiving, distribution, retail display

### 3. Return and Recycling
- **Scenario**: Product return and recycling process
- **Events**: Sale, customer pickup, return, inspection, recycling
- **Locations**: Retail store, returns center, recycling facility

### 4. Quality Control Workflow
- **Scenario**: Comprehensive quality testing
- **Events**: Manufacturing, quality control, testing, certification
- **Locations**: Factory, QC lab, testing facility

## Data Generation

The sample datasets are generated using the built-in data generation framework:

```rust
use epcis_knowledge_graph::data_gen::{EpcisDataGenerator, GeneratorConfig, DataScale};

let config = GeneratorConfig {
    scale: DataScale::Medium,
    output_format: OutputFormat::Turtle,
    output_path: std::path::PathBuf::from("./samples/"),
    custom_counts: None,
};

let generator = EpcisDataGenerator::new();
let result = generator.generate_dataset(&config)?;
```

## Custom Data Generation

To generate custom datasets:

```bash
# Generate with specific entity counts
cargo run -- generate \
  --locations 50 \
  --products 100 \
  --events 200 \
  --output-path ./custom_data/

# Generate large dataset
cargo run -- generate \
  --scale large \
  --format turtle \
  --output-path ./large_dataset/
```

## Performance Notes

- **Small Dataset**: < 1 second to load, suitable for quick tests
- **Medium Dataset**: 1-3 seconds to load, good for development
- **Large Dataset**: 10-30 seconds to load, for performance testing
- **XLarge Dataset**: 2-5 minutes to load, for stress testing

## Validation

All sample datasets include:
- ✅ Valid EPCIS 2.0 event structures
- ✅ Proper RDF/Turtle syntax
- ✅ Complete supply chain journeys
- ✅ Realistic business scenarios
- ✅ Consistent entity relationships
- ✅ Proper geospatial coordinates
- ✅ Valid temporal sequences

## Contributing

To add new sample datasets:
1. Use the data generation framework
2. Follow the existing naming convention
3. Include comprehensive documentation
4. Add validation tests
5. Update this README

## License

Sample data is provided for demonstration and testing purposes. See project license for usage terms.