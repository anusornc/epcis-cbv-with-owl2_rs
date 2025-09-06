#!/bin/bash

# Supply chain traceability example script
# Demonstrates tracking a product through the supply chain

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

echo -e "${BLUE}Supply Chain Traceability Example${NC}"
echo "======================================"

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

# Load supply chain ontology
load_supply_chain_ontology() {
    echo -e "${YELLOW}Loading supply chain ontology...${NC}"
    
    cat > /tmp/supply_chain.ttl << 'EOF'
@prefix : <http://example.org/supply-chain#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix epcis: <urn:epcglobal:epcis:> .

# Classes
:Product a owl:Class ;
    rdfs:label "Product" ;
    rdfs:comment "A product in the supply chain" .

:Shipment a owl:Class ;
    rdfs:label "Shipment" ;
    rdfs:comment "A shipment of products" .

:Location a owl:Class ;
    rdfs:label "Location" ;
    rdfs:comment "A location in the supply chain" .

:Manufacturer a owl:Class ;
    rdfs:subClassOf :Location ;
    rdfs:label "Manufacturer" .

:DistributionCenter a owl:Class ;
    rdfs:subClassOf :Location ;
    rdfs:label "Distribution Center" .

:RetailStore a owl:Class ;
    rdfs:subClassOf :Location ;
    rdfs:label "Retail Store" .

:Customer a owl:Class ;
    rdfs:subClassOf :Location ;
    rdfs:label "Customer" .

# Properties
:hasOrigin a owl:ObjectProperty ;
    rdfs:domain :Shipment ;
    rdfs:range :Location ;
    rdfs:label "has origin" .

:hasDestination a owl:ObjectProperty ;
    rdfs:domain :Shipment ;
    rdfs:range :Location ;
    rdfs:label "has destination" .

:containsProduct a owl:ObjectProperty ;
    rdfs:domain :Shipment ;
    rdfs:range :Product ;
    rdfs:label "contains product" .

:shippedFrom a owl:ObjectProperty ;
    rdfs:domain :Product ;
    rdfs:range :Location ;
    rdfs:label "shipped from" .

:shippedTo a owl:ObjectProperty ;
    rdfs:domain :Product ;
    rdfs:range :Location ;
    rdfs:label "shipped to" .

:hasTrackingNumber a owl:DatatypeProperty ;
    rdfs:domain :Shipment ;
    rdfs:range xsd:string ;
    rdfs:label "has tracking number" .

:hasShipDate a owl:DatatypeProperty ;
    rdfs:domain :Shipment ;
    rdfs:range xsd:dateTime ;
    rdfs:label "has ship date" .

:hasSKU a owl:DatatypeProperty ;
    rdfs:domain :Product ;
    rdfs:range xsd:string ;
    rdfs:label "has SKU" .

:hasSerialNumber a owl:DatatypeProperty ;
    rdfs:domain :Product ;
    rdfs:range xsd:string ;
    rdfs:label "has serial number" .

# Sample instances
:product001 a :Product ;
    :hasSKU "ELEC-001" ;
    :hasSerialNumber "SN001" .

:manufacturer1 a :Manufacturer ;
    rdfs:label "Electronics Manufacturing Co." .

:dc1 a :DistributionCenter ;
    rdfs:label "Regional Distribution Center" .

:store1 a :RetailStore ;
    rdfs:label "Downtown Electronics Store" .

:customer1 a :Customer ;
    rdfs:label "John Doe" .

:shipment1 a :Shipment ;
    :hasTrackingNumber "TRK001" ;
    :hasShipDate "2024-01-01T10:00:00Z"^^xsd:dateTime ;
    :hasOrigin :manufacturer1 ;
    :hasDestination :dc1 ;
    :containsProduct :product001 .

:shipment2 a :Shipment ;
    :hasTrackingNumber "TRK002" ;
    :hasShipDate "2024-01-02T14:00:00Z"^^xsd:dateTime ;
    :hasOrigin :dc1 ;
    :hasDestination :store1 ;
    :containsProduct :product001 .
EOF

    response=$(curl -s -X POST "${API_URL}/ontologies/load" \
        -H "Content-Type: application/json" \
        -d "{
            \"source\": \"/tmp/supply_chain.ttl\",
            \"format\": \"turtle\"
        }")
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Supply chain ontology loaded successfully${NC}"
    else
        echo -e "${RED}✗ Failed to load supply chain ontology${NC}"
        echo "$response"
        exit 1
    fi
}

