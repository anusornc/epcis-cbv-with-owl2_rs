// EPCIS Knowledge Graph - Main JavaScript

// Global state
const AppState = {
    currentPage: 'dashboard',
    apiBaseUrl: '/api/v1',
    metrics: {
        totalTriples: 0,
        queryRate: 0,
        memoryUsage: 0,
        activeConnections: 0
    },
    alerts: [],
    ontologies: [],
    refreshInterval: null,
    isPolling: false,
    graph: {
        nodes: [],
        edges: [],
        svg: null,
        simulation: null,
        currentType: 'force'
    }
};

// Initialize the application
document.addEventListener('DOMContentLoaded', function() {
    initializeApp();
});

function initializeApp() {
    setupNavigation();
    loadDashboardData();
    startPolling();
    setupEventListeners();
}

// Navigation setup
function setupNavigation() {
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.addEventListener('click', function(e) {
            e.preventDefault();
            const page = this.getAttribute('data-page');
            navigateToPage(page);
        });
    });
}

// Page navigation
function navigateToPage(page) {
    // Hide all pages
    const pages = document.querySelectorAll('.page');
    pages.forEach(p => p.classList.remove('active'));
    
    // Show selected page
    const targetPage = document.getElementById(`${page}-page`);
    if (targetPage) {
        targetPage.classList.add('active');
        AppState.currentPage = page;
        
        // Update navigation
        const navLinks = document.querySelectorAll('.nav-link');
        navLinks.forEach(link => {
            link.classList.remove('active');
            if (link.getAttribute('data-page') === page) {
                link.classList.add('active');
            }
        });
        
        // Load page-specific data
        loadPageData(page);
    }
}

// Load page-specific data
function loadPageData(page) {
    switch (page) {
        case 'dashboard':
            loadDashboardData();
            break;
        case 'query':
            loadQueryInterface();
            break;
        case 'ontology':
            loadOntologyData();
            break;
        case 'events':
            loadEventsData();
            break;
        case 'monitoring':
            loadMonitoringData();
            break;
        case 'visualization':
            loadVisualizationData();
            break;
    }
}

// Dashboard functionality
async function loadDashboardData() {
    try {
        await Promise.all([
            loadMetrics(),
            loadAlerts()
        ]);
    } catch (error) {
        console.error('Error loading dashboard data:', error);
        showError('Failed to load dashboard data');
    }
}

async function loadMetrics() {
    try {
        console.log('Loading metrics from:', `${AppState.apiBaseUrl}/statistics`);
        const response = await fetch(`${AppState.apiBaseUrl}/statistics`);
        const data = await response.json();
        console.log('Metrics response:', data);
        
        if (data.status === 'operational') {
            // Get real data from statistics API
            updateMetricsDisplay({
                total_triples: data.total_triples,
                query_rate: 0, // Not available in current API
                memory_usage_mb: 0, // Not available in current API
                active_connections: 1 // Mock value for now
            });
        } else {
            console.error('Metrics API returned non-operational status:', data);
        }
    } catch (error) {
        console.error('Error loading metrics:', error);
        // Use mock data for demo
        updateMetricsDisplay({
            total_triples: 0,
            query_rate: 0,
            memory_usage_mb: 0,
            active_connections: 0
        });
    }
}

function updateMetricsDisplay(metrics) {
    const elements = {
        'total-triples': metrics.total_triples || metrics.totalTriples || 0,
        'query-rate': metrics.query_rate || metrics.queryRate || 0,
        'memory-usage': `${metrics.memory_usage_mb || metrics.memoryUsage || 0} MB`,
        'active-connections': metrics.active_connections || metrics.activeConnections || 0
    };
    
    Object.entries(elements).forEach(([id, value]) => {
        const element = document.getElementById(id);
        if (element) {
            element.textContent = value;
        }
    });
}

async function loadAlerts() {
    try {
        const response = await fetch(`${AppState.apiBaseUrl}/monitoring/alerts`);
        const data = await response.json();
        
        if (data.success) {
            updateAlertsDisplay(data.alerts || []);
        }
    } catch (error) {
        console.error('Error loading alerts:', error);
        updateAlertsDisplay([]);
    }
}

function updateAlertsDisplay(alerts) {
    const alertsList = document.getElementById('alerts-list');
    if (!alertsList) return;
    
    if (alerts.length === 0) {
        alertsList.innerHTML = '<p>No active alerts</p>';
        return;
    }
    
    alertsList.innerHTML = alerts.map(alert => `
        <div class="alert-item severity-${alert.severity || 'info'}">
            <div class="alert-message">${alert.message || 'System alert'}</div>
            <div class="alert-time">${formatTime(alert.timestamp)}</div>
        </div>
    `).join('');
}

