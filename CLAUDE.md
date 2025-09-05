# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust project to create an EPCIS Knowledge Graph demo that combines **owl2_rs** (OWL 2 reasoning library) with **Oxigraph** (RDF triple store) for supply chain traceability using EPCIS/CBV ontologies. The project incorporates research from Context7 about EPCIS 2.0, Oxigraph capabilities, and real-world implementation patterns.

## Project Status

**Phase**: Planning/Initial Setup - Enhanced with comprehensive research documentation but no code implemented yet.

## Architecture Overview

The system integrates multiple components in a modern supply chain traceability platform:
1. **EPCIS 2.0 Event Layer** - JSON-LD event capture, RESTful API, WebSocket subscriptions
2. **EPCIS/CBV Ontologies** (Turtle/RDF format) - Industry standard supply chain vocabularies
3. **owl2_rs** - OWL 2 reasoning engine for ontology validation and inference  
4. **Oxigraph** - High-performance RDF triple store with SPARQL 1.1 query capabilities
5. **Query/API Layer** - SPARQL endpoint, HTTP API, GraphQL interface

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

## Research Documentation

### Technical Research Summary
- **TECHNICAL_RESEARCH.md**: Comprehensive research summary covering EPCIS 2.0 capabilities, Oxigraph technical features, CBV vocabulary details, and integration patterns.

### Context7 Documentation References
- **OpenEPCIS**: `/openepcis/epcis-repository-ce` - Comprehensive EPCIS 2.0 implementation with JSON-LD support, RESTful APIs, and real-time event capture
- **Oxigraph**: `/oxigraph/oxigraph` - High-performance SPARQL database with excellent performance characteristics and full SPARQL 1.1 compliance

### Key Research Insights
- **EPCIS 2.0**: Native JSON-LD support, event-based communication, sensor data integration, and user extension schemas
- **Oxigraph**: Sub-100ms query performance, RocksDB storage, SPARQL 1.1 compliance, and multi-format support
- **Real-world Patterns**: Container deployment with Kafka/OpenSearch, event streaming pipelines, and production-ready monitoring

## Essential Development Commands

### Project Setup (when ready to implement)
```bash
# Initialize Rust project
cargo init

# Add enhanced dependencies (from PROJECT_PLAN.md with research insights)
cargo add oxigraph oxrdf oxsdatatypes spargebra sparesults oxrdfio oxjsonld
cargo add axum tokio tokio-tungstenite serde serde-json quick-xml thiserror anyhow tracing tracing-subscriber clap
cargo add metrics

# Add development dependencies
cargo add --dev criterion tempfile assert_cmd predicates

# Set up project structure
mkdir -p src/{ontology,storage,api,models,utils}
mkdir -p ontologies examples benches tests config docs
```

### Building and Testing (when implemented)
```bash
# Build the project
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Format code
cargo fmt

# Run linter
cargo clippy

# Run with enhanced logging
RUST_LOG=debug cargo run

# Performance testing
cargo bench -- --verbose
```

### Development Environment Setup
```bash
# Start Oxigraph server for development
docker run --rm -v $PWD/data:/data -p 7878:7878 ghcr.io/oxigraph/oxigraph serve --location /data --bind 0.0.0.0:7878

# Test SPARQL queries
curl -X POST -H 'Content-Type:application/sparql-query' \
  --data 'SELECT * WHERE { ?s ?p ?o } LIMIT 10' http://localhost:7878/query

# Load test data
curl -X POST -H 'Content-Type: text/turtle' \
  --data-binary @ontologies/epcis-ontology.ttl http://localhost:7878/store?default
```

## Key Dependencies

### Core Libraries
- **owl2_rs** (local path: `../owl2_rs`) - OWL 2 reasoning and profile checking with EL/QL/RL support
- **oxigraph** (v0.4) - High-performance RDF triple store and SPARQL 1.1 engine
- **oxrdf** (v0.4) - RDF data model with full SPARQL 1.1 support
- **spargebra** (v0.3) - SPARQL parser with algebra optimization
- **sparesults** (v0.2) - SPARQL results serialization
- **oxrdfio** (v0.4) - RDF I/O with streaming support
- **oxjsonld** (v0.1) - JSON-LD processing for EPCIS 2.0

