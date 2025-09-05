# EPCIS Knowledge Graph Demo Project Plan

## Executive Summary

This document outlines a comprehensive plan to create a Rust Knowledge Graph demo project that combines **owl2_rs** (OWL 2 reasoning library) with **Oxigraph** (RDF triple store) to build a powerful supply chain traceability system using EPCIS (Electronic Product Code Information Services) and CBV (Core Business Vocabulary) ontologies.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            EPCIS 2.0 Event Layer                                │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐                │
│  │   JSON-LD       │  │   REST API      │  │   WebSocket     │                │
│  │   Events        │  │   Capture       │  │   Subscriptions │                │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘                │
└─────────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           Ontology & Reasoning Layer                            │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐                │
│  │   EPCIS/CBV     │    │                 │    │                 │                │
│  │   Ontologies    │───▶│     owl2_rs     │───▶│    Oxigraph     │                │
│  │  (Turtle/RDF)   │    │   (Reasoning)   │    │  (Triple Store) │                │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘                │
│                                    │                                           │
│                            ┌───────▼───────┐                                   │
│                            │ Materialized  │                                   │
│                            │   Inferences  │                                   │
│                            └───────────────┘                                   │
└─────────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            Query & API Layer                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐                │
│  │   SPARQL        │  │   HTTP API      │  │   GraphQL       │                │
│  │   Endpoint      │  │   Interface     │  │   Interface     │                │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘                │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Enhanced Architecture Components

#### EPCIS 2.0 Event Processing
- **JSON-LD Event Capture**: Native support for EPCIS 2.0 JSON-LD format
- **RESTful API**: HTTP endpoints for event capture and querying
- **WebSocket Support**: Real-time event subscriptions and notifications
- **Event Validation**: Schema-based validation with user extension support
- **Sensor Data Integration**: Handle IoT and sensor data within events
- **Batch Processing**: Efficient bulk event processing capabilities

#### Ontology Management System
- **Multi-format Support**: Turtle, RDF/XML, JSON-LD, N-Triples, N-Quads
- **Dynamic Loading**: Hot-reload ontologies without system restart
- **Version Control**: Track ontology versions and changes
- **User Extensions**: Custom JSON schema validation for extensions
- **Profile Validation**: OWL 2 EL, QL, RL profile compliance checking
- **Reasoning Integration**: Seamless owl2_rs integration for inference

#### Storage and Query Engine
- **Oxigraph Integration**: High-performance RDF storage with SPARQL 1.1 support
- **Named Graphs**: Multi-tenancy and data isolation
- **Full-text Search**: Integrated search capabilities
- **Bulk Operations**: Efficient import/export with streaming
- **Query Optimization**: Advanced SPARQL optimization and caching
- **Transaction Support**: ACID properties for data consistency

## Required Rust Libraries

### Core Dependencies
```toml
[dependencies]
# Local owl2_rs library
owl2_rs = { path = "../owl2_rs" }

# Oxigraph ecosystem - High-performance RDF database
oxigraph = "0.4"                    # RDF triple store and SPARQL engine
oxrdf = "0.4"                       # RDF data model with full SPARQL 1.1 support
oxsdatatypes = "0.2"                 # XSD datatypes with custom timing support
spargebra = "0.3"                    # SPARQL parser with algebra optimization
sparesults = "0.2"                   # SPARQL results serialization
oxrdfio = "0.4"                      # RDF I/O with streaming support
oxjsonld = "0.1"                     # JSON-LD processing for EPCIS 2.0

# Web framework and async
axum = "0.7"                         # Web framework for REST API
tokio = { version = "1.0", features = ["full"] }  # Async runtime
tokio-tungstenite = "0.20"           # WebSocket support for real-time updates

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
quick-xml = "0.36"                   # RDF/XML parsing
toml = "0.8"

# Error handling and utilities
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# CLI and configuration
clap = { version = "4.0", features = ["derive"] }

# Performance and monitoring
criterion = "0.5"                    # Performance benchmarking
metrics = "0.21"                     # Metrics collection
```

### Oxigraph Technical Capabilities

#### Performance Characteristics
- **Query Performance**: Sub-100ms response times for simple SPARQL queries
- **Storage Efficiency**: RocksDB-based storage with compression and caching
- **Memory Usage**: Configurable memory limits with efficient data structures
- **Concurrency**: Multi-threaded query processing with lock-free algorithms
- **Streaming Support**: Efficient bulk data processing with minimal memory overhead