// Query interface functionality
function loadQueryInterface() {
    const executeButton = document.getElementById('execute-query');
    const clearButton = document.getElementById('clear-query');
    const queryEditor = document.getElementById('sparql-query');
    
    if (executeButton) {
        executeButton.addEventListener('click', executeQuery);
    }
    
    if (clearButton) {
        clearButton.addEventListener('click', clearQuery);
    }
    
    if (queryEditor) {
        // Set default query
        queryEditor.value = `SELECT * WHERE { ?s ?p ?o } LIMIT 10`;
    }
}

async function executeQuery() {
    const queryEditor = document.getElementById('sparql-query');
    const resultsContainer = document.getElementById('query-results');
    
    if (!queryEditor || !resultsContainer) return;
    
    const query = queryEditor.value.trim();
    if (!query) {
        showError('Please enter a SPARQL query');
        return;
    }
    
    try {
        console.log('Executing SPARQL query:', query);
        console.log('API URL:', `${AppState.apiBaseUrl}/sparql/query`);
        
        resultsContainer.innerHTML = '<div class="loading-spinner"><div class="spinner"></div><p>Executing query...</p></div>';
        
        const response = await fetch(`${AppState.apiBaseUrl}/sparql/query`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                query: query,
                default_graph_uri: null,
                named_graph_uri: null
            })
        });
        
        console.log('SPARQL response status:', response.status);
        const data = await response.json();
        console.log('SPARQL response data:', data);
        
        displayQueryResults(data);
    } catch (error) {
        console.error('Error executing query:', error);
        showError('Failed to execute query');
        resultsContainer.innerHTML = '<div class="error-display">Failed to execute query. Please try again.</div>';
    }
}

function displayQueryResults(data) {
    const resultsContainer = document.getElementById('query-results');
    if (!resultsContainer) return;
    
    console.log('SPARQL query results:', data);
    
    if (data.status === 'success' && data.results) {
        // Use the real results from the API
        resultsContainer.innerHTML = formatSparqlResults(data);
    } else {
        console.error('SPARQL query failed:', data);
        resultsContainer.innerHTML = `<div class="error-display">Query failed: ${data.message || 'Unknown error'}</div>`;
    }
}

function formatSparqlResults(results) {
    if (!results.results || !results.results.bindings || results.results.bindings.length === 0) {
        return '<p>No results found</p>';
    }
    
    const vars = results.head.vars;
    const bindings = results.results.bindings;
    
    let html = '<table class="results-table"><thead><tr>';
    vars.forEach(v => {
        html += `<th>${v}</th>`;
    });
    html += '</tr></thead><tbody>';
    
    bindings.forEach(binding => {
        html += '<tr>';
        vars.forEach(v => {
            const value = binding[v];
            if (value) {
                let displayValue = value.value;
                if (value.type === 'uri') {
                    displayValue = `<a href="${value.value}" target="_blank">${value.value}</a>`;
                }
                html += `<td>${displayValue}</td>`;
            } else {
                html += '<td></td>';
            }
        });
        html += '</tr>';
    });
    
    html += '</tbody></table>';
    return html;
}

function clearQuery() {
    const queryEditor = document.getElementById('sparql-query');
    const resultsContainer = document.getElementById('query-results');
    
    if (queryEditor) {
        queryEditor.value = '';
    }
    
    if (resultsContainer) {
        resultsContainer.innerHTML = '<p>Query results will appear here...</p>';
    }
}

// Ontology management
async function loadOntologyData() {
    try {
        console.log('Loading ontology data from:', `${AppState.apiBaseUrl}/ontologies`);
        const response = await fetch(`${AppState.apiBaseUrl}/ontologies`);
        const data = await response.json();
        console.log('Ontology response:', data);
        
        if (data.ontologies) {
            updateOntologyDisplay(data.ontologies);
        } else {
            console.error('No ontologies found in response:', data);
            updateOntologyDisplay([]);
        }
    } catch (error) {
        console.error('Error loading ontology data:', error);
        // Mock data for demo
        updateOntologyDisplay([]);
    }
}

