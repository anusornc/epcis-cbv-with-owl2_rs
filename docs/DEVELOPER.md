# EPCIS Knowledge Graph Developer Guide

## Architecture Overview

The EPCIS Knowledge Graph is built using a modular architecture that combines semantic reasoning with graph database technology. This guide provides detailed information for developers who want to understand, extend, or contribute to the project.

## System Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   REST API       │    │   CLI Interface  │    │   Web UI         │
│   (Axum)         │    │   (Clap)         │    │   (Static)       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Core System   │
                    │   (lib.rs)      │
                    └─────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Ontology      │    │   Storage       │    │   Monitoring    │
│   Management    │    │   Management    │    │   & Logging     │
│   (ontology/)   │    │   (storage/)    │    │   (monitoring/) │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Core Components

### 1. Main Application (`src/main.rs`)

The main entry point that:
- Parses command-line arguments
- Initializes the application
- Starts the web server or runs CLI commands

**Key Functions:**
- `main()`: Application entry point
- `start_web_server()`: Initialize and start the HTTP server
- CLI command handlers for ontology, reasoning, and monitoring operations

### 2. Core Library (`src/lib.rs`)

Central library that coordinates all components:
- Error handling with `EpcisKgError` enum
- Configuration management
- Module exports
- Common utilities

### 3. Ontology Management (`src/ontology/`)

Handles ontology loading, validation, and reasoning.

#### `loader.rs`
- Loads ontologies from various formats (Turtle, RDF/XML, JSON-LD)
- Validates ontology syntax
- Converts to internal representations

**Key Structs:**
```rust
pub struct OntologyLoader {
    supported_formats: Vec<OntologyFormat>,
    validation_rules: Vec<ValidationRule>,
}
```

**Key Functions:**
- `load_from_file()`: Load ontology from file
- `load_from_url()`: Load ontology from URL
- `validate_ontology()`: Validate ontology structure

#### `reasoner.rs`
- Integrates with owl2_rs for OWL 2 reasoning
- Performs materialization and inference
- Manages reasoning cache and performance optimization

**Key Structs:**
```rust
pub struct OntologyReasoner {
    config: Config,
    store: Option<OxigraphStore>,
    owl_ontology: Option<Ontology>,
    owl_reasoner: Option<api::Reasoner>,
    reasoning_cache: HashMap<String, Vec<String>>,
    materialized_triples: HashMap<String, Vec<oxrdf::Triple>>,
    inference_stats: InferenceStats,
    materialization_strategy: MaterializationStrategy,
    // Performance optimization fields
    parallel_processing: bool,
    cache_size_limit: usize,
    performance_metrics: PerformanceMetrics,
    index_structures: IndexStructures,
    batch_size: usize,
}
```

**Key Functions:**
- `load_ontology_data()`: Load and parse ontology
- `perform_inference()`: Perform reasoning operations
- `materialize_inferences()`: Materialize inferred triples
- `perform_parallel_inference()`: Parallel reasoning processing

### 4. Storage Management (`src/storage/`)

Manages data storage using Oxigraph.

#### `oxigraph_store.rs`
- Wraps Oxigraph triple store
- Handles SPARQL queries and updates
- Manages named graphs and datasets

**Key Structs:**
```rust
pub struct OxigraphStore {
    store: oxigraph::store::Store,
    config: StorageConfig,
}
```

**Key Functions:**
- `new()`: Create new store instance
- `query()`: Execute SPARQL SELECT queries
- `update()`: Execute SPARQL UPDATE operations
- `add_triple()`: Add individual triples
- `get_statistics()`: Get storage statistics

### 5. API Layer (`src/api/`)

REST API implementation using Axum.

#### `server.rs`
- HTTP server setup and routing
- State management
- Request handling

**Key Structs:**
```rust
pub struct WebServer {
    config: Arc<AppConfig>,
    store: Arc<Mutex<OxigraphStore>>,
    reasoner: Arc<RwLock<OntologyReasoner>>,
    pipeline: Arc<EpcisEventPipeline>,
    system_monitor: Arc<SystemMonitor>,
    logging_config: Arc<LoggingConfig>,
}
```

#### `routes.rs`
- API route definitions
- Request/response handlers
- Error handling

