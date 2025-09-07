# EPCIS Knowledge Graph Web Frontend Development Plan

## 🎯 Executive Summary

This document outlines the comprehensive plan for developing a web frontend for the EPCIS Knowledge Graph project using the Leptos framework. The frontend will provide an intuitive interface for interacting with the semantic reasoning system, SPARQL querying, ontology management, and real-time monitoring capabilities.

## 🏗️ Technology Stack

### **Primary Framework: Leptos**
- **Type**: Full-stack Rust framework with isomorphic architecture
- **Benefits**: Type safety, performance, seamless Axum integration
- **Architecture**: Server-side rendering + client-side interactivity

### **Supporting Technologies**
- **WebAssembly**: For high-performance client-side execution
- **Axum Integration**: Seamless integration with existing backend
- **reqwest**: HTTP client for API communication
- **Charting Libraries**: For data visualization
- **Styling**: CSS-based with potential for Tailwind CSS

## 📋 Project Structure

```
src/
├── main.rs              # Enhanced with Leptos integration
├── lib.rs               # Existing backend logic
├── frontend/            # NEW: Frontend components
│   ├── mod.rs
│   ├── app.rs           # Main app component
│   ├── components/      # Reusable components
│   │   ├── mod.rs
│   │   ├── layout.rs
│   │   ├── navigation.rs
│   │   ├── sparql_query.rs
│   │   ├── ontology_browser.rs
│   │   ├── event_editor.rs
│   │   ├── monitoring.rs
│   │   └── visualization.rs
│   ├── pages/           # Page components
│   │   ├── mod.rs
│   │   ├── dashboard.rs
│   │   ├── query_interface.rs
│   │   ├── ontology_management.rs
│   │   ├── event_processing.rs
│   │   └── monitoring_dashboard.rs
│   ├── hooks/           # Custom hooks
│   │   ├── mod.rs
│   │   ├── use_api.rs
│   │   ├── use_sparql.rs
│   │   └── use_real_time.rs
│   ├── types/           # Frontend-specific types
│   │   ├── mod.rs
│   │   └── api_types.rs
│   └── utils/           # Frontend utilities
│       ├── mod.rs
│       ├── formatting.rs
│       └── validation.rs
├── api/                 # Existing API layer
├── storage/             # Existing storage layer
├── ontology/            # Existing ontology layer
└── ...                  # Other existing modules

static/                  # Static assets
├── css/
│   ├── main.css
│   └── components.css
├── js/
│   └── utils.js
└── images/
```

## 🚀 Implementation Phases

### **Phase 1: Setup and Basic Interface (Week 1-2)**

#### **1.1 Environment Setup**
- Add Leptos dependencies to Cargo.toml
- Set up trunk for WASM building
- Configure development environment
- Create basic project structure

#### **1.2 Basic Integration**
- Integrate Leptos with existing Axum server
- Set up routing system
- Create basic layout and navigation
- Implement static file serving for assets

#### **1.3 Core Components**
- Dashboard page with system overview
- Basic SPARQL query interface
- Simple API integration hooks
- Navigation and layout components

### **Phase 2: Advanced Features (Week 3-4)**

#### **2.1 Enhanced SPARQL Interface**
- Query builder with syntax highlighting
- Results visualization (tables, charts)
- Query history and saved queries
- Export functionality

#### **2.2 Ontology Management**
- Visual ontology browser
- Class and property inspection
- Ontology upload and management
- Hierarchy visualization

#### **2.3 Event Processing**
- EPCIS event editor with validation
- Batch event processing
- Event timeline visualization
- Real-time event updates

### **Phase 3: Visualization and Monitoring (Week 5-6)**

#### **3.1 Data Visualization**
- Interactive graph visualization
- Timeline and sequence diagrams
- Statistical charts and graphs
- Export and sharing capabilities

#### **3.2 Real-time Features**
- WebSocket integration for live updates
- Real-time metrics dashboard
- Alert management system
- Performance monitoring

### **Phase 4: Production Features (Week 7-8)**

#### **4.1 User Experience**
- Responsive design for mobile
- Accessibility improvements
- Internationalization support
- Theme customization

#### **4.2 Performance Optimization**
- Caching strategies
- Lazy loading components
- Optimized WASM bundle size
- Server-side rendering optimization

## 🔧 Technical Implementation

### **Dependencies**

```toml
[dependencies]
# Leptos framework
leptos = { version = "0.2", features = ["csr", "ssr"] }
leptos_axum = "0.2"
leptos_meta = "0.2"
leptos_router = "0.2"

# WASM dependencies
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
web-sys = "0.3"
js-sys = "0.3"

# HTTP client for API calls
reqwest = { version = "0.11", features = ["json"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["serde"] }
thiserror = "1.0"
anyhow = "1.0"

# Visualization (optional)
resvg = "0.36"
plotters = "0.3"

# Development
tracing = "0.1"
```

### **Key Components**