function updateOntologyDisplay(ontologies) {
    const ontologyList = document.getElementById('ontology-list');
    if (!ontologyList) return;
    
    if (ontologies.length === 0) {
        ontologyList.innerHTML = '<p>No ontologies loaded</p>';
        return;
    }
    
    ontologyList.innerHTML = ontologies.map(ont => `
        <div class="ontology-item">
            <div class="ontology-name">${ont.name || ont}</div>
            <div class="ontology-info">
                <span class="ontology-triples">${ont.triples || 0} triples</span>
                <span class="ontology-status ${ont.loaded ? 'loaded' : 'not-loaded'}">
                    ${ont.loaded ? '✓ Loaded' : 'Not loaded'}
                </span>
            </div>
        </div>
    `).join('');
}

// Events management
async function loadEventsData() {
    try {
        const response = await fetch(`${AppState.apiBaseUrl}/events`);
        const data = await response.json();
        
        if (data.events) {
            updateEventsDisplay(data.events);
        }
    } catch (error) {
        console.error('Error loading events data:', error);
        // Mock data for demo
        updateEventsDisplay([
            { id: 'event1', type: 'ObjectEvent', timestamp: new Date().toISOString(), status: 'processed' },
            { id: 'event2', type: 'AggregationEvent', timestamp: new Date(Date.now() - 300000).toISOString(), status: 'processed' }
        ]);
    }
}

function updateEventsDisplay(events) {
    const eventsContainer = document.querySelector('.event-editor');
    if (!eventsContainer) return;
    
    if (events.length === 0) {
        eventsContainer.innerHTML = '<p>No events found</p>';
        return;
    }
    
    eventsContainer.innerHTML = `
        <div class="events-list">
            <h4>Recent Events</h4>
            ${events.map(event => `
                <div class="event-item">
                    <div class="event-id">${event.id}</div>
                    <div class="event-type">${event.type}</div>
                    <div class="event-time">${formatTime(event.timestamp)}</div>
                    <div class="event-status">${event.status}</div>
                </div>
            `).join('')}
        </div>
    `;
}

// Monitoring dashboard
async function loadMonitoringData() {
    try {
        console.log('Loading monitoring data from:', `${AppState.apiBaseUrl}/monitoring/health`);
        const response = await fetch(`${AppState.apiBaseUrl}/monitoring/health`);
        const data = await response.json();
        console.log('Monitoring response:', data);
        
        if (data.success) {
            updateMonitoringDisplay(data);
        } else {
            console.error('Monitoring API returned non-success status:', data);
            // Use fallback data
            updateMonitoringDisplay({
                status: data.status || 'unknown',
                uptime_seconds: data.uptime_seconds || 0,
                total_requests: data.total_requests || 0,
                successful_requests: data.successful_requests || 0,
                failed_requests: data.failed_requests || 0,
                memory_usage_mb: data.memory_usage_mb || 0,
                cpu_usage_percent: data.cpu_usage_percent || 0
            });
        }
    } catch (error) {
        console.error('Error loading monitoring data:', error);
        // Mock data for demo
        updateMonitoringDisplay({
            status: 'healthy',
            uptime_seconds: 3600,
            total_requests: 150,
            successful_requests: 145,
            failed_requests: 5,
            memory_usage_mb: 256,
            cpu_usage_percent: 25
        });
    }
}

function updateMonitoringDisplay(data) {
    const systemMetrics = document.getElementById('system-metrics');
    if (!systemMetrics) return;
    
    systemMetrics.innerHTML = `
        <div class="metrics-grid">
            <div class="metric-card">
                <div class="metric-label">Status</div>
                <div class="metric-value status-${data.status}">${data.status}</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Uptime</div>
                <div class="metric-value">${formatUptime(data.uptime_seconds)}</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Requests</div>
                <div class="metric-value">${data.total_requests || 0}</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Success Rate</div>
                <div class="metric-value">${calculateSuccessRate(data.successful_requests, data.total_requests)}%</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Memory</div>
                <div class="metric-value">${data.memory_usage_mb || 0} MB</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">CPU</div>
                <div class="metric-value">${data.cpu_usage_percent || 0}%</div>
            </div>
        </div>
    `;
}

// Event listeners setup
function setupEventListeners() {
    // Handle quick action buttons
    const actionButtons = document.querySelectorAll('.action-button');
    actionButtons.forEach(button => {
        button.addEventListener('click', function(e) {
            e.preventDefault();
            const page = this.getAttribute('data-page');
            if (page) {
                navigateToPage(page);
            }
        });
    });
}

