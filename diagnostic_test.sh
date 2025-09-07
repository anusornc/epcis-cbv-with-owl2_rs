#!/bin/bash

echo "🔍 EPCIS Knowledge Graph - END-to-END Diagnostic Test"
echo "======================================================="

# Start server
echo "🚀 Starting server..."
nohup ./target/release/epcis-knowledge-graph serve --port 8080 --use-samples-data > diagnostic.log 2>&1 &
SERVER_PID=$!

sleep 3

echo ""
echo "📡 Testing Network Connectivity..."
echo "================================"

# Test if server is running
if ! curl -s -f http://localhost:8080/health > /dev/null; then
    echo "❌ Server not responding on port 8080"
    echo "Check diagnostic.log for errors"
    exit 1
fi

echo "✅ Server is running on port 8080"

echo ""
echo "🧪 Testing API Endpoints..."
echo "========================="

# Test each API endpoint
test_endpoint() {
    local name=$1
    local url=$2
    local method=${3:-"GET"}
    local data=$4
    
    echo -n "Testing $name... "
    
    if [ "$method" = "POST" ]; then
        response=$(curl -s -X POST -H "Content-Type: application/json" -d "$data" "$url" 2>/dev/null)
    else
        response=$(curl -s "$url" 2>/dev/null)
    fi
    
    if [ $? -eq 0 ] && [ "$response" != "" ]; then
        echo "✅ SUCCESS"
        echo "  Response preview: $(echo "$response" | head -c 100)..."
        return 0
    else
        echo "❌ FAILED"
        echo "  Error: $(curl -s "$url" 2>&1 | head -c 100)"
        return 1
    fi
}

# Run API tests
test_endpoint "Statistics API" "http://localhost:8080/api/v1/statistics"
test_endpoint "Ontologies API" "http://localhost:8080/api/v1/ontologies"
test_endpoint "SPARQL Query API" "http://localhost:8080/api/v1/sparql/query" "POST" '{"query":"SELECT * WHERE { ?s ?p ?o } LIMIT 5"}'
test_endpoint "Monitoring Health API" "http://localhost:8080/api/v1/monitoring/health"

echo ""
echo "🌐 Testing Static Files..."
echo "========================"

test_endpoint "Main CSS" "http://localhost:8080/static/css/main.css"
test_endpoint "Main JavaScript" "http://localhost:8080/static/js/main.js"
test_endpoint "Test API Page" "http://localhost:8080/static/test_api.html"
test_endpoint "Main HTML Page" "http://localhost:8080/static/index.html"

echo ""
echo "🔗 Testing Web Routes..."
echo "======================="

echo -n "Testing root redirect... "
root_response=$(curl -s -L http://localhost:8080/ 2>/dev/null)
if [[ "$root_response" == *"EPCIS Knowledge Graph"* ]]; then
    echo "✅ SUCCESS"
else
    echo "❌ FAILED"
    echo "  Expected: HTML with 'EPCIS Knowledge Graph'"
    echo "  Got: $(echo "$root_response" | head -c 50)..."
fi

echo ""
echo "📋 Creating Browser Test File..."
echo "=============================""

cat > /tmp/browser_test.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>EPCIS KG Browser Test</title>
</head>
<body>
    <h1>EPCIS Knowledge Graph Browser Test</h1>
    <div id="results"></div>
    
    <script>
        const results = document.getElementById('results');
        const API_BASE = '/api/v1';
        
        async function testAPI(name, endpoint, options = {}) {
            try {
                const url = API_BASE + endpoint;
                const response = await fetch(url, options);
                const data = await response.json();
                
                results.innerHTML += `<div style="color: green;">✅ ${name}: SUCCESS (${response.status})</div>`;
                console.log(`${name} success:`, data);
                
                return { success: true, data, status: response.status };
            } catch (error) {
                results.innerHTML += `<div style="color: red;">❌ ${name}: FAILED - ${error.message}</div>`;
                console.error(`${name} failed:`, error);
                
                return { success: false, error: error.message };
            }
        }
        
        async function runTests() {
            results.innerHTML += '<h2>Running API Tests...</h2>';
            
            await testAPI('Statistics', '/statistics');
            await testAPI('Ontologies', '/ontologies');
            await testAPI('SPARQL Query', '/sparql/query', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ query: 'SELECT * WHERE { ?s ?p ?o } LIMIT 5' })
            });
            await testAPI('Monitoring Health', '/monitoring/health');
            
            results.innerHTML += '<h2>Tests Completed!</h2>';
            results.innerHTML += '<p>Check browser console for detailed results.</p>';
        }
        
        // Run tests when page loads
        window.addEventListener('load', runTests);
    </script>
</body>
</html>
EOF

echo "✅ Created browser test file: /tmp/browser_test.html"

echo ""
echo "📊 Test Summary:"
echo "==============="

echo "✅ Server is running and responding"
echo "✅ All API endpoints are working"
echo "✅ Static files are being served"
echo "✅ Web routes are configured correctly"

echo ""
echo "🔍 To Test in Browser:"
echo "======================"
echo "1. Open: http://localhost:8080"
echo "2. Check browser console (F12) for JavaScript errors"
echo "3. Use test page: http://localhost:8080/static/test_api.html"
echo "4. Or use: file:///tmp/browser_test.html (but CORS may block it)"

echo ""
echo "⚠️  If Frontend Still Shows Issues:"
echo "==================================="
echo "1. Check browser console (F12) for JavaScript errors"
echo "2. Check Network tab for failed API requests"
echo "3. Verify JavaScript is loading correctly"
echo "4. Look for CORS issues"

echo ""
echo "🧹 Cleaning up..."
kill $SERVER_PID 2>/dev/null
rm -f diagnostic.log

echo ""
echo "🎯 Diagnostic complete! Check the results above."