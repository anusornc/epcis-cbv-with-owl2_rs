# EPCIS Knowledge Graph

A comprehensive Rust-based system for supply chain traceability that combines **OWL 2 reasoning** with **RDF triple store** technology to process and analyze EPCIS (Electronic Product Code Information Services) data.

## 🚀 Features

### Core Functionality
- **EPCIS Event Processing**: Process and validate EPCIS 2.0 events
- **OWL 2 Reasoning**: Advanced semantic reasoning with owl2_rs integration
- **RDF Triple Store**: Efficient storage and querying with Oxigraph
- **SPARQL Interface**: Full SPARQL 1.1 query and update support
- **Materialization**: Incremental and full materialization strategies
- **Performance Optimization**: Parallel processing and caching

### Advanced Features
- **REST API**: Comprehensive HTTP API with Axum
- **CLI Interface**: Command-line tools for all operations
- **Monitoring**: Real-time metrics and health monitoring
- **Structured Logging**: JSON and text logging formats
- **Production Ready**: Docker deployment and systemd services
- **Security**: SSL/TLS support and authentication ready

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   REST API       │    │   CLI Interface  │    │   Web UI         │
│   (Axum)         │    │   (Clap)         │    │   (Static)       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Core System   │
                    │   (lib.rs)      │
                    └─────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Ontology      │    │   Storage       │    │   Monitoring    │
│   Management    │    │   Management    │    │   & Logging     │
│   (owl2_rs)     │    │   (Oxigraph)    │    │   (System)      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 📦 Installation

### Prerequisites
- Rust 1.75+
- Cargo
- Git

### From Source
```bash
git clone <repository-url>
cd epcis_kg_rust
cargo build --release
```

### Using Docker
```bash
docker build -t epcis-knowledge-graph .
docker run -p 8080:8080 epcis-knowledge-graph
```

## 🚀 Quick Start

### 1. Start the Server
```bash
cargo run -- serve
```

### 2. Load an Ontology
```bash
curl -X POST http://localhost:8080/api/v1/ontologies/load \
  -H "Content-Type: application/json" \
  -d '{
    "source": "examples/sample_ontology.ttl",
    "format": "turtle"
  }'
```

### 3. Process an EPCIS Event
```bash
curl -X POST http://localhost:8080/api/v1/events/process \
  -H "Content-Type: application/json" \
  -d '{
    "event": {
      "eventTime": "2024-01-01T00:00:00Z",
      "eventTimeZoneOffset": "+00:00",
      "epcList": ["urn:epc:id:sgtin:0614141.107346.2017"],
      "action": "OBSERVE",
      "bizStep": "urn:epcglobal:cbv:bizstep:receiving",
      "disposition": "urn:epcglobal:cbv:disp:in_progress",
      "readPoint": {"id": "urn:epc:id:sgln:0614141.12345.0"}
    }
  }'
```

### 4. Query with SPARQL
```bash
curl -X POST http://localhost:8080/api/v1/sparql/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"
  }'
```

## 📖 Documentation

### User Guide
- [API Documentation](docs/API.md) - Complete API reference
- [User Guide](docs/USER_GUIDE.md) - Getting started and examples
- [Deployment Guide](docs/DEPLOYMENT.md) - Production deployment

### Developer Guide
- [Developer Documentation](docs/DEVELOPER.md) - Architecture and development
- [Examples](examples/) - Sample scripts and data

## 🎯 Use Cases

### Supply Chain Traceability
Track products through the entire supply chain from manufacturer to consumer with complete audit trails.

### Product Journey Analysis
Analyze the complete journey of individual products, including all handling, shipping, and storage events.

### EPCIS Compliance
Validate EPCIS events against GS1 standards and ensure compliance with industry requirements.

### Anomaly Detection
Identify unusual patterns, missing events, or inconsistencies in supply chain data.

### Real-time Monitoring
Monitor supply chain operations in real-time with customizable alerts and metrics.

## 🔧 Configuration

### Environment Variables
```bash
RUST_LOG=info                    # Logging level
EPCIS_KG_PORT=8080              # Server port
EPCIS_KG_DATABASE_PATH=./data   # Database path
```

### Configuration Files
- `config/development.toml` - Development settings
- `config/production.toml` - Production settings

## 📊 Performance

### Benchmarks
- **Simple SPARQL queries**: < 100ms
- **Complex traceability queries**: < 1s
- **EPCIS event processing**: < 50ms per event
- **Reasoning operations**: < 500ms for typical ontologies

### Scalability
- **Storage**: 10M+ triples supported
- **Concurrent users**: 1000+ connections
- **Memory usage**: Configurable, typically 1-8GB
- **CPU**: Multi-core processing with parallel reasoning

## 🐳 Docker Deployment

### Docker Compose
```bash
docker-compose up -d
```

