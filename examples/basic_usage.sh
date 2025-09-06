#!/bin/bash

# Example script for basic EPCIS Knowledge Graph operations

set -e

# Configuration
SERVER_URL="http://localhost:8080"
API_URL="${SERVER_URL}/api/v1"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}EPCIS Knowledge Graph Example Script${NC}"
echo "===================================="

# Check if server is running
check_server() {
    echo -e "${YELLOW}Checking if server is running...${NC}"
    if curl -f -s "${SERVER_URL}/health" > /dev/null; then
        echo -e "${GREEN}✓ Server is running${NC}"
    else
        echo -e "${RED}✗ Server is not running${NC}"
        echo "Please start the server with: cargo run -- serve"
        exit 1
    fi
}

# Load sample ontology
load_ontology() {
    echo -e "${YELLOW}Loading sample ontology...${NC}"
    
    cat > /tmp/sample_ontology.ttl << 'EOF'
@prefix : <http://example.org/ontology#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

:Product a owl:Class ;
    rdfs:label "Product" ;
    rdfs:comment "A product in the supply chain" .

:Location a owl:Class ;
    rdfs:label "Location" ;
    rdfs:comment "A location in the supply chain" .

:hasLocation a owl:ObjectProperty ;
    rdfs:domain :Product ;
    rdfs:range :Location ;
    rdfs:label "has location" .

:locatedAt a owl:ObjectProperty ;
    rdfs:subPropertyOf :hasLocation ;
    rdfs:label "located at" .

:Warehouse a owl:Class ;
    rdfs:subClassOf :Location ;
    rdfs:label "Warehouse" .

:Store a owl:Class ;
    rdfs:subClassOf :Location ;
    rdfs:label "Store" .

:hasSKU a owl:DatatypeProperty ;
    rdfs:domain :Product ;
    rdfs:range xsd:string ;
    rdfs:label "has SKU" .
EOF

    response=$(curl -s -X POST "${API_URL}/ontologies/load" \
        -H "Content-Type: application/json" \
        -d "{
            \"source\": \"/tmp/sample_ontology.ttl\",
            \"format\": \"turtle\"
        }")
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Ontology loaded successfully${NC}"
    else
        echo -e "${RED}✗ Failed to load ontology${NC}"
        echo "$response"
        exit 1
    fi
}

# Process sample EPCIS event
process_event() {
    echo -e "${YELLOW}Processing sample EPCIS event...${NC}"
    
    response=$(curl -s -X POST "${API_URL}/events/process" \
        -H "Content-Type: application/json" \
        -d '{
            "event": {
                "eventTime": "2024-01-01T10:00:00Z",
                "eventTimeZoneOffset": "+00:00",
                "epcList": ["urn:epc:id:sgtin:0614141.107346.2017"],
                "action": "OBSERVE",
                "bizStep": "urn:epcglobal:cbv:bizstep:receiving",
                "disposition": "urn:epcglobal:cbv:disp:in_progress",
                "readPoint": {"id": "urn:epc:id:sgln:0614141.12345.0"}
            }
        }')
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Event processed successfully${NC}"
        event_id=$(echo "$response" | jq -r '.data.event_id')
        echo "Event ID: $event_id"
    else
        echo -e "${RED}✗ Failed to process event${NC}"
        echo "$response"
        exit 1
    fi
}

# Perform reasoning
perform_reasoning() {
    echo -e "${YELLOW}Performing reasoning...${NC}"
    
    response=$(curl -s -X POST "${API_URL}/reasoning/infer" \
        -H "Content-Type: application/json" \
        -d '{
            "strategy": "incremental",
            "max_depth": 3
        }')
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Reasoning completed successfully${NC}"
        inferred_triples=$(echo "$response" | jq -r '.data.inference_result.inferred_triples')
        echo "Inferred triples: $inferred_triples"
    else
        echo -e "${RED}✗ Reasoning failed${NC}"
        echo "$response"
        exit 1
    fi
}

# Query with SPARQL
query_data() {
    echo -e "${YELLOW}Querying data with SPARQL...${NC}"
    
    response=$(curl -s -X POST "${API_URL}/sparql/query" \
        -H "Content-Type: application/json" \
        -d '{
            "query": "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 5"
        }')
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Query executed successfully${NC}"
        echo "Results:"
        echo "$response" | jq -r '.data.results.results.bindings[] | "\(.s.value) \(.p.value) \(.o.value)"' 2>/dev/null || echo "No results found"
    else
        echo -e "${RED}✗ Query failed${NC}"
        echo "$response"
        exit 1
    fi
}

# Get system statistics
get_statistics() {
    echo -e "${YELLOW}Getting system statistics...${NC}"
    
    response=$(curl -s -X GET "${API_URL}/statistics")
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Statistics retrieved successfully${NC}"
        echo "$response" | jq '.data.statistics' 2>/dev/null || echo "Statistics data"
    else
        echo -e "${RED}✗ Failed to get statistics${NC}"
        echo "$response"
        exit 1
    fi
}

# Get monitoring metrics
get_metrics() {
    echo -e "${YELLOW}Getting monitoring metrics...${NC}"
    
    response=$(curl -s -X GET "${API_URL}/monitoring/metrics")
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Metrics retrieved successfully${NC}"
        echo "$response" | jq '.data.metrics | {uptime_seconds, total_requests, memory_usage_mb, cpu_usage_percent}' 2>/dev/null || echo "Metrics data"
    else
        echo -e "${RED}✗ Failed to get metrics${NC}"
        echo "$response"
        exit 1
    fi
}

# Main execution
main() {
    check_server
    load_ontology
    process_event
    perform_reasoning
    query_data
    get_statistics
    get_metrics
    
    echo ""
    echo -e "${GREEN}✓ All example operations completed successfully!${NC}"
    echo ""
    echo "Next steps:"
    echo "- Load your own ontologies using the API or CLI"
    echo "- Process your EPCIS events"
    echo "- Explore the data with SPARQL queries"
    echo "- Check the API documentation for more features"
}

# Run main function
main "$@"