**Key Endpoints:**
- `/health`: Health check
- `/api/v1/ontologies/*`: Ontology management
- `/api/v1/sparql/*`: SPARQL operations
- `/api/v1/events/*`: EPCIS event processing
- `/api/v1/reasoning/*`: Reasoning operations
- `/api/v1/monitoring/*`: System monitoring

### 6. Event Processing (`src/models/`)

Handles EPCIS event processing and validation.

#### `events.rs`
- EPCIS event parsing and validation
- Event processing pipeline
- Business rule validation

**Key Structs:**
```rust
pub struct EpcisEvent {
    pub event_time: DateTime<Utc>,
    pub event_time_zone_offset: String,
    pub epc_list: Vec<String>,
    pub action: String,
    pub biz_step: String,
    pub disposition: String,
    pub read_point: ReadPoint,
    pub biz_location: Option<BizLocation>,
    pub biz_transaction_list: Option<Vec<BizTransaction>>,
}

pub struct EpcisEventPipeline {
    validator: EventValidator,
    processor: EventProcessor,
    reasoner: Arc<RwLock<OntologyReasoner>>,
}
```

### 7. Monitoring and Logging (`src/monitoring/`)

System monitoring and structured logging.

#### `metrics.rs`
- System metrics collection
- Performance monitoring
- Alert generation

**Key Structs:**
```rust
pub struct SystemMonitor {
    metrics: Arc<RwLock<SystemMetrics>>,
    alerts: Arc<RwLock<Vec<Alert>>>,
    config: MonitoringConfig,
}

pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub active_connections: u32,
    pub database_metrics: DatabaseMetrics,
    pub reasoning_metrics: ReasoningMetrics,
    pub api_metrics: ApiMetrics,
}
```

#### `logging.rs`
- Structured logging framework
- Multiple logger types
- Configurable output formats

**Key Structs:**
```rust
pub struct LoggingConfig {
    pub level: String,
    pub console_output: bool,
    pub file_output: bool,
    pub log_directory: PathBuf,
    pub max_file_size_mb: usize,
    pub max_files: usize,
    pub include_timestamps: bool,
    pub include_request_ids: bool,
    pub format: LogFormat,
}

pub enum LogFormat {
    Json,
    Text,
}
```

## Development Setup

### Prerequisites

- Rust 1.75+
- Cargo
- Git
- (Optional) Docker for containerized development

### Building the Project

```bash
# Clone the repository
git clone <repository-url>
cd epcis_kg_rust

# Build in development mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Format code
cargo fmt

# Run linter
cargo clippy
```

### Development Environment

#### VS Code Setup

1. Install the Rust extension
2. Install the CodeLLDB extension for debugging
3. Configure `.vscode/settings.json`:

```json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "files.associations": {
        "*.toml": "toml"
    }
}
```

#### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_ontology_reasoner_creation

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_tests

# Run benchmarks
cargo bench
```

### Adding New Features

#### 1. Adding New API Endpoints

1. Define the route in `src/api/routes.rs`
2. Implement the handler function
3. Add tests in `tests/api_integration.rs`
4. Update API documentation

**Example:**
```rust
// routes.rs
pub fn api_new_endpoint() -> Json<ApiResponse> {
    Json(ApiResponse {
        success: true,
        data: serde_json::json!({"message": "New endpoint"}),
        message: "Success".to_string(),
        timestamp: Utc::now(),
    })
}

// Register route
Router::new()
    .route("/api/v1/new-endpoint", get(api_new_endpoint))
```

#### 2. Adding New CLI Commands

1. Add command to `main.rs` CLI definition
2. Implement handler function
3. Add tests

**Example:**
```rust
// main.rs
#[derive(Subcommand)]
enum Commands {
    NewCommand {
        #[arg(short, long)]
        parameter: String,
    },
}

// Handler
fn handle_new_command(parameter: String) -> Result<(), EpcisKgError> {
    // Implementation
    Ok(())
}
```

#### 3. Adding New Ontology Features

1. Extend `OntologyReasoner` in `reasoner.rs`
2. Add validation rules
3. Update materialization logic
4. Add tests

#### 4. Adding New Storage Features

1. Extend `OxigraphStore` in `oxigraph_store.rs`
2. Add new query/update methods
3. Add tests

## Code Standards

### Rust Conventions

- Follow Rust API Guidelines (RFC 2846)
- Use `Result<T, EpcisKgError>` for error handling
- Prefer `Arc<Mutex<T>>` for shared mutable state
- Use `Arc<RwLock<T>>` for mostly-read shared state
- Document all public APIs with Rustdoc

### Error Handling

```rust
// Define custom error type
#[derive(Debug, thiserror::Error)]
pub enum EpcisKgError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Ontology error: {0}")]
    Ontology(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
}

