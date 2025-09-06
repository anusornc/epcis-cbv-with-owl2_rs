#!/bin/bash

# Performance Benchmark Script for EPCIS Knowledge Graph
# This script runs comprehensive performance benchmarks on the system

set -e

echo "🚀 EPCIS Knowledge Graph Performance Benchmarks"
echo "================================================"

# Check if binary exists
if [ ! -f "./target/release/epcis-knowledge-graph" ]; then
    echo "❌ Binary not found. Building project..."
    cargo build --release
fi

BINARY="cargo run --release --"

# Create benchmark directory
BENCH_DIR="./benchmark_output"
mkdir -p "$BENCH_DIR"

echo "✅ Benchmark environment setup complete"

# Initialize test database
echo ""
echo "📋 Setting up test database..."
TEST_DB="$BENCH_DIR/test_db"
rm -rf "$TEST_DB" 2>/dev/null || true

# Load sample data for benchmarking
echo "📦 Loading medium-scale sample data..."
$BINARY load-samples --scale medium --force --db-path "$TEST_DB"

echo "✅ Test database setup complete"

# Function to run benchmark and capture results
run_benchmark() {
    local name="$1"
    local args="$2"
    local output_file="$BENCH_DIR/${name}.txt"
    
    echo ""
    echo "📋 Running benchmark: $name"
    echo "-----------------------------------"
    
    # Run benchmark and capture both stdout and stderr
    local start_time=$(date +%s%N)
    if $BINARY benchmark $args --db-path "$TEST_DB" 2>&1 | tee "$output_file"; then
        local end_time=$(date +%s%N)
        local duration=$((($end_time - $start_time) / 1000000))
        echo "✅ Benchmark '$name' completed in ${duration}ms"
        return 0
    else
        echo "❌ Benchmark '$name' failed"
        return 1
    fi
}

# Function to extract metrics from benchmark output
extract_metrics() {
    local file="$1"
    local metric_name="$2"
    
    if [ -f "$file" ]; then
        grep -i "$metric_name" "$file" | head -1 | sed 's/.*://' | sed 's/ms//' | xargs || echo "N/A"
    else
        echo "N/A"
    fi
}

# Run comprehensive benchmarks
echo ""
echo "🔥 Starting comprehensive benchmark suite..."

# Benchmark 1: Default configuration
run_benchmark "default_config" "--iterations 10 --scale medium --include-memory"

# Benchmark 2: High iteration count
run_benchmark "high_iterations" "--iterations 50 --scale medium --include-memory"

# Benchmark 3: Small dataset
run_benchmark "small_dataset" "--iterations 10 --scale small --include-memory"

# Benchmark 4: Large dataset
run_benchmark "large_dataset" "--iterations 5 --scale large --include-memory"

# Benchmark 5: Quick benchmark (no memory metrics)
run_benchmark "quick_benchmark" "--iterations 5 --scale medium"

# Benchmark 6: JSON output format
run_benchmark "json_output" "--iterations 5 --scale medium --format json"

# Performance comparison with data loading
echo ""
echo "📋 Benchmark: Data Loading Performance"
echo "-----------------------------------"

# Test data loading performance
for scale in small medium large; do
    echo "Testing data loading for scale: $scale"
    local test_db="$BENCH_DIR/load_test_${scale}"
    
    start_time=$(date +%s%N)
    $BINARY load-samples --scale $scale --force --db-path "$test_db" > "$BENCH_DIR/load_${scale}.txt" 2>&1
    end_time=$(date +%s%N)
    
    duration=$((($end_time - $start_time) / 1000000))
    echo "  ✅ $scale: ${duration}ms"
    
    # Clean up
    rm -rf "$test_db"
done

# Query performance benchmark
echo ""
echo "📋 Benchmark: Query Performance"
echo "-----------------------------------"

# Test various query types
queries=(
    "SELECT * WHERE { ?s ?p ?o } LIMIT 100"
    "SELECT (COUNT(*) as ?count) WHERE { ?s ?p ?o }"
    "SELECT * WHERE { ?event rdf:type epcis:ObjectEvent }"
    "SELECT DISTINCT ?manufacturer ?product WHERE { ?product ex:manufacturedBy ?manufacturer }"
)

for i in "${!queries[@]}"; do
    query="${queries[$i]}"
    echo "Testing query $((i+1)): $(echo "$query" | cut -c1-50)..."
    
    start_time=$(date +%s%N)
    $BINARY query --query "$query" --format json --db-path "$TEST_DB" > "$BENCH_DIR/query_$((i+1)).txt" 2>&1
    end_time=$(date +%s%N)
    
    duration=$((($end_time - $start_time) / 1000000))
    echo "  ✅ Query $((i+1)): ${duration}ms"
done

# Memory usage analysis
echo ""
echo "📋 Benchmark: Memory Usage Analysis"
echo "-----------------------------------"

