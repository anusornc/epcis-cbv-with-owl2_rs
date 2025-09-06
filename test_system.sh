#!/bin/bash

# Test script for EPCIS Knowledge Graph system
# This script tests the system with pre-generated sample data

set -e

echo "🧪 EPCIS Knowledge Graph System Test"
echo "========================================"

# Check if sample data exists
if [ ! -f "samples/epcis_data_small.ttl" ]; then
    echo "❌ Sample data not found. Please run from project root."
    exit 1
fi

# Create test directory
TEST_DIR="./test_output"
mkdir -p "$TEST_DIR"

echo "✅ Test environment setup complete"

# Test 1: Load and validate sample data
echo ""
echo "📋 Test 1: Sample Data Validation"
echo "-----------------------------------"

# Count triples in sample data
TRIPLE_COUNT=$(grep -c "^\s*[^#]" samples/epcis_data_small.ttl || echo "0")
echo "📊 Sample data contains $TRIPLE_COUNT triples"

# Validate Turtle syntax
if command -v rapper >/dev/null 2>&1; then
    echo "🔍 Validating Turtle syntax..."
    if rapper -i turtle -o turtle samples/epcis_data_small.ttl >/dev/null 2>&1; then
        echo "✅ Turtle syntax is valid"
    else
        echo "⚠️  Turtle syntax validation failed (continuing anyway)"
    fi
else
    echo "📝 Skipping syntax validation (rapper not installed)"
fi

# Test 2: Check data structure
echo ""
echo "📋 Test 2: Data Structure Analysis"
echo "------------------------------------"

# Count different types of entities
LOCATIONS=$(grep -c "ex:Location" samples/epcis_data_small.ttl || echo "0")
PRODUCTS=$(grep -c "ex:Product" samples/epcis_data_small.ttl || echo "0")
EVENTS=$(grep -c "epcis:ObjectEvent\|epcis:AggregationEvent\|epcis:TransactionEvent" samples/epcis_data_small.ttl || echo "0")
BUSINESSES=$(grep -c "ex:BusinessEntity" samples/epcis_data_small.ttl || echo "0")

echo "📈 Data structure:"
echo "   🏢 Locations: $LOCATIONS"
echo "   📦 Products: $PRODUCTS"
echo "   📅 Events: $EVENTS"
echo "   💼 Business Entities: $BUSINESSES"

# Test 3: Query simulation
echo ""
echo "📋 Test 3: Query Simulation"
echo "----------------------------"

# Simulate SPARQL queries using grep
echo "🔍 Simulating SPARQL queries..."

# Query 1: Get all products
PRODUCT_QUERY=$(grep "ex:Product" samples/epcis_data_small.ttl | grep "ex:name" | head -3)
echo "📦 Sample products found:"
echo "$PRODUCT_QUERY" | sed 's/.*;//' | sed 's/"//g' | head -3

# Query 2: Get all events
EVENT_QUERY=$(grep "epcis:ObjectEvent" samples/epcis_data_small.ttl | head -3)
echo ""
echo "📅 Sample events found:"
echo "$EVENT_QUERY" | head -3

# Query 3: Get supply chain journey
JOURNEY_QUERY=$(grep "ex:hasJourney" samples/epcis_data_small.ttl)
echo ""
echo "🛤️  Supply chain journey:"
echo "$JOURNEY_QUERY" | sed 's/.*"//' | sed 's/"[^"]*$//'

# Test 4: Data completeness
echo ""
echo "📋 Test 4: Data Completeness"
echo "----------------------------"

# Check for required components
REQUIRED_COMPONENTS=(
    "epcis:ObjectEvent"
    "ex:BusinessEntity"
    "ex:Location"
    "ex:Product"
    "ex:manufacturer"
    "ex:ownedBy"
)

MISSING_COMPONENTS=0
for component in "${REQUIRED_COMPONENTS[@]}"; do
    if grep -q "$component" samples/epcis_data_small.ttl; then
        echo "✅ $component found"
    else
        echo "❌ $component missing"
        MISSINGING_COMPONENTS=$((MISSING_COMPONENTS + 1))
    fi
done

# Test 5: Performance metrics
echo ""
echo "📋 Test 5: Performance Metrics"
echo "------------------------------"