#### SPARQL 1.1 Compliance
- **Full SPARQL 1.1 Query Language**: SELECT, CONSTRUCT, ASK, DESCRIBE
- **SPARQL 1.1 Update**: INSERT DATA, DELETE DATA, LOAD, CLEAR operations
- **Property Paths**: Complex path expressions (`ex:p+`, `ex:p?`, `ex:p*`)
- **Aggregation Functions**: COUNT, SUM, AVG, MIN, MAX, GROUP_CONCAT
- **Subqueries**: Nested queries and correlated subqueries
- **Federated Queries**: SERVICE clauses for external endpoint queries
- **Graph Management**: Named graphs, default graphs, and union graphs

#### Storage Engine Features
- **ACID Transactions**: Full transaction support with rollback capabilities
- **Multi-format Support**: Turtle, RDF/XML, JSON-LD, N-Triples, N-Quads, TriG
- **Bulk Operations**: High-performance bulk load and export operations
- **Indexing Strategies**: Automatic index selection and optimization
- **Compression**: Efficient storage compression for large datasets
- **Backup/Recovery**: Point-in-time recovery and backup utilities

#### Advanced Features
- **Query Optimization**: Advanced algebraic optimization with cost-based planning
- **Full-text Search**: Integrated text search with relevance scoring
- **Geospatial Support**: Spatial querying capabilities (with extensions)
- **Temporal Reasoning**: Time-based queries and temporal data handling
- **Custom Functions**: Extension points for user-defined functions
- **HTTP API**: Built-in SPARQL endpoint with CORS and authentication support

### EPCIS 2.0 and CBV Standards Integration

#### EPCIS 2.0 Standard Capabilities
- **Modern Event Format**: JSON-LD as primary format with XML legacy support
- **Event Types**: ObjectEvent, AggregationEvent, TransactionEvent, TransformationEvent
- **Sensor Data**: Integrated sensor and measurement data support
- **WebVocabulary**: Built-in vocabulary for common business concepts
- **Extension Mechanisms**: User-defined extensions with JSON schema validation
- **Query Language**: SPARQL-based querying with EPCIS-specific extensions
- **Capture Interface**: RESTful API for event capture with authentication
- **Subscription Service**: WebSocket-based real-time event subscriptions

#### Core Business Vocabulary (CBV)
- **Business Steps**: Standardized process identifiers (COMMISSIONING, RECEPTION, etc.)
- **Dispositions**: Product status indicators (ACTIVE, INACTIVE, DESTROYED, etc.)
- **Business Locations**: Standardized location identification and classification
- **Party Roles**: Organization and role definitions in supply chain
- **Product Classifications**: Hierarchical product categorization systems
- **Transaction Types**: Standard transaction identifiers and workflows
- **Measurement Types**: Standardized measurement units and dimensions
- **Certification Information**: Product certification and compliance data

#### GS1 Standards Integration
- **EPC Coding**: Support for SGTIN, SSCC, GRAI, GIAI, GID, etc.
- **GLN Identification**: Global Location Number support for locations
- **GTIN Standards**: Global Trade Item Number integration
- **EPCIS URIs**: Standard URI patterns for EPCIS resources
- **CBV Extensions**: Industry-specific vocabulary extensions
- **Digital Link**: GS1 Digital Link URI scheme support

### Real-World Implementation Patterns

#### OpenEPCIS Integration Patterns
- **Container Deployment**: Docker/Podman-based deployment with Kafka and OpenSearch
- **Event Capture Pipeline**: RESTful API → Kafka → Processing → Storage
- **Query Architecture**: SPARQL endpoint → OpenSearch indexing → Web interface
- **Real-time Processing**: WebSocket subscriptions for live event notifications
- **Test Data Generation**: Synthetic event generation for development and testing
- **Multi-tenant Support**: Named graphs for data isolation and security

#### Production Deployment Scenarios
- **High-Availability Setup**: Load-balanced Oxigraph instances with replication
- **Event Stream Processing**: Kafka-based event ingestion with backpressure handling
- **Caching Strategy**: Redis caching for frequent query results and ontologies
- **Monitoring Stack**: Prometheus metrics + Grafana dashboards + Alerting
- **Security Model**: OAuth2 authentication + Role-based access control
- **Backup Strategy**: Incremental backups with point-in-time recovery

