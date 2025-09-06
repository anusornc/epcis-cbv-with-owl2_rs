# EPCIS Knowledge Graph API Documentation

## Overview

The EPCIS Knowledge Graph provides a comprehensive REST API for managing ontologies, processing EPCIS events, performing reasoning, and querying the knowledge graph. This document describes all available API endpoints, request/response formats, and usage examples.

## Base URL

```
http://localhost:8080/api/v1
```

## Authentication

Currently, the API does not require authentication. In production environments, you should implement appropriate authentication mechanisms.

## Response Format

All API responses follow a standard JSON format:

```json
{
  "success": true,
  "data": {},
  "message": "Operation completed successfully",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

Error responses include additional error details:

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "details": {}
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Endpoints

### Health Check

#### GET /health
Check if the service is running.

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime": 3600
}
```

### Ontology Management

#### GET /ontologies
List all loaded ontologies.

**Response:**
```json
{
  "ontologies": [
    {
      "iri": "http://example.org/ontology",
      "version": "1.0",
      "triples_count": 1500,
      "loaded_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

#### POST /ontologies/load
Load an ontology from a file or URL.

**Request:**
```json
{
  "source": "/path/to/ontology.ttl",
  "format": "turtle",
  "base_iri": "http://example.org/base"
}
```

**Response:**
```json
{
  "ontology": {
    "iri": "http://example.org/ontology",
    "version": "1.0",
    "triples_count": 1500
  }
}
```

### SPARQL Operations

#### POST /sparql/query
Execute a SPARQL query.

**Request:**
```json
{
  "query": "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10",
  "default_graph": "http://example.org/graph"
}
```

**Response:**
```json
{
  "results": {
    "head": {
      "vars": ["s", "p", "o"]
    },
    "results": {
      "bindings": [
        {
          "s": {"type": "uri", "value": "http://example.org/subj1"},
          "p": {"type": "uri", "value": "http://example.org/pred1"},
          "o": {"type": "literal", "value": "example"}
        }
      ]
    }
  },
  "execution_time_ms": 15.2
}
```

#### POST /sparql/update
Execute a SPARQL update operation.

**Request:**
```json
{
  "update": "INSERT DATA { <s> <p> <o> }",
  "graph": "http://example.org/graph"
}
```

### EPCIS Event Processing

#### POST /events/validate
Validate an EPCIS event against loaded ontologies.

**Request:**
```json
{
  "event": {
    "eventTime": "2024-01-01T00:00:00Z",
    "eventTimeZoneOffset": "+00:00",
    "epcList": ["urn:epc:id:sgtin:0614141.107346.2017"],
    "action": "OBSERVE",
    "bizStep": "urn:epcglobal:cbv:bizstep:receiving",
    "disposition": "urn:epcglobal:cbv:disp:in_progress",
    "readPoint": {"id": "urn:epc:id:sgln:0614141.12345.0"}
  }
}
```

**Response:**
```json
{
  "validation_result": {
    "is_valid": true,
    "errors": [],
    "warnings": [],
    "validation_time_ms": 5.3
  }
}
```

#### POST /events/process
Process and store an EPCIS event.

**Request:** Same as /events/validate

**Response:**
```json
{
  "event_id": "event-123",
  "processing_time_ms": 12.7,
  "inferred_triples": 5
}
```

### Reasoning Operations

#### POST /reasoning/infer
Perform reasoning on loaded ontologies.

**Request:**
```json
{
  "strategy": "incremental",
  "max_depth": 3,
  "timeout_ms": 5000
}
```

**Response:**
```json
{
  "inference_result": {
    "inferred_triples": 45,
    "reasoning_time_ms": 234.5,
    "strategy_used": "incremental",
    "max_depth_reached": 3
  }
}
```

#### GET /reasoning/materialized
Get materialized triples.

**Parameters:**
- `graph` (optional): Filter by graph IRI
- `limit` (optional): Maximum number of triples to return

**Response:**
```json
{
  "materialized_triples": [
    {
      "subject": "http://example.org/subj1",
      "predicate": "http://example.org/pred1",
      "object": "http://example.org/obj1"
    }
  ],
  "count": 1
}
```

#### POST /reasoning/materialize
Materialize inference results.

**Request:**
```json
{
  "strategy": "full",
  "clear_existing": false
}
```

### Statistics and Monitoring

#### GET /statistics
Get system statistics.

**Response:**
```json
{
  "statistics": {
    "total_triples": 15234,
    "ontologies_count": 3,
    "reasoning_operations": 45,
    "sparql_queries": 1234,
    "uptime_seconds": 86400
  }
}
```

#### GET /monitoring/metrics
Get system monitoring metrics.

**Response:**
```json
{
  "metrics": {
    "uptime_seconds": 86400,
    "total_requests": 1500,
    "successful_requests": 1450,
    "failed_requests": 50,
    "avg_response_time_ms": 45.2,
    "memory_usage_mb": 256,
    "cpu_usage_percent": 12.5,
    "active_connections": 5,
    "database_metrics": {
      "total_triples": 15234,
      "query_cache_size": 100,
      "last_backup": "2024-01-01T00:00:00Z"
    },
    "reasoning_metrics": {
      "total_inferences": 45,
      "avg_inference_time_ms": 234.5,
      "cache_hit_rate": 0.85
    },
    "api_metrics": {
      "sparql_queries": 1000,
      "events_processed": 200,
      "ontology_operations": 50
    }
  }
}
```

#### GET /monitoring/alerts
Get active system alerts.

**Response:**
```json
{
  "alerts": [
    {
      "id": "alert-1",
      "type": "PERFORMANCE",
      "severity": "WARNING",
      "message": "High memory usage detected",
      "timestamp": "2024-01-01T00:00:00Z",
      "resolved": false
    }
  ]
}
```

#### POST /monitoring/alerts/clear
Clear resolved alerts.

**Request:**
```json
{
  "alert_ids": ["alert-1", "alert-2"]
}
```

## Error Codes

| Code | Description |
|------|-------------|
| `VALIDATION_ERROR` | Input validation failed |
| `ONTOLOGY_ERROR` | Ontology loading or processing error |
| `REASONING_ERROR` | Reasoning operation failed |
| `STORAGE_ERROR` | Database storage error |
| `SPARQL_ERROR` | SPARQL query execution error |
| `EVENT_ERROR` | EPCIS event processing error |
| `INTERNAL_ERROR` | Internal server error |

## Example Usage

### Load an Ontology

```bash
curl -X POST http://localhost:8080/api/v1/ontologies/load \
  -H "Content-Type: application/json" \
  -d '{
    "source": "/path/to/epcis.ttl",
    "format": "turtle"
  }'
```

### Execute SPARQL Query

```bash
curl -X POST http://localhost:8080/api/v1/sparql/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"
  }'
```

### Process EPCIS Event

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

## Rate Limiting

Currently, no rate limiting is implemented. In production, consider implementing rate limiting to prevent abuse.

## WebSocket Support

WebSocket support is not currently implemented but may be added in future versions for real-time updates.

## Version History

- **v1.0.0**: Initial API release
  - Basic ontology management
  - SPARQL query support
  - EPCIS event processing
  - Reasoning operations
  - Monitoring endpoints