### Web and Async
- **axum** (v0.7) - Web framework for REST API
- **tokio** (v1.0) - Async runtime with full features
- **tokio-tungstenite** (v0.20) - WebSocket support for real-time updates

### Serialization and Utilities
- **serde/serde_json** (v1.0) - JSON serialization
- **quick-xml** (v0.36) - RDF/XML parsing
- **thiserror** (v1.0) - Error handling
- **anyhow** (v1.0) - Error propagation
- **tracing** (v0.1) - Structured logging
- **clap** (v4.0) - CLI interface with derive features
- **metrics** (v0.21) - Metrics collection

### Development Tools
- **criterion** (v0.5) - Performance benchmarking
- **tempfile** (v3.8) - Test file management
- **assert_cmd** (v2.0) - CLI testing
- **predicates** (v3.0) - Predicate assertions

## Enhanced Project Structure

Based on enhanced `PROJECT_PLAN.md` with research insights:

```
src/
├── main.rs                    # CLI entry point
├── lib.rs                     # Main library interface
├── ontology/                  # Ontology management
│   ├── mod.rs                 # Ontology module
│   ├── loader.rs              # Load EPCIS/CBV ontologies
│   ├── reasoner.rs            # Integration with owl2_rs
│   └── extensions.rs          # User extension schemas
├── storage/                   # Storage management
│   ├── mod.rs                 # Storage module
│   ├── oxigraph_store.rs      # Oxigraph integration
│   ├── migration.rs           # Data migration utilities
│   └── bulk_operations.rs     # Bulk data processing
├── api/                       # REST API
│   ├── mod.rs                 # API module
│   ├── sparql.rs              # SPARQL endpoint
│   ├── routes.rs              # HTTP routes
│   ├── websocket.rs           # WebSocket subscriptions
│   └── graphql.rs             # GraphQL interface
├── models/                    # Data models
│   ├── mod.rs                 # Models module
│   ├── epcis.rs               # EPCIS-specific models
│   ├── events.rs              # Event models
│   └── reasoning.rs           # Reasoning models
├── utils/                     # Utilities
│   ├── mod.rs                 # Utils module
│   ├── conversion.rs          # Format conversion
│   ├── validation.rs          # Data validation
│   ├── metrics.rs             # Performance metrics
│   └── monitoring.rs          # System monitoring
└── cli/                       # CLI interface
    ├── mod.rs                 # CLI module
    ├── commands.rs            # CLI commands
    └── config.rs              # Configuration management

ontologies/                    # EPCIS/CBV ontologies
├── epcis-2.0-ontology.ttl      # EPCIS 2.0 ontology
├── cbv-ontology.ttl           # CBV ontology
├── extensions/                # User extensions
│   └── custom-schema.json     # Custom JSON schema

examples/                      # Example implementations
├── basic_usage.rs             # Basic usage example
├── supply_chain.rs            # Supply chain scenario
├── reasoning_demo.rs          # Reasoning demonstration
└── deployment/                # Deployment examples
    ├── docker-compose.yml     # Docker deployment
    └── kubernetes/            # Kubernetes manifests

benches/                       # Performance benchmarks
├── query_performance.rs       # Query performance tests
├── reasoning_performance.rs   # Reasoning performance tests
└── storage_performance.rs     # Storage performance tests

tests/                         # Test suites
├── integration_tests.rs       # Integration tests
├── epcis_tests.rs             # EPCIS-specific tests
├── reasoning_tests.rs         # Reasoning tests
└── performance_tests.rs       # Performance tests

config/                        # Configuration files
├── default.toml               # Default configuration
├── development.toml           # Development config
└── production.toml            # Production config

docs/                          # Documentation
├── API.md                     # API documentation
├── DEVELOPMENT.md             # Development guide
├── DEPLOYMENT.md              # Deployment guide
└── TECHNICAL_RESEARCH.md      # Technical research summary
```

## Enhanced Development Phases

The project follows a research-enhanced 4-phase development plan:

### Phase 1: Enhanced Basic Integration (Weeks 1-2)
- **1.1 Research-Driven Setup**: Initialize project with enhanced dependencies
- **1.2 EPCIS 2.0 Support**: Implement JSON-LD event processing
- **1.3 Advanced Oxigraph Integration**: Full SPARQL 1.1 support with optimization
- **1.4 Enhanced CLI**: Comprehensive CLI with SPARQL query interface