#### Integration Examples
```bash
# Example: Capture EPCIS event (OpenEPCIS style)
curl -X POST http://localhost:8080/capture \
  -H "Content-Type: application/ld+json" \
  -d @epcis_event.jsonld

# Example: Query product journey
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/sparql-query" \
  -H "Accept: application/sparql-results+json" \
  --data 'SELECT ?event ?location ?time WHERE {
    ?event epcis:epcList <urn:epc:id:sgtin:123456.789.100> .
    ?event epcis:bizLocation ?location .
    ?event epcis:eventTime ?time .
  } ORDER BY ?time'

# Example: Register user extension schema
curl -X POST 'http://localhost:8080/userExtension/jsonSchema?namespace=http://example.com/' \
  -H 'Content-Type: application/json' \
  -d '{
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "CustomExtension",
    "properties": {
      "temperature": {"type": "number"},
      "humidity": {"type": "number"}
    }
  }'
```

#### Supply Chain Use Case Implementation
```json
{
  "@context": ["https://ref.gs1.org/standards/epcis/2.0.0/epcis-context.jsonld"],
  "type": "ObjectEvent",
  "eventTime": "2025-01-01T10:00:00.000Z",
  "eventTimeZoneOffset": "+00:00",
  "epcList": ["urn:epc:id:sgtin:123456.789.100"],
  "action": "ADD",
  "bizStep": "urn:epcglobal:cbv:bizstep:commissioning",
  "disposition": "urn:epcglobal:cbv:disp:active",
  "bizLocation": {
    "id": "urn:epc:id:sgln:123456.789.0"
  },
  "sensorElementList": [{
    "sensorMetadata": {"time": "2025-01-01T10:00:00.000Z"},
    "sensorReport": [{
      "type": "Temperature",
      "value": 25.5,
      "uom": "CEL"
    }]
  }]
}
```

### Development Dependencies
assert_cmd = "2.0"                   # CLI testing
predicates = "3.0"                   # Predicate assertions
```

## Project Structure

```
epcis-knowledge-graph/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs                    # CLI entry point
│   ├── lib.rs                     # Main library interface
│   ├── ontology/
│   │   ├── mod.rs                 # Ontology management
│   │   ├── loader.rs              # Load EPCIS/CBV ontologies
│   │   └── reasoner.rs            # Integration with owl2_rs
│   ├── storage/
│   │   ├── mod.rs                 # Storage management
│   │   ├── oxigraph_store.rs      # Oxigraph integration
│   │   └── migration.rs           # Data migration utilities
│   ├── api/
│   │   ├── mod.rs                 # REST API
│   │   ├── sparql.rs              # SPARQL endpoint
│   │   └── routes.rs              # HTTP routes
│   ├── models/
│   │   ├── mod.rs                 # Data models
│   │   ├── epcis.rs               # EPCIS-specific models
│   │   └── events.rs              # Event models
│   └── utils/
│       ├── mod.rs                 # Utilities
│       ├── conversion.rs          # Format conversion utilities
│       └── validation.rs          # Data validation
├── ontologies/
│   ├── epcis-ontology.ttl         # EPCIS ontology (Turtle format)
│   └── cbv-ontology.ttl           # CBV ontology (Turtle format)
├── examples/
│   ├── basic_usage.rs             # Basic usage example
│   ├── supply_chain.rs            # Supply chain scenario
│   └── reasoning_demo.rs          # Reasoning demonstration
├── benches/
│   └── performance.rs             # Performance benchmarks
├── tests/
│   ├── integration_tests.rs
│   └── epcis_tests.rs
├── config/
│   └── default.toml               # Default configuration
└── docs/
    ├── API.md
    ├── DEVELOPMENT.md
    └── DEPLOYMENT.md