// Use in functions
pub fn example_function() -> Result<(), EpcisKgError> {
    // Implementation
    Ok(())
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_example() {
        let temp_dir = TempDir::new().unwrap();
        let result = example_function();
        assert!(result.is_ok());
    }
}
```

### Logging

```rust
use tracing::{info, warn, error, debug};

pub fn example_function() {
    info!("Starting example function");
    debug!("Detailed debug information");
    warn!("This is a warning");
    error!("This is an error");
}
```

## Performance Considerations

### 1. Memory Management

- Use `Arc` for shared ownership
- Use `Mutex` for exclusive access
- Use `RwLock` for read-heavy access patterns
- Monitor memory usage with system metrics

### 2. Concurrency

- Use `tokio` for async operations
- Avoid blocking operations in async contexts
- Use `rayon` for CPU-bound parallel processing
- Monitor lock contention

### 3. Database Operations

- Use prepared statements for repeated queries
- Batch operations for bulk inserts
- Monitor query performance
- Use appropriate indexes

### 4. Caching

- Implement LRU cache for frequently accessed data
- Use cache invalidation strategies
- Monitor cache hit rates
- Set appropriate cache sizes

## Debugging

### 1. Logging

```rust
// Enable debug logging
RUST_LOG=debug cargo run

// Enable specific module logging
RUST_LOG=epcis_kg::ontology=debug cargo run
```

### 2. Testing

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Run specific test with output
cargo test test_name -- --nocapture
```

### 3. Performance Analysis

```bash
# Run benchmarks
cargo bench

# Generate flamegraph
cargo install flamegraph
cargo flamegraph --bin epcis-knowledge-graph
```

## Contributing

### 1. Development Workflow

1. Fork the repository
2. Create feature branch
3. Make changes
4. Add tests
5. Run tests and benchmarks
6. Submit pull request

### 2. Code Review Checklist

- [ ] All tests pass
- [ ] Code follows style guidelines
- [ ] Documentation is updated
- [ ] Performance impact is considered
- [ ] Error handling is comprehensive
- [ ] Security implications are considered

### 3. Pull Request Template

```markdown
## Changes
- Description of changes

## Testing
- How changes were tested

## Performance
- Performance impact assessment

## Documentation
- Documentation updates
```

## Common Issues and Solutions

### 1. Compilation Errors

**Issue**: Missing dependencies
```bash
cargo update
cargo build
```

**Issue**: Type inference errors
```rust
// Add explicit type annotations
let variable: Type = expression;
```

### 2. Runtime Errors

**Issue**: Database connection failed
- Check database path permissions
- Verify database is not corrupted
- Monitor disk space

**Issue**: Memory usage high
- Check for memory leaks
- Monitor cache sizes
- Consider increasing system memory

### 3. Performance Issues

**Issue**: Slow query performance
- Check query execution plans
- Add appropriate indexes
- Consider query optimization

**Issue**: High CPU usage
- Monitor lock contention
- Consider parallel processing
- Profile CPU usage

## Resources

### Documentation

- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Oxigraph Documentation](https://docs.rs/oxigraph/latest/oxigraph/)
- [owl2_rs Documentation](https://docs.rs/owl2-rs/latest/owl2_rs/)

### Tools

- [VS Code](https://code.visualstudio.com/) with Rust extension
- [cargo-watch](https://github.com/passcod/cargo-watch) for auto-reload
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) for profiling
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) for code coverage

### Community

- [Rust Users Forum](https://users.rust-lang.org/)
- [Stack Overflow](https://stackoverflow.com/)
- [Project Repository](https://github.com/your-repo/epcis-knowledge-graph)

## Future Enhancements

### Planned Features

- [ ] GraphQL API support
- [ ] Real-time event streaming
- [ ] Advanced analytics dashboard
- [ ] Machine learning integration
- [ ] Multi-tenant support
- [ ] Advanced security features

### Technical Improvements

- [ ] Database sharding
- [ ] Caching layer optimization
- [ ] Async reasoning engine
- [ ] Performance optimizations
- [ ] Memory usage optimization