// Focused test to identify main frontend issues
console.log('=== STARTING MAIN FRONTEND DEBUG TEST ===');

const API_BASE = '/api/v1';

// Test 1: Dashboard Metrics
async function testDashboardMetrics() {
    console.log('\n--- Testing Dashboard Metrics ---');
    try {
        const response = await fetch(`${API_BASE}/statistics`);
        console.log('Statistics API status:', response.status);
        
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        const data = await response.json();
        console.log('Statistics data:', data);
        
        // Simulate what updateMetricsDisplay should do
        const metrics = {
            'total-triples': data.total_triples || 0,
            'query-rate': 0,
            'memory-usage': `${data.memory_usage_mb || 0} MB`,
            'active-connections': data.active_connections || 1
        };
        
        console.log('Metrics that would be displayed:', metrics);
        return { success: true, data, metrics };
        
    } catch (error) {
        console.error('Dashboard metrics test failed:', error);
        return { success: false, error: error.message };
    }
}

// Test 2: Ontology Loading
async function testOntologyLoading() {
    console.log('\n--- Testing Ontology Loading ---');
    try {
        const response = await fetch(`${API_BASE}/ontologies`);
        console.log('Ontologies API status:', response.status);
        
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        const data = await response.json();
        console.log('Ontologies data:', data);
        
        // Check if ontologies array exists and has items
        const hasOntologies = data.ontologies && Array.isArray(data.ontologies) && data.ontologies.length > 0;
        console.log('Has ontologies:', hasOntologies);
        console.log('Ontology count:', data.ontologies ? data.ontologies.length : 0);
        
        if (hasOntologies) {
            console.log('First ontology:', data.ontologies[0]);
        }
        
        return { success: true, data, hasOntologies };
        
    } catch (error) {
        console.error('Ontology loading test failed:', error);
        return { success: false, error: error.message };
    }
}

// Test 3: SPARQL Query Execution
async function testSparqlQuery() {
    console.log('\n--- Testing SPARQL Query Execution ---');
    try {
        const query = 'SELECT * WHERE { ?s ?p ?o } LIMIT 5';
        console.log('Testing query:', query);
        
        const response = await fetch(`${API_BASE}/sparql/query`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                query: query,
                default_graph_uri: null,
                named_graph_uri: null
            })
        });
        
        console.log('SPARQL API status:', response.status);
        
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        const data = await response.json();
        console.log('SPARQL response data:', data);
        
        const resultCount = data.results && data.results.bindings ? data.results.bindings.length : 0;
        console.log('Result count:', resultCount);
        
        if (resultCount > 0) {
            console.log('First result:', data.results.bindings[0]);
        }
        
        return { success: true, data, resultCount };
        
    } catch (error) {
        console.error('SPARQL query test failed:', error);
        return { success: false, error: error.message };
    }
}

// Test 4: Monitoring Data
async function testMonitoringData() {
    console.log('\n--- Testing Monitoring Data ---');
    try {
        const response = await fetch(`${API_BASE}/monitoring/health`);
        console.log('Monitoring API status:', response.status);
        
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        const data = await response.json();
        console.log('Monitoring data:', data);
        
        const hasHealthData = data.status && data.cpu_usage_percent !== undefined && data.memory_usage_mb !== undefined;
        console.log('Has health data:', hasHealthData);
        
        return { success: true, data, hasHealthData };
        
    } catch (error) {
        console.error('Monitoring data test failed:', error);
        return { success: false, error: error.message };
    }
}

// Test 5: DOM Element Simulation (what should happen in the real frontend)
function testDomElementSimulation() {
    console.log('\n--- Testing DOM Element Simulation ---');
    
    // Simulate what should be in the HTML
    const simulatedDom = {
        'total-triples': { element: 'exists', value: null },
        'query-rate': { element: 'exists', value: null },
        'memory-usage': { element: 'exists', value: null },
        'active-connections': { element: 'exists', value: null },
        'query-results': { element: 'exists', value: null },
        'ontology-list': { element: 'exists', value: null },
        'system-metrics': { element: 'exists', value: null }
    };
    
    console.log('Simulated DOM elements:', simulatedDom);
    return { success: true, domElements: simulatedDom };
}

// Run all tests
async function runAllTests() {
    console.log('Starting comprehensive main frontend debug test...');
    
    const results = {
        dashboard: await testDashboardMetrics(),
        ontology: await testOntologyLoading(),
        sparql: await testSparqlQuery(),
        monitoring: await testMonitoringData(),
        dom: testDomElementSimulation()
    };
    
    console.log('\n=== TEST RESULTS SUMMARY ===');
    console.log('Dashboard Metrics:', results.dashboard.success ? '✅ PASS' : '❌ FAIL');
    console.log('Ontology Loading:', results.ontology.success ? '✅ PASS' : '❌ FAIL');
    console.log('SPARQL Query:', results.sparql.success ? '✅ PASS' : '❌ FAIL');
    console.log('Monitoring Data:', results.monitoring.success ? '✅ PASS' : '❌ FAIL');
    console.log('DOM Elements:', results.dom.success ? '✅ PASS' : '❌ FAIL');
    
    // Identify issues
    console.log('\n=== ISSUE ANALYSIS ===');
    
    if (!results.dashboard.success) {
        console.log('❌ ISSUE: Dashboard metrics not loading -', results.dashboard.error);
    }
    
    if (!results.ontology.success) {
        console.log('❌ ISSUE: Ontology data not loading -', results.ontology.error);
    } else if (!results.ontology.hasOntologies) {
        console.log('❌ ISSUE: Ontology API responded but no ontologies found');
    }
    
    if (!results.sparql.success) {
        console.log('❌ ISSUE: SPARQL query not working -', results.sparql.error);
    } else if (results.sparql.resultCount === 0) {
        console.log('❌ ISSUE: SPARQL query succeeded but no results returned');
    }
    
    if (!results.monitoring.success) {
        console.log('❌ ISSUE: Monitoring data not loading -', results.monitoring.error);
    } else if (!results.monitoring.hasHealthData) {
        console.log('❌ ISSUE: Monitoring API responded but missing health data');
    }
    
    // If all APIs work, the issue is likely in frontend JavaScript execution
    const allApisWork = results.dashboard.success && results.ontology.success && 
                       results.sparql.success && results.monitoring.success;
    
    if (allApisWork) {
        console.log('\n🔍 LIKELY ISSUE: All APIs work correctly. The problem is in frontend JavaScript execution or DOM manipulation.');
        console.log('Possible causes:');
        console.log('1. JavaScript not loading or executing properly');
        console.log('2. DOM elements not found by JavaScript');
        console.log('3. Event handlers not attached correctly');
        console.log('4. CSS/display issues hiding content');
        console.log('5. Browser console errors preventing execution');
    }
    
    return results;
}

// Auto-run the test
runAllTests().then(results => {
    console.log('\n=== DEBUG TEST COMPLETE ===');
}).catch(error => {
    console.error('Debug test failed:', error);
});