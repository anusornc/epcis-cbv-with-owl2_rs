# EPCIS Knowledge Graph User Guide

## Getting Started

Welcome to the EPCIS Knowledge Graph! This guide will help you understand how to use the system for supply chain traceability and EPCIS event processing.

## What is EPCIS Knowledge Graph?

The EPCIS Knowledge Graph is a powerful system that combines:
- **EPCIS (Electronic Product Code Information Services)** - Industry standard for supply chain event data
- **OWL 2 Reasoning** - Semantic inference and validation
- **RDF Triple Store** - Efficient graph-based storage and querying

## Quick Start

### 1. Installation

#### From Source
```bash
git clone <repository-url>
cd epcis_kg_rust
cargo build --release
```

#### Using Docker
```bash
docker build -t epcis-knowledge-graph .
docker run -p 8080:8080 epcis-knowledge-graph
```

### 2. Running the System

#### Development Mode
```bash
cargo run -- serve --config config/development.toml
```

#### Production Mode
```bash
./target/release/epcis-knowledge-graph serve --config config/production.toml
```

### 3. Basic Usage

Once the system is running, you can:

- **Web Interface**: Open http://localhost:8080 in your browser
- **REST API**: Use the API endpoints (see API documentation)
- **Command Line**: Use the CLI tools

## Common Use Cases

### 1. Supply Chain Traceability

Track products through the supply chain from manufacturer to consumer.

**Example Scenario:**
```
Manufacturer → Warehouse → Retail Store → Consumer
```

**Steps:**
1. Load EPCIS/CBV ontologies
2. Process EPCIS events at each supply chain step
3. Query the knowledge graph for product journey
4. Perform reasoning to infer additional information

### 2. Product Journey Tracking

Track individual products through their lifecycle.

**Example EPCIS Event:**
```json
{
  "eventTime": "2024-01-01T10:00:00Z",
  "eventTimeZoneOffset": "+00:00",
  "epcList": ["urn:epc:id:sgtin:0614141.107346.2017"],
  "action": "OBSERVE",
  "bizStep": "urn:epcglobal:cbv:bizstep:receiving",
  "disposition": "urn:epcglobal:cbv:disp:in_progress",
  "readPoint": {"id": "urn:epc:id:sgln:0614141.12345.0"}
}
```

### 3. EPCIS Event Validation

Validate EPCIS events against industry standards.

**Benefits:**
- Ensure compliance with GS1 standards
- Detect data quality issues
- Validate business rules
- Prevent invalid events from entering the system

### 4. Anomaly Detection

Identify unusual patterns in supply chain data.

**Example Anomalies:**
- Products missing expected locations
- Unusual time gaps between events
- Invalid state transitions
- Duplicate events

## CLI Usage

### Basic Commands

#### Load Ontology
```bash
./epcis-knowledge-graph ontology load \
  --source /path/to/epcis.ttl \
  --format turtle
```

#### Process EPCIS Event
```bash
./epcis-knowledge-graph event process \
  --event-file /path/to/event.json
```

#### Query with SPARQL
```bash
./epcis-knowledge-graph sparql query \
  --query "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"
```

#### Perform Reasoning
```bash
./epcis-knowledge-graph reasoning infer \
  --strategy incremental \
  --max-depth 3
```

### Advanced Commands

#### Performance Optimization
```bash
./epcis-knowledge-graph optimize \
  --parallel \
  --cache-limit 10000 \
  --batch-size 1000
```

#### Monitoring
```bash
./epcis-knowledge-graph monitor \
  --action metrics \
  --format json
```

## REST API Examples

### Load Ontology
```bash
curl -X POST http://localhost:8080/api/v1/ontologies/load \
  -H "Content-Type: application/json" \
  -d '{
    "source": "/path/to/epcis.ttl",
    "format": "turtle"
  }'
```

### Process Event
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