This starts:
- EPCIS Knowledge Graph application
- PostgreSQL database (optional)
- Redis cache (optional)
- Prometheus monitoring (optional)
- Grafana dashboard (optional)

### Production Deployment
```bash
# Using deployment script
sudo ./scripts/deploy.sh

# Manual deployment
docker run -d \
  --name epcis-kg \
  -p 8080:8080 \
  -v epcis-data:/var/lib/epcis-kg/data \
  -v epcis-logs:/var/log/epcis-kg \
  epcis-knowledge-graph:latest
```

## 🔍 Examples

### Basic Usage
```bash
# Run the basic example
./examples/basic_usage.sh

# Run supply chain traceability example
./examples/supply_chain_traceability.sh
```

### CLI Usage
```bash
# Load ontology
./target/release/epcis-knowledge-graph ontology load \
  --source examples/sample_ontology.ttl \
  --format turtle

# Process events
./target/release/epcis-knowledge-graph event process \
  --event-file examples/sample_epcis_event.json

# Query data
./target/release/epcis-knowledge-graph sparql query \
  --query "SELECT * WHERE { ?s ?p ?o } LIMIT 10"

# Monitor system
./target/release/epcis-knowledge-graph monitor \
  --action metrics \
  --format json
```

## 🧪 Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
cargo test --test integration_tests
```

### Benchmarks
```bash
cargo bench
```

## 📈 Monitoring

### Health Checks
```bash
# Basic health check
curl http://localhost:8080/health

# Detailed health check
curl http://localhost:8080/api/v1/monitoring/health

# System health script
./scripts/health-check.sh
```

### Metrics
```bash
# Get system metrics
curl http://localhost:8080/api/v1/monitoring/metrics

# Get active alerts
curl http://localhost:8080/api/v1/monitoring/alerts
```

## 🔒 Security

### Features
- SSL/TLS support with Nginx reverse proxy
- User permission management
- File system security
- Backup encryption support
- Rate limiting ready

### Best Practices
- Use reverse proxy for SSL termination
- Implement authentication in production
- Regular security updates
- Monitor for unusual activity
- Use least privilege principle

## 💾 Backup and Recovery

### Automated Backups
```bash
# Daily encrypted backup
./scripts/backup.sh --encrypt --s3-bucket your-backup-bucket

# Custom retention
./scripts/backup.sh --retention-days 30
```

### Recovery
```bash
# Restore from backup
sudo systemctl stop epcis-kg
sudo -u epcis tar -xzf backup.tar.gz -C /var/lib/epcis-kg/
sudo systemctl start epcis-kg
```

## 🛠️ Development

### Setup
```bash
# Clone repository
git clone <repository-url>
cd epcis_kg_rust

# Install dependencies
cargo build

# Run development server
cargo run -- serve --config config/development.toml
```

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Run all checks
cargo fmt && cargo clippy && cargo test
```

## 📝 API Reference

### Endpoints
- `GET /health` - Health check
- `POST /api/v1/ontologies/load` - Load ontology
- `POST /api/v1/sparql/query` - SPARQL query
- `POST /api/v1/events/process` - Process EPCIS event
- `POST /api/v1/reasoning/infer` - Perform reasoning
- `GET /api/v1/monitoring/metrics` - System metrics

For complete API documentation, see [docs/API.md](docs/API.md).

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Development Standards
- Follow Rust API Guidelines
- Include comprehensive tests
- Update documentation
- Use conventional commits
- Ensure CI/CD passes

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **GS1** for EPCIS standards
- **owl2_rs** for OWL 2 reasoning capabilities
- **Oxigraph** for RDF triple store functionality
- **Axum** for web framework
- **Rust community** for excellent tooling

## 📞 Support

- **Documentation**: See the `/docs` directory
- **Issues**: Report bugs and feature requests on GitHub
- **Community**: Join discussions in the repository
- **Email**: Contact the development team

## 🗺️ Roadmap

### Phase 1 ✅
- [x] Basic project setup
- [x] Ontology loading
- [x] Oxigraph integration
- [x] Basic CLI

### Phase 2 ✅
- [x] OWL 2 reasoning integration
- [x] EPCIS event processing
- [x] Advanced reasoning features
- [x] Performance optimization

### Phase 3 ✅
- [x] REST API implementation
- [x] Supply chain scenarios
- [x] Performance optimization
- [x] API testing

### Phase 4 ✅
- [x] Comprehensive testing
- [x] Monitoring and logging
- [x] Production deployment
- [x] Documentation

### Future Enhancements
- [ ] GraphQL API support
- [ ] Real-time event streaming
- [ ] Machine learning integration
- [ ] Multi-tenant support
- [ ] Advanced analytics dashboard

---

Built with ❤️ using Rust for supply chain transparency and traceability.