# Simulate manufacturing event
manufacturing_event() {
    echo -e "${YELLOW}Simulating manufacturing event...${NC}"
    
    response=$(curl -s -X POST "${API_URL}/events/process" \
        -H "Content-Type: application/json" \
        -d '{
            "event": {
                "eventTime": "2024-01-01T09:00:00Z",
                "eventTimeZoneOffset": "+00:00",
                "epcList": ["urn:epc:id:sgtin:0614141.107346.2017"],
                "action": "ADD",
                "bizStep": "urn:epcglobal:cbv:bizstep:commissioning",
                "disposition": "urn:epcglobal:cbv:disp:active",
                "readPoint": {"id": "urn:epc:id:sgln:0614141.10000.0"},
                "bizLocation": {"id": "urn:epc:id:sgln:0614141.10000.1"}
            }
        }')
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Manufacturing event processed${NC}"
    else
        echo -e "${RED}✗ Failed to process manufacturing event${NC}"
        echo "$response"
        exit 1
    fi
}

# Simulate shipping to distribution center
shipping_to_dc() {
    echo -e "${YELLOW}Simulating shipping to distribution center...${NC}"
    
    response=$(curl -s -X POST "${API_URL}/events/process" \
        -H "Content-Type: application/json" \
        -d '{
            "event": {
                "eventTime": "2024-01-01T10:00:00Z",
                "eventTimeZoneOffset": "+00:00",
                "epcList": ["urn:epc:id:sgtin:0614141.107346.2017"],
                "action": "OBSERVE",
                "bizStep": "urn:epcglobal:cbv:bizstep:shipping",
                "disposition": "urn:epcglobal:cbv:disp:in_transit",
                "readPoint": {"id": "urn:epc:id:sgln:0614141.10000.0"},
                "bizLocation": {"id": "urn:epc:id:sgln:0614141.10000.1"},
                "bizTransactionList": [
                    {
                        "type": "urn:epcglobal:cbv:btt:po",
                        "id": "urn:epcglobal:cbv:bt:PO123456"
                    }
                ]
            }
        }')
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Shipping event processed${NC}"
    else
        echo -e "${RED}✗ Failed to process shipping event${NC}"
        echo "$response"
        exit 1
    fi
}

# Simulate receiving at distribution center
receiving_at_dc() {
    echo -e "${YELLOW}Simulating receiving at distribution center...${NC}"
    
    response=$(curl -s -X POST "${API_URL}/events/process" \
        -H "Content-Type: application/json" \
        -d '{
            "event": {
                "eventTime": "2024-01-01T15:00:00Z",
                "eventTimeZoneOffset": "+00:00",
                "epcList": ["urn:epc:id:sgtin:0614141.107346.2017"],
                "action": "OBSERVE",
                "bizStep": "urn:epcglobal:cbv:bizstep:receiving",
                "disposition": "urn:epcglobal:cbv:disp:in_progress",
                "readPoint": {"id": "urn:epc:id:sgln:0614141.20000.0"},
                "bizLocation": {"id": "urn:epc:id:sgln:0614141.20000.1"}
            }
        }')
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Receiving event processed${NC}"
    else
        echo -e "${RED}✗ Failed to process receiving event${NC}"
        echo "$response"
        exit 1
    fi
}

# Simulate shipping to retail store
shipping_to_store() {
    echo -e "${YELLOW}Simulating shipping to retail store...${NC}"
    
    response=$(curl -s -X POST "${API_URL}/events/process" \
        -H "Content-Type: application/json" \
        -d '{
            "event": {
                "eventTime": "2024-01-02T09:00:00Z",
                "eventTimeZoneOffset": "+00:00",
                "epcList": ["urn:epc:id:sgtin:0614141.107346.2017"],
                "action": "OBSERVE",
                "bizStep": "urn:epcglobal:cbv:bizstep:shipping",
                "disposition": "urn:epcglobal:cbv:disp:in_transit",
                "readPoint": {"id": "urn:epc:id:sgln:0614141.20000.0"},
                "bizLocation": {"id": "urn:epc:id:sgln:0614141.20000.1"}
            }
        }')
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Shipping to store event processed${NC}"
    else
        echo -e "${RED}✗ Failed to process shipping to store event${NC}"
        echo "$response"
        exit 1
    fi
}

# Simulate receiving at retail store
receiving_at_store() {
    echo -e "${YELLOW}Simulating receiving at retail store...${NC}"
    
    response=$(curl -s -X POST "${API_URL}/events/process" \
        -H "Content-Type: application/json" \
        -d '{
            "event": {
                "eventTime": "2024-01-02T14:00:00Z",
                "eventTimeZoneOffset": "+00:00",
                "epcList": ["urn:epc:id:sgtin:0614141.107346.2017"],
                "action": "OBSERVE",
                "bizStep": "urn:epcglobal:cbv:bizstep:receiving",
                "disposition": "urn:epcglobal:cbv:disp:available",
                "readPoint": {"id": "urn:epc:id:sgln:0614141.30000.0"},
                "bizLocation": {"id": "urn:epc:id:sgln:0614141.30000.1"}
            }
        }')
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Receiving at store event processed${NC}"
    else
        echo -e "${RED}✗ Failed to process receiving at store event${NC}"
        echo "$response"
        exit 1
    fi
}