### Query Product Journey
```bash
curl -X POST http://localhost:8080/api/v1/sparql/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "PREFIX epcis: <urn:epcglobal:epcis:> SELECT ?eventTime ?bizStep ?readPoint WHERE { ?event epcis:epcList ?epc ; epcis:eventTime ?eventTime ; epcis:bizStep ?bizStep ; epcis:readPoint ?readPoint . FILTER(?epc = <urn:epc:id:sgtin:0614141.107346.2017>) } ORDER BY ?eventTime"
  }'
```

## Configuration

### Environment Variables

- `RUST_LOG`: Logging level (debug, info, warn, error)
- `EPCIS_KG_CONFIG_PATH`: Path to configuration file
- `EPCIS_KG_DATABASE_PATH`: Path to database directory
- `EPCIS_KG_PORT`: Server port (default: 8080)

### Configuration Files

The system supports TOML configuration files:

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
path = "./data"
max_connections = 100

[reasoning]
enabled = true
max_depth = 5
cache_size = 10000

[monitoring]
enabled = true
metrics_interval = 30
```

## Example Workflows

### 1. Basic Setup
```bash
# 1. Start the server
./epcis-knowledge-graph serve

# 2. Load EPCIS ontology
curl -X POST http://localhost:8080/api/v1/ontologies/load \
  -H "Content-Type: application/json" \
  -d '{
    "source": "ontologies/epcis.ttl",
    "format": "turtle"
  }'

# 3. Process an event
curl -X POST http://localhost:8080/api/v1/events/process \
  -H "Content-Type: application/json" \
  -d '{"event": {...}}'

# 4. Query the data
curl -X POST http://localhost:8080/api/v1/sparql/query \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT * WHERE { ?s ?p ?o } LIMIT 10"}'
```

### 2. Supply Chain Analysis
```bash
# 1. Load all required ontologies
./epcis-knowledge-graph ontology load --source ontologies/epcis.ttl
./epcis-knowledge-graph ontology load --source ontologies/cbv.ttl

# 2. Process batch of events
./epcis-knowledge-graph event batch --events-dir ./events/

# 3. Perform reasoning
./epcis-knowledge-graph reasoning infer --strategy full

# 4. Analyze results
./epcis-knowledge-graph sparql query --query-file analysis.sparql

# 5. Generate report
./epcis-knowledge-graph report --format json --output report.json
```

### 3. Performance Monitoring
```bash
# 1. Enable monitoring
./epcis-knowledge-graph monitor --action start

# 2. Load test data
./epcis-knowledge-graph load-test --events 10000

# 3. Check performance metrics
./epcis-knowledge-graph monitor --action metrics

# 4. Optimize if needed
./epcis-knowledge-graph optimize --parallel --cache-limit 20000
```

## Troubleshooting

### Common Issues

#### Server Won't Start
- Check if port 8080 is available
- Verify configuration file syntax
- Check database directory permissions

#### Ontology Loading Fails
- Verify ontology file format
- Check file permissions
- Ensure file is valid Turtle/RDF

#### SPARQL Query Errors
- Check query syntax
- Verify prefixes are defined
- Ensure required data exists

#### Performance Issues
- Enable performance monitoring
- Check memory usage
- Consider increasing cache size
- Use parallel processing

### Getting Help

- **API Documentation**: See `docs/API.md`
- **Developer Guide**: See `docs/DEVELOPER.md`
- **Deployment Guide**: See `docs/DEPLOYMENT.md`
- **Issues**: Report bugs and feature requests

## Best Practices

### 1. Data Management
- Regular backup of database
- Monitor disk space usage
- Use appropriate batch sizes for bulk operations

### 2. Performance
- Enable parallel processing for large datasets
- Use incremental reasoning for frequent updates
- Monitor memory usage and cache hit rates

### 3. Security
- Use authentication in production
- Validate all input data
- Monitor for unusual activity

### 4. Monitoring
- Set up regular health checks
- Monitor response times
- Set up alerts for critical issues

## Next Steps

- Explore the API documentation for advanced features
- Set up monitoring and alerting
- Deploy to production environment
- Integrate with existing supply chain systems

## Support

For support and questions:
- Check the documentation
- Review example configurations
- Monitor system logs
- Contact the development team