### Phase 2: Advanced Reasoning Integration (Weeks 3-4)
- **2.1 owl2_rs Deep Integration**: Advanced reasoning with EL/QL/RL profiles
- **2.2 EPCIS 2.0 Event Processing**: Full event type support with sensor data
- **2.3 Materialization Strategies**: Hybrid reasoning approaches
- **2.4 Performance Optimization**: Caching, indexing, and query optimization

### Phase 3: Production-Ready Features (Weeks 5-7)
- **3.1 Comprehensive REST API**: Full CRUD operations with WebSocket support
- **3.2 Real-world Scenarios**: Complete supply chain implementations
- **3.3 Monitoring & Metrics**: Production monitoring with Prometheus/Grafana
- **3.4 Deployment Support**: Docker, Kubernetes, and cloud deployment

### Phase 4: Production Deployment (Weeks 8-9)
- **4.1 Security & Compliance**: Authentication, authorization, and audit logging
- **4.2 Performance Testing**: Comprehensive benchmarking and optimization
- **4.3 Documentation**: Complete documentation and examples
- **4.4 Production Deployment**: Production-ready deployment with monitoring

## Enhanced Key Features

### EPCIS 2.0 Capabilities
- **Native JSON-LD Support**: Full EPCIS 2.0 JSON-LD event processing
- **Event Types**: ObjectEvent, AggregationEvent, TransactionEvent, TransformationEvent
- **Sensor Data Integration**: IoT and sensor data within events
- **User Extensions**: Custom JSON schema validation for extensions
- **Real-time Processing**: WebSocket subscriptions for live updates

### Advanced Oxigraph Features
- **SPARQL 1.1 Compliance**: Full SPARQL 1.1 query and update support
- **Performance Optimization**: Advanced query optimization and caching
- **Multi-format Support**: Turtle, RDF/XML, JSON-LD, N-Triples, N-Quads
- **Full-text Search**: Integrated text search with relevance scoring
- **ACID Transactions**: Full transaction support with rollback

### Enhanced Use Cases
- **Complete Supply Chain Traceability**: End-to-end product journey tracking
- **Real-time Anomaly Detection**: Live monitoring and alerting
- **Advanced Analytics**: Complex SPARQL queries with reasoning
- **IoT Integration**: Sensor data processing and analysis
- **Multi-tenant Support**: Named graphs for data isolation

## Important Notes

### Enhanced owl2_rs Integration
- **Profile Support**: Full EL, QL, RL profile compliance checking
- **Performance Optimization**: Optimized reasoning for large ontologies
- **Incremental Reasoning**: Support for incremental knowledge updates
- **Explanation Generation**: Detailed explanations for reasoning results

### EPCIS 2.0 Standards Implementation
- **Modern Architecture**: JSON-LD first with XML legacy support
- **Event Streaming**: Kafka-based event processing for scalability
- **Query Subscriptions**: WebSocket-based real-time subscriptions
- **Extension Framework**: Flexible user extension system with JSON schema

### Production Considerations
- **High Availability**: Multi-instance deployment with failover
- **Security**: OAuth2 authentication and role-based access control
- **Monitoring**: Comprehensive metrics collection and alerting
- **Scalability**: Horizontal scaling with load balancing

### Enhanced Performance Targets
- **Simple Queries**: < 100ms for basic SPARQL queries
- **Complex Queries**: < 1s for complex traceability queries
- **Event Processing**: 1000+ events/second throughput
- **Storage Capacity**: 10M+ triples with efficient indexing
- **Concurrent Users**: 100+ concurrent query operations

## Current Development Status

**Enhanced Planning Phase**: Project has comprehensive research documentation with detailed technical insights from Context7. Ready to begin Phase 1.1 (Research-Driven Setup) with enhanced dependencies and project structure.

**Next Steps**:
1. Initialize Cargo project with enhanced dependencies
2. Set up project structure with research insights
3. Create basic module structure with EPCIS 2.0 support
4. Implement Oxigraph integration with SPARQL 1.1 features

Refer to `PROJECT_PLAN.md` for detailed implementation guidance, `TECHNICAL_RESEARCH.md` for comprehensive research insights, and enhanced development commands above for practical implementation steps.