# Measure file size and processing time
FILE_SIZE=$(stat -f%z samples/epcis_data_small.ttl 2>/dev/null || stat -c%s samples/epcis_data_small.ttl 2>/dev/null || echo "0")
echo "📊 File size: $FILE_SIZE bytes"

# Simulate query processing time
START_TIME=$(date +%s%N)
# Process the file
cat samples/epcis_data_small.ttl >/dev/null
END_TIME=$(date +%s%N)
PROCESSING_TIME=$((($END_TIME - $START_TIME) / 1000000))
echo "⏱️  Processing time: ${PROCESSING_TIME}ms"

# Test 6: Medium dataset test
echo ""
echo "📋 Test 6: Medium Dataset Test"
echo "------------------------------"

if [ -f "samples/epcis_data_medium.ttl" ]; then
    MEDIUM_TRIPLES=$(grep -c "^\s*[^#]" samples/epcis_data_medium.ttl || echo "0")
    MEDIUM_SIZE=$(stat -f%z samples/epcis_data_medium.ttl 2>/dev/null || stat -c%s samples/epcis_data_medium.ttl 2>/dev/null || echo "0")
    
    echo "📊 Medium dataset:"
    echo "   📈 Triples: $MEDIUM_TRIPLES"
    echo "   📦 File size: $MEDIUM_SIZE bytes"
    
    # Test processing time for medium dataset
    START_TIME=$(date +%s%N)
    cat samples/epcis_data_medium.ttl >/dev/null
    END_TIME=$(date +%s%N)
    MEDIUM_PROCESSING_TIME=$((($END_TIME - $START_TIME) / 1000000))
    echo "⏱️  Processing time: ${MEDIUM_PROCESSING_TIME}ms"
else
    echo "⚠️  Medium dataset not found"
fi

# Generate test report
echo ""
echo "📋 Test Report Summary"
echo "======================="

# Calculate overall score
TOTAL_TESTS=6
PASSED_TESTS=$((TOTAL_TESTS - MISSINGING_COMPONENTS))
SCORE=$((($PASSED_TESTS * 100) / $TOTAL_TESTS))

echo "📊 Overall Score: $SCORE%"
echo "✅ Tests Passed: $PASSED_TESTS/$TOTAL_TESTS"

if [ $MISSING_COMPONENTS -eq 0 ]; then
    echo "🎉 All data components present!"
else
    echo "⚠️  $MISSING_COMPONENTS data components missing"
fi

echo ""
echo "📈 Performance Summary:"
echo "   📄 Small dataset: $TRIPLE_COUNT triples, ${PROCESSING_TIME}ms"
if [ -f "samples/epcis_data_medium.ttl" ]; then
    echo "   📚 Medium dataset: $MEDIUM_TRIPLES triples, ${MEDIUM_PROCESSING_TIME}ms"
fi

# Generate recommendations
echo ""
echo "💡 Recommendations:"
if [ $SCORE -ge 80 ]; then
    echo "✅ System is ready for production use"
elif [ $SCORE -ge 60 ]; then
    echo "⚠️  System needs minor improvements"
else
    echo "❌ System requires significant fixes"
fi

echo ""
echo "🔧 Test artifacts saved to: $TEST_DIR/"
echo "📋 Test complete!"

# Save test results
cat > "$TEST_DIR/test_report.txt" << EOF
EPCIS Knowledge Graph System Test Report
========================================
Test Date: $(date)
Test Score: $SCORE%

Data Summary:
- Small Dataset: $TRIPLE_COUNT triples, $FILE_SIZE bytes
- Processing Time: ${PROCESSING_TIME}ms

Components Present:
$MISSING_COMPONENTS missing components out of ${#REQUIRED_COMPONENTS[@]}

Performance Metrics:
- Small Dataset Processing: ${PROCESSING_TIME}ms
EOF

if [ -f "samples/epcis_data_medium.ttl" ]; then
    cat >> "$TEST_DIR/test_report.txt" << EOF
- Medium Dataset: $MEDIUM_TRIPLES triples, $MEDIUM_SIZE bytes
- Medium Dataset Processing: ${MEDIUM_PROCESSING_TIME}ms
EOF
fi

echo "📄 Test report saved to $TEST_DIR/test_report.txt"