// Polling for real-time updates
function startPolling() {
    if (AppState.isPolling) return;
    
    AppState.isPolling = true;
    AppState.refreshInterval = setInterval(() => {
        if (AppState.currentPage === 'dashboard') {
            loadDashboardData();
        } else if (AppState.currentPage === 'monitoring') {
            loadMonitoringData();
        }
    }, 30000); // Refresh every 30 seconds
}

function stopPolling() {
    if (AppState.refreshInterval) {
        clearInterval(AppState.refreshInterval);
        AppState.refreshInterval = null;
    }
    AppState.isPolling = false;
}

// Utility functions
function formatTime(timestamp) {
    if (!timestamp) return 'N/A';
    const date = new Date(timestamp);
    return date.toLocaleString();
}

function formatUptime(seconds) {
    if (!seconds) return 'N/A';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    
    if (hours > 0) {
        return `${hours}h ${minutes}m ${secs}s`;
    } else if (minutes > 0) {
        return `${minutes}m ${secs}s`;
    } else {
        return `${secs}s`;
    }
}

function calculateSuccessRate(successful, total) {
    if (!total || total === 0) return 0;
    return Math.round((successful / total) * 100);
}

function showError(message) {
    // Create and show error notification
    const errorDiv = document.createElement('div');
    errorDiv.className = 'error-notification';
    errorDiv.textContent = message;
    document.body.appendChild(errorDiv);
    
    setTimeout(() => {
        document.body.removeChild(errorDiv);
    }, 5000);
}

// Cleanup on page unload
window.addEventListener('beforeunload', function() {
    stopPolling();
});

// Knowledge Graph Visualization
async function loadVisualizationData() {
    console.log('Loading visualization data...');
    setupVisualizationControls();
    
    // Auto-load graph when page is opened
    setTimeout(() => {
        loadKnowledgeGraph();
    }, 500);
}

function setupVisualizationControls() {
    const loadButton = document.getElementById('load-graph');
    const resetButton = document.getElementById('reset-graph');
    const graphType = document.getElementById('graph-type');
    const nodeLimit = document.getElementById('node-limit');
    
    if (loadButton) {
        loadButton.addEventListener('click', loadKnowledgeGraph);
    }
    
    if (resetButton) {
        resetButton.addEventListener('click', resetGraphView);
    }
    
    if (graphType) {
        graphType.addEventListener('change', function() {
            AppState.graph.currentType = this.value;
            document.getElementById('current-graph-type').textContent = 
                this.options[this.selectedIndex].text;
        });
    }
}

async function loadKnowledgeGraph() {
    const container = document.getElementById('graph-container');
    const nodeLimit = parseInt(document.getElementById('node-limit').value) || 50;
    
    if (!container) return;
    
    try {
        container.innerHTML = '<div class="loading-spinner"><div class="spinner"></div><p>Loading knowledge graph...</p></div>';
        
        // Load graph data from SPARQL query
        const sparqlQuery = `SELECT * WHERE { ?s ?p ?o } LIMIT ${nodeLimit}`;
        const response = await fetch(`${AppState.apiBaseUrl}/sparql/query`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                query: sparqlQuery,
                default_graph_uri: null,
                named_graph_uri: null
            })
        });
        
        const data = await response.json();
        console.log('Graph data response:', data);
        
        if (data.status === 'success' && data.results && data.results.bindings) {
            const graphData = processSparqlToGraph(data.results.bindings);
            renderKnowledgeGraph(graphData, container);
            updateGraphInfo(graphData);
        } else {
            container.innerHTML = '<div class="error-display">Failed to load graph data</div>';
        }
    } catch (error) {
        console.error('Error loading knowledge graph:', error);
        container.innerHTML = '<div class="error-display">Error loading graph data</div>';
    }
}

function processSparqlToGraph(bindings) {
    const nodes = new Map();
    const edges = [];
    let nodeIdCounter = 0;
    
    bindings.forEach(binding => {
        const subject = binding.s.value;
        const predicate = binding.p.value;
        const object = binding.o.value;
        
        // Add subject node if not exists
        if (!nodes.has(subject)) {
            nodes.set(subject, {
                id: nodeIdCounter++,
                uri: subject,
                label: getNodeLabel(subject),
                type: binding.s.type,
                group: getNodeGroup(subject)
            });
        }
        
        // Add object node if not exists
        if (!nodes.has(object)) {
            nodes.set(object, {
                id: nodeIdCounter++,
                uri: object,
                label: getNodeLabel(object),
                type: binding.o.type,
                group: getNodeGroup(object)
            });
        }
        
        // Add edge
        edges.push({
            source: nodes.get(subject).id,
            target: nodes.get(object).id,
            predicate: predicate,
            label: getEdgeLabel(predicate)
        });
    });
    
    return {
        nodes: Array.from(nodes.values()),
        edges: edges
    };
}