```

## Key Features

### 1. EPCIS 2.0 Integration
- **Native JSON-LD support** for modern EPCIS 2.0 events
- **RESTful API integration** compatible with OpenEPCIS standards
- **Event-based communication** for real-time supply chain data
- **Sensor data and IoT integration** for comprehensive tracking
- **Web-based capture endpoints** for easy event submission
- **Query-based subscription** support for real-time notifications

### 2. Ontology Management
- **Load EPCIS/CBV ontologies** from Turtle/RDF files
- **Validate ontology compliance** with OWL 2 profiles using owl2_rs
- **Perform reasoning** to infer new knowledge
- **Convert between RDF representations** (Turtle, RDF/XML, JSON-LD)
- **User extension schemas** for custom business vocabularies
- **Dynamic ontology loading** and hot-reload capabilities

### 3. Knowledge Graph Storage
- **Store ontologies and instance data** in Oxigraph
- **Support bulk data import/export** with streaming capabilities
- **Efficient SPARQL querying** with optimization
- **Transaction support** for data consistency
- **Named graph management** for multi-tenancy
- **Full-text search** capabilities on stored data

### 4. Reasoning Integration
- **Use owl2_rs for OWL 2 reasoning** with EL/QL/RL profile support
- **Materialize inferred triples** back to Oxigraph
- **Support incremental reasoning** for large datasets
- **Generate explanations** for inferences and rule violations
- **Custom rule engines** for business logic validation
- **Performance-optimized reasoning** with caching strategies

### 5. Supply Chain Use Cases
- **Track products** through complete supply chain journeys
- **Validate EPCIS events** against business rules and ontologies
- **Perform traceability queries** with complex patterns
- **Detect anomalies** and inconsistencies in real-time
- **Generate synthetic test data** for development and testing
- **Support multiple event types** (ObjectEvent, AggregationEvent, TransactionEvent, TransformationEvent)

## Development Phases

### Phase 1: Basic Integration (Weeks 1-2)

#### 1.1 Project Setup
- [ ] Initialize Cargo project with proper structure
- [ ] Set up dependencies and build configuration
- [ ] Create basic CLI interface
- [ ] Set up development environment and tooling

#### 1.2 Ontology Loading
- [ ] Implement basic Turtle/RDF parser
- [ ] Create ontology loader module
- [ ] Add support for EPCIS/CBV ontology files
- [ ] Implement basic ontology validation

#### 1.3 Oxigraph Integration
- [ ] Set up Oxigraph store connection
- [ ] Implement basic triple storage
- [ ] Create simple SPARQL query interface
- [ ] Add data import/export functionality

#### 1.4 Basic CLI
- [ ] Create CLI commands for ontology loading
- [ ] Add basic query capabilities
- [ ] Implement configuration management
- [ ] Add logging and error handling

### Phase 2: Reasoning Integration (Weeks 3-4)

#### 2.1 owl2_rs Integration
- [ ] Integrate owl2_rs reasoning engine
- [ ] Implement ontology conversion for owl2_rs
- [ ] Add reasoning pipeline
- [ ] Create materialization process

#### 2.2 EPCIS Event Processing
- [ ] Implement EPCIS event parsing
- [ ] Add event validation against ontologies
- [ ] Create event storage mechanism
- [ ] Implement basic event queries

#### 2.3 Advanced Reasoning
- [ ] Add incremental reasoning support
- [ ] Implement rule-based validation
- [ ] Create explanation generation
- [ ] Add performance optimizations

#### 2.4 Query Enhancement
- [ ] Extend SPARQL endpoint
- [ ] Add complex query patterns
- [ ] Implement pagination and filtering
- [ ] Add query result formatting

### Phase 3: Advanced Features (Weeks 5-7)

#### 3.1 REST API
- [ ] Set up Axum web framework
- [ ] Implement HTTP routes for all operations
- [ ] Add authentication and authorization
- [ ] Create API documentation

#### 3.2 Supply Chain Scenarios
- [ ] Implement end-to-end supply chain tracking
- [ ] Add traceability queries
- [ ] Create anomaly detection
- [ ] Implement business rule validation

#### 3.3 Performance Optimization
- [ ] Add caching mechanisms
- [ ] Implement query optimization
- [ ] Add connection pooling
- [ ] Create performance monitoring

#### 3.4 Data Integration
- [ ] Add support for multiple data formats
- [ ] Implement batch processing
- [ ] Add data transformation utilities
- [ ] Create integration tests

### Phase 4: Production Features (Weeks 8-9)

#### 4.1 Monitoring and Observability
- [ ] Add metrics collection
- [ ] Implement health checks
- [ ] Add distributed tracing
- [ ] Create monitoring dashboard

#### 4.2 Production Readiness
- [ ] Implement proper error handling
- [ ] Add data backup and recovery
- [ ] Create deployment scripts
- [ ] Add security features

#### 4.3 Testing and Quality
- [ ] Create comprehensive test suite
- [ ] Add integration tests
- [ ] Implement performance benchmarks
- [ ] Add code quality checks

#### 4.4 Documentation and Examples
- [ ] Create user documentation
- [ ] Add comprehensive examples
- [ ] Write API documentation
- [ ] Create deployment guides

## Sample Implementation

### Core Library Structure
```rust
pub struct EpcisKnowledgeGraph {
    ontology_loader: OntologyLoader,
    reasoner: Owl2Reasoner,
    store: OxigraphStore,
    api_server: ApiServer,
}