#### **1. Main App Integration**
```rust
// src/main.rs - Enhanced with Leptos
use leptos::*;
use leptos_axum::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Route path="/" view=Dashboard/>
            <Route path="/query" view=SparqlInterface/>
            <Route path="/ontology" view=OntologyBrowser/>
            <Route path="/events" view=EventProcessor/>
            <Route path="/monitoring" view=MonitoringDashboard/>
        </Router>
    }
}

// Integration with existing Axum server
let app = Router::new()
    .route("/", get(leptos_routes_handler))
    .route("/api/v1/*rest", axum_routing::fallback(api_routes))
    .leptos_routes(leptos_options, routes, App)
    .layer(cors_layer)
    .layer(TraceLayer::new_for_http());
```

#### **2. SPARQL Query Interface**
```rust
#[component]
pub fn SparqlQueryInterface() -> impl IntoView {
    let (query, set_query) = create_signal("".to_string());
    let (results, set_results) = create_signal(Vec::new());
    let (loading, set_loading) = create_signal(false);
    
    view! {
        <div class="sparql-interface">
            <h2>"SPARQL Query Editor"</h2>
            <textarea 
                class="query-editor"
                value=query
                on:input=move |ev| set_query(event_target_value(&ev))
            />
            <button 
                on:click=move |_| execute_query(query.get(), set_results, set_loading)
                disabled=loading
            >
                {move || if loading.get() { "Executing..." } else { "Execute Query" }}
            </button>
            
            <ResultsDisplay results=results/>
        </div>
    }
}
```

#### **3. API Integration Layer**
```rust
// src/frontend/hooks/use_api.rs
use leptos::*;
use reqwest::Client;

pub fn use_api() -> ApiClient {
    ApiClient::new()
}

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "/api/v1".to_string(),
        }
    }
    
    pub async fn execute_sparql(&self, query: &str) -> Result<SparqlResult, ApiError> {
        self.client
            .post(&format!("{}/sparql/query", self.base_url))
            .json(&SparqlRequest { query: query.to_string() })
            .send()
            .await?
            .json()
            .await
            .map_err(|e| ApiError::from(e))
    }
    
    pub async fn get_ontologies(&self) -> Result<Vec<OntologyInfo>, ApiError> {
        self.client
            .get(&format!("{}/ontologies", self.base_url))
            .send()
            .await?
            .json()
            .await
            .map_err(|e| ApiError::from(e))
    }
}
```

## 🎨 UI Design System

### **Color Palette**
- **Primary**: #2563eb (Blue)
- **Secondary**: #64748b (Slate)
- **Success**: #10b981 (Green)
- **Warning**: #f59e0b (Amber)
- **Error**: #ef4444 (Red)
- **Background**: #ffffff (White)
- **Surface**: #f8fafc (Light gray)

### **Typography**
- **Font Family**: Inter, system-ui, sans-serif
- **Font Sizes**: 12px, 14px, 16px, 18px, 24px, 32px
- **Font Weights**: 400, 500, 600, 700

### **Component Library**
- **Buttons**: Multiple variants (primary, secondary, ghost, link)
- **Inputs**: Consistent styling with validation states
- **Cards**: Flexible container components
- **Modals**: Dialog overlays for forms and confirmations
- **Tables**: Sortable, filterable data tables
- **Charts**: Reusable chart components

### **Layout System**
- **Grid**: 12-column responsive grid system
- **Spacing**: 4px base unit (8px, 16px, 24px, 32px, etc.)
- **Breakpoints**: Mobile (640px), Tablet (768px), Desktop (1024px)

## 📊 Testing Strategy

### **Component Testing**
- Unit tests for individual components
- Integration tests for API calls
- Snapshot tests for UI rendering

### **End-to-End Testing**
- Full workflow testing
- Cross-browser compatibility
- Performance benchmarking

### **User Acceptance Testing**
- Supply chain domain expert testing
- Usability testing
- Performance validation

## 🚀 Deployment Strategy

### **Development**
- Local development with hot reload
- Integrated development environment
- Debug tools and browser extensions

### **Production**
- Optimized WASM builds
- Static asset optimization
- CDN integration for assets
- Container deployment with existing backend

## 📈 Success Metrics

### **Performance Targets**
- **Page Load Time**: < 2 seconds
- **Time to Interactive**: < 3 seconds
- **Bundle Size**: < 1MB (gzipped)
- **Runtime Performance**: 60fps animations

### **User Experience Targets**
- **Task Success Rate**: > 95%
- **User Satisfaction**: > 4.5/5
- **Error Rate**: < 1%
- **Accessibility**: WCAG 2.1 AA compliant

## 🔄 Maintenance and Updates

### **Code Quality**
- Regular code reviews
- Automated testing and linting
- Performance monitoring
- Security updates

### **Feature Development**
- Regular user feedback collection
- Incremental feature releases
- Documentation updates
- Training and onboarding

## 📝 Documentation

### **Developer Documentation**
- Component API documentation
- Style guide and design system
- Testing guidelines
- Deployment procedures

### **User Documentation**
- User guide and tutorials
- Feature documentation
- Troubleshooting guide
- FAQ and support

---

**Created**: 2025-09-06  
**Version**: 1.0  
**Status**: Ready for Implementation  
**Next Review**: After Phase 1 completion