function getNodeLabel(uri) {
    if (uri.startsWith('http://')) {
        const parts = uri.split('/');
        return parts[parts.length - 1] || uri;
    }
    return uri;
}

function getNodeGroup(uri) {
    if (uri.includes('product')) return 1;
    if (uri.includes('location')) return 2;
    if (uri.includes('manufacturer') || uri.includes('business')) return 3;
    if (uri.includes('event')) return 4;
    if (uri.includes('epcis')) return 5;
    return 0;
}

function getEdgeLabel(predicate) {
    if (predicate.startsWith('http://')) {
        const parts = predicate.split('/');
        return parts[parts.length - 1] || predicate;
    }
    return predicate;
}

function renderKnowledgeGraph(graphData, container) {
    // Clear previous graph
    container.innerHTML = '';
    
    const width = container.clientWidth;
    const height = container.clientHeight;
    
    // Create SVG
    const svg = d3.select(container)
        .append('svg')
        .attr('width', width)
        .attr('height', height);
    
    // Create force simulation
    const simulation = d3.forceSimulation(graphData.nodes)
        .force('link', d3.forceLink(graphData.edges).id(d => d.id).distance(80))
        .force('charge', d3.forceManyBody().strength(-300))
        .force('center', d3.forceCenter(width / 2, height / 2))
        .force('collision', d3.forceCollide().radius(25));
    
    // Create links
    const link = svg.append('g')
        .selectAll('line')
        .data(graphData.edges)
        .enter().append('line')
        .attr('class', 'graph-link')
        .attr('stroke-width', 2);
    
    // Create nodes
    const node = svg.append('g')
        .selectAll('circle')
        .data(graphData.nodes)
        .enter().append('circle')
        .attr('class', 'graph-node')
        .attr('r', 8)
        .attr('fill', d => getNodeColor(d.group))
        .call(drag(simulation));
    
    // Create labels
    const labels = svg.append('g')
        .selectAll('text')
        .data(graphData.nodes)
        .enter().append('text')
        .attr('class', 'node-label')
        .text(d => d.label)
        .attr('font-size', '10px')
        .attr('dx', 12)
        .attr('dy', 4);
    
    // Add tooltips
    node.append('title')
        .text(d => `${d.label}\\nType: ${d.type}\\nURI: ${d.uri}`);
    
    // Update positions on tick
    simulation.on('tick', () => {
        link
            .attr('x1', d => d.source.x)
            .attr('y1', d => d.source.y)
            .attr('x2', d => d.target.x)
            .attr('y2', d => d.target.y);
        
        node
            .attr('cx', d => d.x)
            .attr('cy', d => d.y);
        
        labels
            .attr('x', d => d.x)
            .attr('y', d => d.y);
    });
    
    // Store in AppState
    AppState.graph.svg = svg;
    AppState.graph.simulation = simulation;
    AppState.graph.nodes = graphData.nodes;
    AppState.graph.edges = graphData.edges;
}

function getNodeColor(group) {
    const colors = ['#6366f1', '#8b5cf6', '#ec4899', '#f59e0b', '#10b981', '#3b82f6'];
    return colors[group % colors.length];
}

function drag(simulation) {
    function dragstarted(event) {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        event.subject.fx = event.subject.x;
        event.subject.fy = event.subject.y;
    }
    
    function dragged(event) {
        event.subject.fx = event.x;
        event.subject.fy = event.y;
    }
    
    function dragended(event) {
        if (!event.active) simulation.alphaTarget(0);
        event.subject.fx = null;
        event.subject.fy = null;
    }
    
    return d3.drag()
        .on('start', dragstarted)
        .on('drag', dragged)
        .on('end', dragended);
}

function resetGraphView() {
    if (AppState.graph.simulation) {
        AppState.graph.simulation.alpha(1).restart();
    }
}

function updateGraphInfo(graphData) {
    document.getElementById('total-nodes').textContent = graphData.nodes.length;
    document.getElementById('total-edges').textContent = graphData.edges.length;
    document.getElementById('graph-updated').textContent = new Date().toLocaleString();
}