impl EpcisKnowledgeGraph {
    pub async fn new(config: Config) -> Result<Self> {
        // Initialize components
        let store = OxigraphStore::new(&config.database_path)?;
        let ontology_loader = OntologyLoader::new();
        let reasoner = Owl2Reasoner::new();
        
        // Load ontologies
        let epcis_ontology = ontology_loader.load_epcis()?;
        let cbv_ontology = ontology_loader.load_cbv()?;
        
        // Perform reasoning
        reasoner.load_ontology(&epcis_ontology)?;
        reasoner.load_ontology(&cbv_ontology)?;
        
        // Store in Oxigraph
        store.store_ontology(&epcis_ontology)?;
        store.store_ontology(&cbv_ontology)?;
        
        Ok(Self {
            ontology_loader,
            reasoner,
            store,
            api_server: ApiServer::new(config.port),
        })
    }
    
    pub async fn run(&self) -> Result<()> {
        self.api_server.start().await
    }
}
```

### Key Use Cases

#### 1. Supply Chain Traceability
```rust
// Track product through supply chain
let product_id = "urn:epc:id:sgtin:123456.789.100";
let journey = kg.track_product_journey(product_id).await?;

// Validate EPCIS events
let event = parse_epcis_event(event_data)?;
let validation_result = kg.validate_event(&event).await?;
```

#### 2. SPARQL Queries
```rust
// Find all products in a specific location
let query = r#"
    PREFIX epcis: <urn:epcglobal:epcis:>
    SELECT ?product WHERE {
        ?event epcis:action "ADD" .
        ?event epcis:bizLocation ?location .
        ?location epcis:id "urn:epc:id:sgln:123456.789.0" .
        ?event epcis:epcList ?product .
    }
"#;
let results = kg.sparql_query(query).await?;
```

#### 3. Reasoning and Inference
```rust
// Perform reasoning and get inferred knowledge
let inferred_triples = kg.reason_and_materialize().await?;

// Generate explanations for inferences
let explanation = kg.explain_inference(&inferred_triple).await?;
```

## Performance Targets

### Query Performance
- **Simple SPARQL queries**: < 100ms
- **Complex traceability queries**: < 1s
- **Reasoning operations**: < 5s for large ontologies

### Scalability
- **Storage**: Handle 10M+ triples
- **Concurrent users**: Support 100+ concurrent queries
- **Memory usage**: < 4GB for typical workloads

### Data Processing
- **Ontology loading**: < 10s for EPCIS/CBV
- **Event processing**: 1000+ events/second
- **Batch imports**: 100K triples/minute

## Success Metrics

### Technical Metrics
- [ ] All unit tests passing (> 95% coverage)
- [ ] All integration tests passing
- [ ] Performance benchmarks met
- [ ] Zero critical bugs in production

### Functional Metrics
- [ ] Complete EPCIS/CBV ontology support
- [ ] End-to-end supply chain demo working
- [ ] SPARQL endpoint fully functional
- [ ] Reasoning integration complete

### Quality Metrics
- [ ] Comprehensive documentation
- [ ] Performance benchmarks established
- [ ] Security audit completed
- [ ] Deployment guide available

## Next Steps

1. **Immediate Actions**:
   - Set up project repository
   - Download EPCIS/CBV ontology files
   - Implement basic project structure

2. **Week 1 Focus**:
   - Complete Phase 1.1 (Project Setup)
   - Start Phase 1.2 (Ontology Loading)
   - Create initial CLI interface

3. **Week 2 Focus**:
   - Complete Phase 1 (Basic Integration)
   - Start Phase 2 (Reasoning Integration)
   - Prepare first working prototype

## Conclusion

This demo project will showcase the power of combining owl2_rs's reasoning capabilities with Oxigraph's scalable storage to create a practical supply chain knowledge graph. The phased approach ensures steady progress while building a robust, production-ready system.

The project will serve as both a demonstration of owl2_rs capabilities and a practical tool for supply chain traceability using industry-standard EPCIS/CBV ontologies.