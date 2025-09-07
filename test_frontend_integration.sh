#!/bin/bash

echo "🧪 EPCIS Knowledge Graph - Frontend-Backend Integration Test"
echo "============================================================"

# Start server in background
echo "🚀 Starting server..."
nohup ./target/release/epcis-knowledge-graph serve --port 8080 --use-samples-data > test_server.log 2>&1 &
SERVER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 5

# Test API endpoints
echo ""
echo "🔍 Testing API endpoints..."

# Test statistics
echo "1. Testing /api/v1/statistics..."
STATS_RESPONSE=$(curl -s http://localhost:8080/api/v1/statistics)
echo "Response: $STATS_RESPONSE"

# Test ontologies
echo ""
echo "2. Testing /api/v1/ontologies..."
ONTOLOGIES_RESPONSE=$(curl -s http://localhost:8080/api/v1/ontologies)
echo "Response: $ONTOLOGIES_RESPONSE"

# Test SPARQL query
echo ""
echo "3. Testing /api/v1/sparql/query..."
SPARQL_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d '{"query":"SELECT * WHERE { ?s ?p ?o } LIMIT 5"}' http://localhost:8080/api/v1/sparql/query)
echo "Response: $SPARQL_RESPONSE"

# Test static file serving
echo ""
echo "4. Testing static file serving..."
STATIC_RESPONSE=$(curl -s -I http://localhost:8080/static/js/main.js | head -1)
echo "Response: $STATIC_RESPONSE"

# Extract key metrics for verification
echo ""
echo "📊 Extracting key metrics..."

# Extract total triples from statistics
TOTAL_TRIPLES=$(echo "$STATS_RESPONSE" | grep -o '"total_triples":[0-9]*' | cut -d: -f2)
echo "Total Triples: $TOTAL_TRIPLES"

# Extract ontologies count
ONTOLOGIES_COUNT=$(echo "$ONTOLOGIES_RESPONSE" | grep -o '"name"' | wc -l)
echo "Ontologies Loaded: $ONTOLOGIES_COUNT"

# Extract SPARQL results count
SPARQL_RESULTS=$(echo "$SPARQL_RESPONSE" | grep -o '"s"' | wc -l)
echo "SPARQL Results: $SPARQL_RESULTS"

# Verification
echo ""
echo "✅ Verification Results..."

if [ "$TOTAL_TRIPLES" = "776" ]; then
    echo "✓ Total triples count is correct (776)"
else
    echo "✗ Total triples count is incorrect (expected 776, got $TOTAL_TRIPLES)"
fi

if [ "$ONTOLOGIES_COUNT" = "2" ]; then
    echo "✓ Ontologies count is correct (2)"
else
    echo "✗ Ontologies count is incorrect (expected 2, got $ONTOLOGIES_COUNT)"
fi

if [ "$SPARQL_RESULTS" -gt 0 ]; then
    echo "✓ SPARQL query returned results ($SPARQL_RESULTS results)"
else
    echo "✗ SPARQL query returned no results"
fi

# Clean up
echo ""
echo "🧹 Cleaning up..."
kill $SERVER_PID 2>/dev/null
rm -f test_server.log

echo ""
echo "🎉 Integration test completed!"
echo ""
echo "📋 Instructions for manual testing:"
echo "1. Start the server: ./target/release/epcis-knowledge-graph serve --port 8080 --use-samples-data"
echo "2. Open browser to: http://localhost:8080"
echo "3. Open browser developer tools (F12) to see console logs"
echo "4. Check the Dashboard - should show 776 total triples"
echo "5. Try a SPARQL query - should return real EPCIS data"
echo "6. Check the Ontology page - should show 2 loaded ontologies"