# Simulate sale to customer
sale_to_customer() {
    echo -e "${YELLOW}Simulating sale to customer...${NC}"
    
    response=$(curl -s -X POST "${API_URL}/events/process" \
        -H "Content-Type: application/json" \
        -d '{
            "event": {
                "eventTime": "2024-01-03T16:00:00Z",
                "eventTimeZoneOffset": "+00:00",
                "epcList": ["urn:epc:id:sgtin:0614141.107346.2017"],
                "action": "OBSERVE",
                "bizStep": "urn:epcglobal:cbv:bizstep:selling",
                "disposition": "urn:epcglobal:cbv:disp:sold",
                "readPoint": {"id": "urn:epc:id:sgln:0614141.30000.0"},
                "bizLocation": {"id": "urn:epc:id:sgln:0614141.30000.1"}
            }
        }')
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Sale event processed${NC}"
    else
        echo -e "${RED}✗ Failed to process sale event${NC}"
        echo "$response"
        exit 1
    fi
}

# Perform reasoning to infer additional relationships
perform_reasoning() {
    echo -e "${YELLOW}Performing reasoning to infer supply chain relationships...${NC}"
    
    response=$(curl -s -X POST "${API_URL}/reasoning/infer" \
        -H "Content-Type: application/json" \
        -d '{
            "strategy": "full",
            "max_depth": 5
        }')
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Reasoning completed${NC}"
        inferred_triples=$(echo "$response" | jq -r '.data.inference_result.inferred_triples')
        echo "Inferred triples: $inferred_triples"
    else
        echo -e "${RED}✗ Reasoning failed${NC}"
        echo "$response"
        exit 1
    fi
}

# Query product journey
query_product_journey() {
    echo -e "${YELLOW}Querying product journey...${NC}"
    
    response=$(curl -s -X POST "${API_URL}/sparql/query" \
        -H "Content-Type: application/json" \
        -d '{
            "query": "PREFIX epcis: <urn:epcglobal:epcis:> SELECT ?eventTime ?bizStep ?readPoint WHERE { ?event epcis:epcList ?epc ; epcis:eventTime ?eventTime ; epcis:bizStep ?bizStep ; epcis:readPoint ?readPoint . FILTER(?epc = <urn:epc:id:sgtin:0614141.107346.2017>) } ORDER BY ?eventTime"
        }')
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Product journey retrieved${NC}"
        echo "Supply Chain Journey:"
        echo "$response" | jq -r '.data.results.results.bindings[] | "\(.eventTime.value) - \(.bizStep.value) at \(.readPoint.value)"' 2>/dev/null || echo "No journey data found"
    else
        echo -e "${RED}✗ Failed to query product journey${NC}"
        echo "$response"
        exit 1
    fi
}

# Query all events for the product
query_all_events() {
    echo -e "${YELLOW}Querying all events for the product...${NC}"
    
    response=$(curl -s -X POST "${API_URL}/sparql/query" \
        -H "Content-Type: application/json" \
        -d '{
            "query": "PREFIX epcis: <urn:epcglobal:epcis:> SELECT ?event ?eventTime ?action WHERE { ?event epcis:epcList ?epc ; epcis:eventTime ?eventTime ; epcis:action ?action . FILTER(?epc = <urn:epc:id:sgtin:0614141.107346.2017>) } ORDER BY ?eventTime"
        }')
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ All events retrieved${NC}"
        echo "Event Timeline:"
        echo "$response" | jq -r '.data.results.results.bindings[] | "\(.eventTime.value) - \(.action.value)"' 2>/dev/null || echo "No events found"
    else
        echo -e "${RED}✗ Failed to query events${NC}"
        echo "$response"
        exit 1
    fi
}

# Get system statistics
get_statistics() {
    echo -e "${YELLOW}Getting system statistics...${NC}"
    
    response=$(curl -s -X GET "${API_URL}/statistics")
    
    if echo "$response" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Statistics retrieved${NC}"
        total_events=$(echo "$response" | jq -r '.data.statistics.total_events // 0')
        echo "Total events processed: $total_events"
    else
        echo -e "${RED}✗ Failed to get statistics${NC}"
        echo "$response"
        exit 1
    fi
}

# Main execution
main() {
    check_server
    load_supply_chain_ontology
    manufacturing_event
    shipping_to_dc
    receiving_at_dc
    shipping_to_store
    receiving_at_store
    sale_to_customer
    perform_reasoning
    query_product_journey
    query_all_events
    get_statistics
    
    echo ""
    echo -e "${GREEN}✓ Supply chain traceability example completed successfully!${NC}"
    echo ""
    echo "Summary:"
    echo "- Product was manufactured at Electronics Manufacturing Co."
    echo "- Shipped to Regional Distribution Center"
    echo "- Received at Distribution Center"
    echo "- Shipped to Downtown Electronics Store"
    echo "- Received at Retail Store"
    echo "- Sold to Customer"
    echo ""
    echo "The product journey is now tracked in the knowledge graph!"
    echo "You can query additional information using SPARQL queries."
}

# Run main function
main "$@"