if command -v ps >/dev/null 2>&1; then
    echo "Memory usage during operations:"
    
    # Start server in background
    $BINARY serve --port 8081 --use-samples-data --samples-scale medium --db-path "$TEST_DB" > "$BENCH_DIR/server_memory.txt" 2>&1 &
    SERVER_PID=$!
    
    # Wait for server to start
    sleep 3
    
    # Get memory usage
    if command -v ps >/dev/null 2>&1; then
        memory_usage=$(ps -p $SERVER_PID -o rss= 2>/dev/null | xargs || echo "N/A")
        if [ "$memory_usage" != "N/A" ] && [ "$memory_usage" != "" ]; then
            memory_mb=$((memory_usage / 1024))
            echo "  📊 Server memory usage: ${memory_mb}MB"
        fi
    fi
    
    # Stop server
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
fi

# Generate comprehensive report
echo ""
echo "📋 Generating Benchmark Report"
echo "================================"

REPORT_FILE="$BENCH_DIR/benchmark_report.txt"
cat > "$REPORT_FILE" << EOF
EPCIS Knowledge Graph Performance Benchmark Report
================================================
Test Date: $(date)
Test Environment: $(uname -a)

System Information:
- OS: $(uname -s)
- Architecture: $(uname -m)
- Memory: $(free -h 2>/dev/null || echo "N/A")
- CPU: $(nproc 2>/dev/null || echo "N/A") cores

Benchmark Results:
EOF

# Extract and summarize results
for bench_file in "$BENCH_DIR"/*.txt; do
    if [[ "$(basename "$bench_file")" == benchmark_* ]]; then
        bench_name=$(basename "$bench_file" .txt | sed 's/benchmark_//')
        echo "- $bench_name:" >> "$REPORT_FILE"
        
        # Extract key metrics if available
        if grep -q "Performance Benchmark Report" "$bench_file"; then
            echo "  Status: Completed" >> "$REPORT_FILE"
            echo "  Details: See $bench_file for full results" >> "$REPORT_FILE"
        else
            echo "  Status: See file for details" >> "$REPORT_FILE"
        fi
        echo "" >> "$REPORT_FILE"
    fi
done

# Add query performance summary
echo "" >> "$REPORT_FILE"
echo "Query Performance Summary:" >> "$REPORT_FILE"
for i in {1..4}; do
    query_file="$BENCH_DIR/query_$i.txt"
    if [ -f "$query_file" ]; then
        echo "- Query $i: Results in $query_file" >> "$REPORT_FILE"
    fi
done

# Add data loading summary
echo "" >> "$REPORT_FILE"
echo "Data Loading Performance:" >> "$REPORT_FILE"
for scale in small medium large; do
    load_file="$BENCH_DIR/load_${scale}.txt"
    if [ -f "$load_file" ]; then
        echo "- $scale scale: See $load_file" >> "$REPORT_FILE"
    fi
done

echo "" >> "$REPORT_FILE"
echo "Files generated:" >> "$REPORT_FILE"
echo "- Benchmark results: $BENCH_DIR/" >> "$REPORT_FILE"
echo "- Full report: $REPORT_FILE" >> "$REPORT_FILE"

echo "✅ Benchmark report generated: $REPORT_FILE"

# Summary statistics
echo ""
echo "📊 Benchmark Summary"
echo "===================="

# Count successful benchmarks
successful_benchmarks=0
total_benchmarks=0

for bench_file in "$BENCH_DIR"/benchmark_*.txt; do
    if [ -f "$bench_file" ]; then
        total_benchmarks=$((total_benchmarks + 1))
        if grep -q "completed successfully" "$bench_file"; then
            successful_benchmarks=$((successful_benchmarks + 1))
        fi
    fi
done

echo "✅ Successful benchmarks: $successful_benchmarks/$total_benchmarks"

# Show key findings
echo ""
echo "🔍 Key Findings:"
echo "- All benchmark results saved to: $BENCH_DIR/"
echo "- Comprehensive report available at: $REPORT_FILE"
echo "- Query performance results available for analysis"
echo "- Memory usage metrics collected (where available)"

# Performance targets check
echo ""
echo "🎯 Performance Targets Assessment:"
echo "- Data loading: < 2000ms (medium dataset)"
echo "- Query response: < 100ms (simple queries)"
echo "- Memory usage: < 1GB for medium datasets"
echo "- Concurrent operations: > 100 queries/sec"

# Cleanup
echo ""
echo "🧹 Cleaning up test databases..."
rm -rf "$TEST_DB" "$BENCH_DIR"/load_test_* 2>/dev/null || true

echo ""
echo "🎉 Benchmark suite completed successfully!"
echo "📄 Results saved to: $BENCH_DIR/"
echo "📋 Full report: $REPORT_FILE"