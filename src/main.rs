use clap::{Parser, Subcommand};
use epcis_knowledge_graph::{EpcisKgError, Config};
use epcis_knowledge_graph::ontology::loader::OntologyLoader;
use epcis_knowledge_graph::storage::oxigraph_store::OxigraphStore;
use epcis_knowledge_graph::ontology::reasoner::OntologyReasoner;
use epcis_knowledge_graph::pipeline::EpcisEventPipeline;
use epcis_knowledge_graph::models::epcis::EpcisEvent;
use epcis_knowledge_graph::api::server::WebServer;
use epcis_knowledge_graph::monitoring::metrics::{SystemMonitor, AlertSeverity};
use epcis_knowledge_graph::monitoring::logging::{init_logging, LoggingConfig};
use epcis_knowledge_graph::data_gen::{generator::EpcisDataGenerator, GeneratorConfig, DataScale, OutputFormat};
use epcis_knowledge_graph::benchmarks::{run_performance_benchmarks, run_custom_benchmarks, DataScale as BenchmarkDataScale};
use tracing::info;
use std::time::Instant;
use chrono;

#[derive(Parser, Debug)]
#[command(
    name = "epcis-knowledge-graph",
    about = "EPCIS Knowledge Graph demo combining owl2_rs reasoning with Oxigraph storage",
    version = "0.1.0",
    author = "Your Name <your.email@example.com>"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, default_value = "config/default.toml")]
    config: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the EPCIS Knowledge Graph server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// Use pre-generated sample data
        #[arg(long)]
        use_samples_data: bool,

        /// Sample data scale (small, medium, large) - requires --use-samples-data
        #[arg(long, default_value = "medium")]
        samples_scale: String,
    },

    /// Load ontologies into the knowledge graph
    Load {
        /// Path to ontology file(s)
        #[arg(required = true)]
        files: Vec<String>,

        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,
    },

    /// Execute a SPARQL query
    Query {
        /// SPARQL query string
        #[arg(required = true)]
        query: String,

        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// Output format (json, csv, tsv)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Validate EPCIS events
    Validate {
        /// Path to EPCIS event file
        #[arg(required = true)]
        event_file: String,

        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,
    },

    /// Perform reasoning on the knowledge graph
    Reason {
        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// OWL profile to check (el, ql, rl)
        #[arg(short, long, default_value = "el")]
        profile: String,

        /// Perform inference
        #[arg(short, long)]
        inference: bool,
    },

    /// Comprehensive OWL profile validation
    Profile {
        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// OWL profile to validate (el, ql, rl, full)
        #[arg(short, long, default_value = "el")]
        profile: String,

        /// Output format (json, text)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Process EPCIS events
    Process {
        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,
        
        /// Event file (JSON format)
        #[arg(short, long)]
        event_file: String,
        
        /// Output format (json, text)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Initialize the knowledge graph
    Init {
        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// Force initialization (overwrite existing data)
        #[arg(short, long)]
        force: bool,
    },

    /// Show current configuration
    Config,

    /// Perform inference with materialization
    Infer {
        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// Materialization strategy (full, incremental, ondemand, hybrid)
        #[arg(short, long, default_value = "incremental")]
        strategy: String,

        /// Clear existing materialized triples before inference
        #[arg(short, long)]
        clear: bool,

        /// Output format (json, text)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Manage materialized triples
    Materialize {
        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// Action (show, clear, stats)
        #[arg(required = true)]
        action: String,

        /// Graph name (optional, for specific graphs)
        #[arg(short, long)]
        graph: Option<String>,
    },

    /// Perform incremental inference on new data
    Increment {
        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// Path to file with new triples (Turtle format)
        #[arg(short, long)]
        triples_file: String,

        /// Output format (json, text)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Performance optimization commands
    Optimize {
        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// Action (configure, run, report, benchmark)
        #[arg(required = true)]
        action: String,

        /// Enable parallel processing
        #[arg(long)]
        parallel: bool,

        /// Cache size limit
        #[arg(long, default_value = "10000")]
        cache_limit: usize,

        /// Batch size for processing
        #[arg(long, default_value = "1000")]
        batch_size: usize,
    },

    /// Perform parallel inference
    ParallelInfer {
        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// Output format (json, text)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// System monitoring and metrics
    Monitor {
        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// Action (metrics, alerts, health, status)
        #[arg(required = true)]
        action: String,

        /// Output format (json, text)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Limit for alerts (default: 10)
        #[arg(long, default_value = "10")]
        limit: usize,
    },

    /// Load pre-generated sample data into the knowledge graph
    LoadSamples {
        /// Sample data scale (small, medium, large, xlarge)
        #[arg(short, long, default_value = "medium")]
        scale: String,

        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// Force reload (clear existing data first)
        #[arg(long)]
        force: bool,
    },

    /// Generate test data for the knowledge graph
    Generate {
        /// Output directory for generated data
        #[arg(short, long, default_value = "./generated_data")]
        output_path: String,

        /// Scale of generation (small, medium, large, xlarge)
        #[arg(short, long, default_value = "medium")]
        scale: String,

        /// Output format (turtle, ntriples, jsonld)
        #[arg(short, long, default_value = "turtle")]
        format: String,

        /// Number of locations to generate
        #[arg(long)]
        locations: Option<usize>,

        /// Number of products to generate
        #[arg(long)]
        products: Option<usize>,

        /// Number of events to generate
        #[arg(long)]
        events: Option<usize>,

        /// Load generated data into database
        #[arg(long)]
        load: bool,

        /// Database path (for loading)
        #[arg(short, long, default_value = "./data")]
        db_path: String,
    },

    /// Run performance benchmarks
    Benchmark {
        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// Number of iterations for each test
        #[arg(long, default_value = "10")]
        iterations: usize,

        /// Data scale (small, medium, large)
        #[arg(long, default_value = "medium")]
        scale: String,

        /// Include memory metrics (platform dependent)
        #[arg(long)]
        include_memory: bool,

        /// Output format (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), EpcisKgError> {
    let args = Args::parse();

    // Load configuration
    let config = Config::from_file_or_default(&args.config)?;
    config.validate()?;

    // Initialize structured logging system
    let logging_config = LoggingConfig {
        level: if args.verbose {
            "debug".to_string()
        } else {
            config.log_level.clone()
        },
        console_output: true,
        file_output: false,
        log_directory: std::path::PathBuf::from("./logs"),
        max_file_size_mb: 100,
        max_files: 5,
        include_timestamps: true,
        include_request_ids: false,
        format: epcis_knowledge_graph::monitoring::logging::LogFormat::Text,
    };
    
    init_logging(logging_config).map_err(|e| EpcisKgError::Config(format!("Failed to initialize logging: {}", e)))?;

    info!("Starting EPCIS Knowledge Graph with configuration from: {}", args.config);

    match args.command {
        Commands::Serve { port, db_path, use_samples_data, samples_scale } => {
            let final_port = if port != 8080 { port } else { config.server_port };
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!(
                "Starting server on port {} with database at {}",
                final_port, final_db_path
            );
            
            // Initialize the store
            let store = OxigraphStore::new(&final_db_path)?;
            
            // Load sample data if requested
            if use_samples_data {
                info!("Loading sample data with scale: {}", samples_scale);
                match load_sample_data(&samples_scale, &final_db_path, false) {
                    Ok(count) => {
                        println!("✓ Loaded {} triples of sample data", count);
                    },
                    Err(e) => {
                        eprintln!("⚠️  Failed to load sample data: {}", e);
                        eprintln!("⚠️  Continuing with empty database...");
                    }
                }
            }
            
            // Create and run the web server
            let web_server = WebServer::new(config.clone(), store).await?;
            
            println!("🚀 Starting EPCIS Knowledge Graph server...");
            println!("📊 Server will be available at: http://localhost:{}", final_port);
            println!("🔍 SPARQL endpoint: http://localhost:{}/api/v1/sparql", final_port);
            println!("📖 API documentation: http://localhost:{}/", final_port);
            if use_samples_data {
                println!("📦 Sample data loaded ({} scale)", samples_scale);
            }
            println!("⏹️  Press Ctrl+C to stop the server");
            
            if let Err(e) = web_server.run(final_port).await {
                eprintln!("❌ Server error: {}", e);
                return Err(EpcisKgError::Config(format!("Failed to start server: {}", e)));
            }
        }
        Commands::Load { files, db_path } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!(
                "Loading ontologies from {:?} into database at {}",
                files, final_db_path
            );
            load_ontologies(&files, &final_db_path)?;
        }
        Commands::Query {
            query,
            db_path,
            format,
        } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!("Executing query against database at {}", final_db_path);
            execute_query(&query, &final_db_path, &format)?;
        }
        Commands::Validate {
            event_file,
            db_path,
        } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!(
                "Validating EPCIS events from {} against database at {}",
                event_file, final_db_path
            );
            // TODO: Implement event validation
            println!("Event validation not yet implemented");
        }
        Commands::Reason { db_path, profile, inference } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            let final_profile = if profile != "el" { profile } else { config.reasoning.default_profile.clone() };
            
            info!(
                "Performing reasoning on knowledge graph at {} (profile: {}, inference: {})",
                final_db_path, final_profile, inference
            );
            perform_reasoning(&final_db_path, &final_profile, inference)?;
        }
        Commands::Profile { db_path, profile, format } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            let final_profile = if profile != "el" { profile } else { config.reasoning.default_profile.clone() };
            
            info!(
                "Performing comprehensive OWL profile validation on knowledge graph at {} (profile: {})",
                final_db_path, final_profile
            );
            perform_profile_validation(&final_db_path, &final_profile, &format)?;
        }
        Commands::Process { db_path, event_file, format } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!(
                "Processing EPCIS events from {} using knowledge graph at {}",
                event_file, final_db_path
            );
            perform_event_processing(&final_db_path, &event_file, &format)?;
        }
        Commands::Init { db_path, force } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!(
                "Initializing knowledge graph at {} (force: {})",
                final_db_path, force
            );
            initialize_knowledge_graph(&final_db_path, force, &config.ontology_paths)?;
        }
        Commands::Infer { db_path, strategy, clear, format } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!(
                "Performing inference with materialization (strategy: {}, clear: {}) on knowledge graph at {}",
                strategy, clear, final_db_path
            );
            perform_inference_with_materialization(&final_db_path, &strategy, clear, &format)?;
        }
        Commands::Materialize { db_path, action, graph } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!(
                "Managing materialized triples (action: {}) on knowledge graph at {}",
                action, final_db_path
            );
            manage_materialized_triples(&final_db_path, &action, &graph)?;
        }
        Commands::Increment { db_path, triples_file, format } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!(
                "Performing incremental inference on new data from {} using knowledge graph at {}",
                triples_file, final_db_path
            );
            perform_incremental_inference(&final_db_path, &triples_file, &format)?;
        }
        Commands::Optimize { db_path, action, parallel, cache_limit, batch_size } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!(
                "Performing optimization action '{}' on knowledge graph at {}",
                action, final_db_path
            );
            perform_optimization(&final_db_path, &action, parallel, cache_limit, batch_size)?;
        }
        Commands::ParallelInfer { db_path, format } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!(
                "Performing parallel inference using knowledge graph at {}",
                final_db_path
            );
            perform_parallel_inference(&final_db_path, &format)?;
        }
        Commands::Monitor { db_path, action, format, limit } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!(
                "Performing monitoring action '{}' using knowledge graph at {}",
                action, final_db_path
            );
            perform_monitoring_action(&final_db_path, &action, format, limit)?;
        }
        Commands::LoadSamples { scale, db_path, force } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            info!(
                "Loading sample data with scale '{}' into database at {}",
                scale, final_db_path
            );
            
            match load_sample_data(&scale, &final_db_path, force) {
                Ok(count) => {
                    println!("✓ Successfully loaded {} triples of sample data", count);
                    println!("✓ Sample data scale: {}", scale);
                    if force {
                        println!("✓ Cleared existing data before loading");
                    }
                },
                Err(e) => {
                    eprintln!("✗ Failed to load sample data: {}", e);
                    return Err(EpcisKgError::Config(format!("Failed to load sample data: {}", e)));
                }
            }
        }
        Commands::Generate { 
            output_path, 
            scale, 
            format, 
            locations, 
            products, 
            events, 
            load, 
            db_path 
        } => {
            info!(
                "Generating test data with scale '{}' to output path {}",
                scale, output_path
            );
            
            // Parse scale
            let data_scale = match scale.to_lowercase().as_str() {
                "small" => DataScale::Small,
                "medium" => DataScale::Medium,
                "large" => DataScale::Large,
                "xlarge" => DataScale::XLarge,
                _ => DataScale::Medium,
            };
            
            // Parse output format
            let output_format = match format.to_lowercase().as_str() {
                "turtle" => OutputFormat::Turtle,
                "ntriples" => OutputFormat::NTriples,
                "jsonld" => OutputFormat::JsonLd,
                _ => OutputFormat::Turtle,
            };
            
            // Create generator config
            let mut generator_config = GeneratorConfig {
                scale: data_scale,
                output_format,
                output_path: std::path::PathBuf::from(&output_path),
                custom_counts: None,
            };
            
            // Override with custom counts if provided
            if locations.is_some() || products.is_some() || events.is_some() {
                generator_config.custom_counts = Some((
                    locations.unwrap_or(0),
                    products.unwrap_or(0),
                    events.unwrap_or(0),
                ));
            }
            
            // Generate data
            let generator = EpcisDataGenerator::new();
            match generator.generate_dataset(&generator_config) {
                Ok(result) => {
                    println!("✓ Data generation completed successfully");
                    println!("  - Generated {} triples", result.triple_count);
                    println!("  - Generated {} events", result.event_count);
                    println!("  - Generated {} locations", result.location_count);
                    println!("  - Generated {} products", result.product_count);
                    println!("  - Generation time: {}ms", result.generation_time_ms);
                    
                    for file in &result.output_files {
                        println!("  - Output file: {}", file);
                    }
                    
                    // Load data into database if requested
                    if load {
                        let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
                        info!("Loading generated data into database at {}", final_db_path);
                        
                        for file in &result.output_files {
                            match load_generated_data(file, &final_db_path) {
                                Ok(count) => {
                                    println!("✓ Loaded {} triples from {}", count, file);
                                },
                                Err(e) => {
                                    eprintln!("✗ Failed to load data from {}: {}", file, e);
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("✗ Data generation failed: {}", e);
                    return Err(EpcisKgError::Config(format!("Failed to generate data: {}", e)));
                }
            }
        }
        Commands::Benchmark { db_path, iterations, scale, include_memory, format } => {
            let final_db_path = if db_path != "./data" { db_path } else { config.database_path.clone() };
            
            println!("🚀 Running performance benchmarks...");
            println!("📊 Configuration:");
            println!("  - Database: {}", final_db_path);
            println!("  - Iterations: {}", iterations);
            println!("  - Scale: {}", scale);
            println!("  - Memory metrics: {}", include_memory);
            println!("  - Format: {}", format);
            
            // Parse data scale
            let benchmark_scale = match scale.to_lowercase().as_str() {
                "small" => BenchmarkDataScale::Small,
                "medium" => BenchmarkDataScale::Medium,
                "large" => BenchmarkDataScale::Large,
                _ => BenchmarkDataScale::Medium,
            };
            
            // Run benchmarks
            let start_time = std::time::Instant::now();
            let result = if iterations == 10 && scale == "medium" {
                // Use default configuration
                run_performance_benchmarks(std::path::Path::new(&final_db_path))
            } else {
                // Use custom configuration
                run_custom_benchmarks(std::path::Path::new(&final_db_path), iterations, benchmark_scale)
            };
            
            let total_time = start_time.elapsed();
            
            match result {
                Ok(()) => {
                    println!("\n✅ Performance benchmarks completed successfully!");
                    println!("⏱️  Total benchmark time: {:?}", total_time);
                    
                    if format == "json" {
                        // JSON output would require collecting results from the benchmarks
                        // For now, just show completion
                        let json_output = serde_json::json!({
                            "status": "completed",
                            "total_time_ms": total_time.as_millis() as u64,
                            "iterations": iterations,
                            "scale": scale,
                            "memory_metrics": include_memory,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        });
                        println!("{}", serde_json::to_string_pretty(&json_output)?);
                    }
                },
                Err(e) => {
                    eprintln!("❌ Performance benchmarks failed: {}", e);
                    return Err(e);
                }
            }
        }
        Commands::Config => {
            show_configuration(&config)?;
        }
    }

    Ok(())
}

/// Load ontologies from files into the knowledge graph
fn load_ontologies(files: &[String], db_path: &str) -> Result<(), EpcisKgError> {
    let mut store = OxigraphStore::new(db_path)?;
    let loader = OntologyLoader::new();
    
    println!("Loading ontologies...");
    let mut total_triples = 0;
    
    for file in files {
        info!("Loading ontology from: {}", file);
        match loader.load_ontology(file) {
            Ok(ontology_data) => {
                store.store_ontology_data(&ontology_data)?;
                println!("✓ Loaded {} triples from {}", ontology_data.triples_count, file);
                total_triples += ontology_data.triples_count;
                
                // Print basic statistics
                let stats = loader.get_statistics(&ontology_data);
                println!("  - Classes: {}", stats.classes);
                println!("  - Properties: {}", stats.properties);
                println!("  - Individuals: {}", stats.individuals);
            },
            Err(e) => {
                eprintln!("✗ Failed to load ontology from {}: {}", file, e);
                return Err(e);
            }
        }
    }
    
    let store_stats = store.get_statistics()?;
    println!("\n✓ Successfully loaded {} total triples", total_triples);
    println!("  - Named graphs: {}", store_stats.named_graphs);
    println!("  - Storage path: {}", store_stats.storage_path);
    
    Ok(())
}

/// Execute a SPARQL query against the knowledge graph
fn execute_query(query: &str, db_path: &str, format: &str) -> Result<(), EpcisKgError> {
    let store = OxigraphStore::new(db_path)?;
    
    info!("Executing SPARQL query: {}", query);
    
    // Determine query type and execute
    let query_upper = query.to_uppercase();
    let result = if query_upper.contains("SELECT") {
        store.query_select(query)?
    } else if query_upper.contains("ASK") {
        let result = store.query_ask(query)?;
        format!("{{\"boolean\": {}}}", result)
    } else if query_upper.contains("CONSTRUCT") {
        store.query_construct(query)?
    } else {
        return Err(EpcisKgError::Query("Unsupported SPARQL query type".to_string()));
    };
    
    // Output results based on format
    match format.to_lowercase().as_str() {
        "json" => {
            println!("{}", result);
        },
        "csv" | "tsv" => {
            // For CSV/TSV, we'd need to parse the JSON and convert
            // For now, just output the JSON
            println!("CSV/TSV format not yet implemented, showing JSON:");
            println!("{}", result);
        },
        _ => {
            return Err(EpcisKgError::Config(format!("Unsupported output format: {}", format)));
        }
    }
    
    Ok(())
}

/// Perform reasoning on the knowledge graph
fn perform_reasoning(db_path: &str, profile: &str, inference: bool) -> Result<(), EpcisKgError> {
    let store = OxigraphStore::new(db_path)?;
    let mut reasoner = OntologyReasoner::with_store(store);
    
    println!("Performing reasoning with OWL 2 {} profile", profile.to_uppercase());
    
    // Load ontologies for validation
    let loader = OntologyLoader::new();
    let mut validation_results = Vec::new();
    
    // Try to validate ontologies if they exist
    let ontology_files = vec!["ontologies/epcis2.ttl", "ontologies/cbv.ttl"];
    
    for file in ontology_files {
        if std::path::Path::new(file).exists() {
            match loader.load_ontology(file) {
                Ok(ontology_data) => {
                    match reasoner.validate_ontology(&ontology_data) {
                        Ok(()) => {
                            validation_results.push(format!("✓ {} validation passed", file));
                        },
                        Err(e) => {
                            validation_results.push(format!("✗ {} validation failed: {}", file, e));
                        }
                    }
                    
                    // Check OWL profile
                    match reasoner.check_owl_profile(&ontology_data, profile) {
                        Ok(()) => {
                            validation_results.push(format!("✓ {} {} profile compliant", file, profile.to_uppercase()));
                        },
                        Err(e) => {
                            validation_results.push(format!("✗ {} {} profile violation: {}", file, profile.to_uppercase(), e));
                        }
                    }
                },
                Err(e) => {
                    validation_results.push(format!("✗ Failed to load {}: {}", file, e));
                }
            }
        }
    }
    
    // Print validation results
    println!("\nOntology Validation Results:");
    for result in validation_results {
        println!("  {}", result);
    }
    
    // Perform inference if requested
    if inference {
        println!("\nPerforming inference...");
        match reasoner.perform_inference() {
            Ok(inferences) => {
                println!("✓ Inference completed successfully");
                for (i, inference) in inferences.iter().enumerate() {
                    println!("  {}: {}", i + 1, inference);
                }
            },
            Err(e) => {
                eprintln!("✗ Inference failed: {}", e);
            }
        }
    }
    
    // Show reasoning statistics
    match reasoner.get_reasoning_stats() {
        Ok(stats) => {
            println!("\nReasoning Statistics:");
            println!("  {}", stats);
        },
        Err(e) => {
            eprintln!("Failed to get reasoning stats: {}", e);
        }
    }
    
    Ok(())
}

/// Initialize the knowledge graph
fn initialize_knowledge_graph(db_path: &str, force: bool, default_ontologies: &[String]) -> Result<(), EpcisKgError> {
    let path = std::path::Path::new(db_path);
    
    if path.exists() && !force {
        return Err(EpcisKgError::Config(format!(
            "Database path {} already exists. Use --force to overwrite.",
            db_path
        )));
    }
    
    if force && path.exists() {
        info!("Removing existing database at {}", db_path);
        std::fs::remove_dir_all(path)?;
    }
    
    // Create the database directory
    std::fs::create_dir_all(path)?;
    
    // Initialize an empty store
    let mut store = OxigraphStore::new(db_path)?;
    
    // Load default ontologies if they exist
    let mut loaded_count = 0;
    let loader = OntologyLoader::new();
    
    for ontology_file in default_ontologies {
        if std::path::Path::new(ontology_file).exists() {
            info!("Loading default ontology: {}", ontology_file);
            match loader.load_ontology(ontology_file) {
                Ok(ontology_data) => {
                    store.store_ontology_data(&ontology_data)?;
                    loaded_count += 1;
                    println!("✓ Loaded {} triples from {}", ontology_data.triples_count, ontology_file);
                },
                Err(e) => {
                    eprintln!("Warning: Failed to load default ontology {}: {}", ontology_file, e);
                }
            }
        } else {
            info!("Default ontology not found: {}", ontology_file);
        }
    }
    
    let stats = store.get_statistics()?;
    println!("✓ Knowledge graph initialized at {}", db_path);
    println!("  - Loaded {} default ontologies", loaded_count);
    println!("  - Total triples: {}", stats.total_quads);
    println!("  - Named graphs: {}", stats.named_graphs);
    
    Ok(())
}

/// Show current configuration
fn show_configuration(config: &Config) -> Result<(), EpcisKgError> {
    println!("Current Configuration:");
    println!("  Database Path: {}", config.database_path);
    println!("  Server Port: {}", config.server_port);
    println!("  Log Level: {}", config.log_level);
    println!("  Ontology Paths:");
    for path in &config.ontology_paths {
        println!("    - {}", path);
    }
    println!("  Reasoning:");
    println!("    - Default Profile: {}", config.reasoning.default_profile);
    println!("    - Enable Inference: {}", config.reasoning.enable_inference);
    println!("    - Max Inference Time: {}s", config.reasoning.max_inference_time);
    println!("  SPARQL:");
    println!("    - Max Query Time: {}s", config.sparql.max_query_time);
    println!("    - Max Results: {}", config.sparql.max_results);
    println!("    - Enable Updates: {}", config.sparql.enable_updates);
    println!("  Server:");
    println!("    - Enable CORS: {}", config.server.enable_cors);
    println!("    - CORS Origins: {:?}", config.server.cors_origins);
    println!("    - Request Timeout: {}s", config.server.request_timeout);
    println!("  Persistence:");
    println!("    - Auto Save: {}", config.persistence.auto_save);
    println!("    - Save Interval: {}s", config.persistence.save_interval);
    println!("    - Backup on Startup: {}", config.persistence.backup_on_startup);
    
    Ok(())
}

/// Perform comprehensive OWL profile validation
fn perform_profile_validation(db_path: &str, profile: &str, format: &str) -> Result<(), EpcisKgError> {
    let store = OxigraphStore::new(db_path)?;
    let mut reasoner = OntologyReasoner::with_store(store);
    
    println!("Performing comprehensive OWL 2 {} profile validation", profile.to_uppercase());
    
    // Load ontologies for validation
    let loader = OntologyLoader::new();
    let mut validation_results = Vec::new();
    
    // Try to load and validate each ontology
    let default_ontologies = vec![
        "ontologies/epcis2.ttl".to_string(),
        "ontologies/cbv.ttl".to_string(),
    ];
    
    for ontology_file in &default_ontologies {
        if std::path::Path::new(ontology_file).exists() {
            println!("Validating ontology: {}", ontology_file);
            
            match loader.load_ontology(ontology_file) {
                Ok(ontology_data) => {
                    match reasoner.validate_owl_profile_comprehensive(&ontology_data, profile) {
                        Ok(result) => {
                            validation_results.push((ontology_file.clone(), result));
                        },
                        Err(e) => {
                            eprintln!("✗ Failed to validate {}: {}", ontology_file, e);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("✗ Failed to load {}: {}", ontology_file, e);
                }
            }
        }
    }
    
    // Display results
    if format == "json" {
        let json_output = serde_json::json!({
            "profile": profile,
            "validation_results": validation_results,
            "summary": {
                "total_ontologies": validation_results.len(),
                "conforming_ontologies": validation_results.iter().filter(|(_, r)| r.conforms).count(),
                "non_conforming_ontologies": validation_results.iter().filter(|(_, r)| !r.conforms).count(),
            }
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Text format
        println!("\n=== OWL 2 {} Profile Validation Results ===", profile.to_uppercase());
        
        for (file, result) in &validation_results {
            println!("\n📄 {}", file);
            println!("  Status: {}", if result.conforms { "✅ Conforms" } else { "❌ Non-conforming" });
            
            if !result.violations.is_empty() {
                println!("  Violations:");
                for violation in &result.violations {
                    println!("    - {}", violation);
                }
            }
            
            println!("  Ontology Stats:");
            println!("    - Total Axioms: {}", result.ontology_stats.total_axioms);
            println!("    - Classes: {}", result.ontology_stats.classes);
            println!("    - Properties: {}", result.ontology_stats.properties);
            println!("    - Individuals: {}", result.ontology_stats.individuals);
            
            println!("  EPCIS Compliance:");
            println!("    - EPCIS Classes: {}", if result.epcis_compliance.has_epcis_classes { "✅" } else { "❌" });
            println!("    - CBV Vocabulary: {}", if result.epcis_compliance.has_cbv_vocabulary { "✅" } else { "❌" });
            println!("    - Event Types: {}", if result.epcis_compliance.has_event_types { "✅" } else { "❌" });
            println!("    - Vocabulary Extensions: {}", if result.epcis_compliance.has_vocabulary_extensions { "✅" } else { "❌" });
            
            println!("  Performance Indicators:");
            println!("    - Estimated Classification Time: {}ms", result.performance_indicators.estimated_classification_time_ms);
            println!("    - Estimated Realization Time: {}ms", result.performance_indicators.estimated_realization_time_ms);
            println!("    - Complexity: {}", result.performance_indicators.ontology_complexity);
            println!("    - Feasibility: {}", result.performance_indicators.reasoning_feasibility);
            
            if let Some(el_specific) = &result.el_specific {
                println!("  EL Profile Analysis:");
                println!("    - Existential Restrictions: {}", el_specific.existential_restrictions);
                println!("    - Conjunctions: {}", el_specific.conjunctions);
                println!("    - Optimization Potential: {}", if el_specific.el_optimization_potential { "✅" } else { "❌" });
            }
            
            if let Some(ql_specific) = &result.ql_specific {
                println!("  QL Profile Analysis:");
                println!("    - Simple Inclusions: {}", ql_specific.simple_inclusions);
                println!("    - Query Rewriting Potential: {}", if ql_specific.query_rewriting_potential { "✅" } else { "❌" });
            }
            
            if let Some(rl_specific) = &result.rl_specific {
                println!("  RL Profile Analysis:");
                println!("    - Property Chains: {}", rl_specific.property_chains);
                println!("    - Simple Rules: {}", rl_specific.simple_rules);
                println!("    - Rule Safety: {}", if rl_specific.rule_safety { "✅" } else { "❌" });
            }
        }
        
        println!("\n=== Summary ===");
        let total = validation_results.len();
        let conforming = validation_results.iter().filter(|(_, r)| r.conforms).count();
        let non_conforming = total - conforming;
        
        println!("Total ontologies: {}", total);
        println!("Conforming: {}", conforming);
        println!("Non-conforming: {}", non_conforming);
        println!("Success rate: {:.1}%", (conforming as f64 / total as f64) * 100.0);
    }
    
    Ok(())
}

/// Perform EPCIS event processing
fn perform_event_processing(db_path: &str, event_file: &str, format: &str) -> Result<(), EpcisKgError> {
    let store = OxigraphStore::new(db_path)?;
    let reasoner = OntologyReasoner::with_store(store.clone());
    
    println!("Processing EPCIS events from: {}", event_file);
    
    // Load events from file
    let events = load_events_from_file(event_file)?;
    println!("Loaded {} events from file", events.len());
    
    // Create event processing pipeline
    let config = Config::default();
    let mut pipeline = futures::executor::block_on(EpcisEventPipeline::new(
        config,
        store,
        reasoner,
    ))?;
    
    // Process events
    let start_time = std::time::Instant::now();
    let results = futures::executor::block_on(pipeline.process_events_batch(events));
    let processing_time = start_time.elapsed();
    
    // Display results
    if format == "json" {
        let json_output = serde_json::json!({
            "event_file": event_file,
            "total_events": results.len(),
            "successful_events": results.iter().filter(|r| r.success).count(),
            "failed_events": results.iter().filter(|r| !r.success).count(),
            "total_processing_time_ms": processing_time.as_millis() as u64,
            "average_processing_time_ms": results.iter()
                .map(|r| r.processing_time_ms as f64)
                .sum::<f64>() / results.len() as f64,
            "total_triples_generated": results.iter().map(|r| r.triples_generated).sum::<usize>(),
            "total_inferences_made": results.iter().map(|r| r.inferences_made).sum::<usize>(),
            "results": results,
            "pipeline_stats": pipeline.get_stats()
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Text format
        println!("\n=== EPCIS Event Processing Results ===");
        println!("Event file: {}", event_file);
        println!("Total events: {}", results.len());
        println!("Processing time: {:?}", processing_time);
        
        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();
        
        println!("Successful events: {}", successful);
        println!("Failed events: {}", failed);
        println!("Success rate: {:.1}%", (successful as f64 / results.len() as f64) * 100.0);
        
        let total_triples: usize = results.iter().map(|r| r.triples_generated).sum();
        let total_inferences: usize = results.iter().map(|r| r.inferences_made).sum();
        let avg_processing_time = results.iter()
            .map(|r| r.processing_time_ms as f64)
            .sum::<f64>() / results.len() as f64;
        
        println!("Total triples generated: {}", total_triples);
        println!("Total inferences made: {}", total_inferences);
        println!("Average processing time: {:.2}ms", avg_processing_time);
        
        // Show failed events
        if failed > 0 {
            println!("\n=== Failed Events ===");
            for result in &results {
                if !result.success {
                    println!("✗ Event {}: {}", result.event_id, result.error.as_deref().unwrap_or("Unknown error"));
                }
            }
        }
        
        // Show pipeline statistics
        let stats = pipeline.get_stats();
        println!("\n=== Pipeline Statistics ===");
        println!("Total events processed: {}", stats.total_events_processed);
        println!("Successful events: {}", stats.successful_events);
        println!("Failed events: {}", stats.failed_events);
        println!("Validation errors: {}", stats.validation_errors);
        println!("Processing errors: {}", stats.processing_errors);
        println!("Average processing time: {:.2}ms", stats.average_processing_time_ms);
        
        if let Some(last_time) = stats.last_processed_time {
            println!("Last processed: {}", last_time);
        }
    }
    
    Ok(())
}

/// Load EPCIS events from a JSON file
fn load_events_from_file(file_path: &str) -> Result<Vec<EpcisEvent>, EpcisKgError> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| EpcisKgError::Io(e))?;
    
    let events: Vec<EpcisEvent> = serde_json::from_str(&content)
        .map_err(|e| EpcisKgError::Json(e))?;
    
    Ok(events)
}

/// Perform inference with materialization
fn perform_inference_with_materialization(db_path: &str, strategy: &str, clear: bool, format: &str) -> Result<(), EpcisKgError> {
    let store = OxigraphStore::new(db_path)?;
    let mut reasoner = OntologyReasoner::with_store(store);
    
    println!("Performing inference with materialization strategy: {}", strategy);
    
    // Set materialization strategy
    let materialization_strategy = match strategy.to_lowercase().as_str() {
        "full" => epcis_knowledge_graph::ontology::reasoner::MaterializationStrategy::Full,
        "incremental" => epcis_knowledge_graph::ontology::reasoner::MaterializationStrategy::Incremental,
        "ondemand" | "on-demand" => epcis_knowledge_graph::ontology::reasoner::MaterializationStrategy::OnDemand,
        "hybrid" => epcis_knowledge_graph::ontology::reasoner::MaterializationStrategy::Hybrid,
        _ => {
            return Err(EpcisKgError::Config(format!("Unknown materialization strategy: {}", strategy)));
        }
    };
    
    reasoner.set_materialization_strategy(materialization_strategy.clone());
    
    // Clear existing materialized triples if requested
    if clear {
        println!("Clearing existing materialized triples...");
        reasoner.clear_materialized_triples();
    }
    
    // Load ontologies for inference
    let loader = OntologyLoader::new();
    let mut ontology_loaded = false;
    
    let ontology_files = vec!["ontologies/epcis2.ttl", "ontologies/cbv.ttl"];
    
    for file in ontology_files {
        if std::path::Path::new(file).exists() {
            match loader.load_ontology(file) {
                Ok(ontology_data) => {
                    match reasoner.load_ontology_data(&ontology_data) {
                        Ok(()) => {
                            println!("✓ Loaded ontology for inference: {}", file);
                            ontology_loaded = true;
                        },
                        Err(e) => {
                            eprintln!("✗ Failed to load ontology data for inference {}: {}", file, e);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("✗ Failed to load ontology {}: {}", file, e);
                }
            }
        }
    }
    
    if !ontology_loaded {
        return Err(EpcisKgError::Validation("No ontologies loaded for inference".to_string()));
    }
    
    // Perform inference with materialization
    println!("Performing inference with materialization...");
    let start_time = std::time::Instant::now();
    
    match reasoner.perform_inference_with_materialization() {
        Ok(result) => {
            let processing_time = start_time.elapsed();
            
            // Display results
            if format == "json" {
                let stats = reasoner.get_detailed_stats();
                let json_output = serde_json::json!({
                    "inference_result": result,
                    "materialization_strategy": strategy,
                    "processing_time_ms": processing_time.as_millis() as u64,
                    "inference_stats": stats,
                    "materialized_triples_count": reasoner.get_materialized_triples().len()
                });
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                // Text format
                println!("\n=== Inference with Materialization Results ===");
                println!("Strategy: {}", strategy);
                println!("Processing time: {:?}", processing_time);
                println!("Consistent: {}", if result.consistent { "✅ Yes" } else { "❌ No" });
                println!("Classification performed: {}", if result.classification_performed { "✅ Yes" } else { "❌ No" });
                println!("Realization performed: {}", if result.realization_performed { "✅ Yes" } else { "❌ No" });
                println!("Materialized triples: {}", result.materialized_triples);
                println!("SPARQL inferences: {}", result.sparql_inferences);
                println!("Individuals classified: {}", result.individuals_classified);
                println!("Incremental: {}", if result.incremental { "✅ Yes" } else { "❌ No" });
                println!("New triples processed: {}", result.new_triples_processed);
                
                if !result.inference_errors.is_empty() {
                    println!("\nInference Errors:");
                    for error in &result.inference_errors {
                        println!("  - {}", error);
                    }
                }
                
                // Show detailed statistics
                let stats = reasoner.get_detailed_stats();
                println!("\n=== Detailed Statistics ===");
                println!("Total inferences: {}", stats.total_inferences);
                println!("Incremental inferences: {}", stats.incremental_inferences);
                println!("Full inferences: {}", stats.full_inferences);
                println!("Materialized triples count: {}", stats.materialized_triples_count);
                println!("Total processing time: {}ms", stats.total_processing_time_ms);
                println!("Average processing time: {:.2}ms", stats.average_processing_time_ms);
                println!("Cache hits: {}", stats.cache_hits);
                println!("Cache misses: {}", stats.cache_misses);
                println!("Cache hit rate: {:.2}%", stats.cache_hit_rate() * 100.0);
                
                // Show materialized triples sample
                let materialized = reasoner.get_materialized_triples();
                if !materialized.is_empty() {
                    println!("\n=== Materialized Triples (Sample) ===");
                    let mut count = 0;
                    for (_graph_name, triples) in materialized {
                        for triple in triples {
                            if count >= 5 { break; }
                            println!("  {}. {} {} {}", count + 1, triple.subject, triple.predicate, triple.object);
                            count += 1;
                        }
                        if count >= 5 { break; }
                    }
                    let total_triples: usize = materialized.values().map(|v| v.len()).sum();
                    if total_triples > 5 {
                        println!("  ... and {} more", total_triples - 5);
                    }
                }
            }
        },
        Err(e) => {
            eprintln!("✗ Inference with materialization failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Manage materialized triples
fn manage_materialized_triples(db_path: &str, action: &str, graph: &Option<String>) -> Result<(), EpcisKgError> {
    let store = OxigraphStore::new(db_path)?;
    let mut reasoner = OntologyReasoner::with_store(store);
    
    println!("Managing materialized triples - Action: {}", action);
    
    match action.to_lowercase().as_str() {
        "show" => {
            let materialized = reasoner.get_materialized_triples();
            let stats = reasoner.get_detailed_stats();
            
            println!("\n=== Materialized Triples ===");
            println!("Total materialized triples: {}", materialized.len());
            println!("Materialization strategy: {:?}", stats.strategy);
            println!("Total inferences performed: {}", stats.total_inferences);
            println!("Last inference time: {:?}", stats.last_inference_time);
            
            if let Some(graph_name) = graph {
                // Show triples for specific graph
                println!("\nTriples in graph '{}':", graph_name);
                if let Some(triples) = reasoner.get_materialized_triples_for_graph(graph_name) {
                    for (i, triple) in triples.iter().enumerate() {
                        println!("  {}. {} {} {}", i + 1, triple.subject, triple.predicate, triple.object);
                    }
                } else {
                    println!("  No triples found in graph '{}'", graph_name);
                }
            } else {
                // Show all materialized triples
                let total_triples: usize = materialized.values().map(|v| v.len()).sum();
                if total_triples > 0 {
                    println!("\nAll materialized triples:");
                    let mut count = 0;
                    for (_graph_name, triples) in materialized {
                        for triple in triples {
                            println!("  {}. {} {} {}", count + 1, triple.subject, triple.predicate, triple.object);
                            count += 1;
                        }
                    }
                } else {
                    println!("No materialized triples found");
                }
                
                // Show by graph
                if !reasoner.get_materialized_triples().is_empty() {
                    println!("\nBy graph:");
                    for (graph_name, triples) in reasoner.get_materialized_triples() {
                        println!("  '{}': {} triples", graph_name, triples.len());
                    }
                }
            }
        },
        "clear" => {
            let count = reasoner.get_materialized_triples().len();
            reasoner.clear_materialized_triples();
            println!("✓ Cleared {} materialized triples", count);
            
            if let Some(graph_name) = graph {
                println!("Cleared triples for graph: '{}'", graph_name);
            }
        },
        "stats" => {
            let stats = reasoner.get_detailed_stats();
            let materialized = reasoner.get_materialized_triples();
            
            println!("\n=== Materialization Statistics ===");
            println!("Total materialized triples: {}", materialized.len());
            println!("Materialization strategy: {:?}", stats.strategy);
            println!("Total inferences: {}", stats.total_inferences);
            println!("Incremental inferences: {}", stats.incremental_inferences);
            println!("Full inferences: {}", stats.full_inferences);
            println!("Total processing time: {}ms", stats.total_processing_time_ms);
            println!("Average processing time: {:.2}ms", stats.average_processing_time_ms);
            println!("Cache hits: {}", stats.cache_hits);
            println!("Cache misses: {}", stats.cache_misses);
            println!("Cache hit rate: {:.2}%", stats.cache_hit_rate() * 100.0);
            println!("Last inference time: {:?}", stats.last_inference_time);
            
            if let Some(graph_name) = graph {
                if let Some(triples) = reasoner.get_materialized_triples_for_graph(graph_name) {
                    println!("\nStats for graph '{}':", graph_name);
                    println!("  Triples: {}", triples.len());
                }
            }
        },
        _ => {
            return Err(EpcisKgError::Config(format!("Unknown action: {}. Use 'show', 'clear', or 'stats'", action)));
        }
    }
    
    Ok(())
}

/// Perform incremental inference on new data
fn perform_incremental_inference(db_path: &str, triples_file: &str, format: &str) -> Result<(), EpcisKgError> {
    let store = OxigraphStore::new(db_path)?;
    let mut reasoner = OntologyReasoner::with_store(store);
    
    println!("Performing incremental inference on new data from: {}", triples_file);
    
    // Load new triples from file
    let new_triples = load_triples_from_file(triples_file)?;
    println!("Loaded {} new triples from file", new_triples.len());
    
    // Perform incremental inference
    println!("Performing incremental inference...");
    let start_time = std::time::Instant::now();
    
    match reasoner.perform_incremental_inference(&new_triples) {
        Ok(result) => {
            let processing_time = start_time.elapsed();
            
            // Display results
            if format == "json" {
                let stats = reasoner.get_detailed_stats();
                let json_output = serde_json::json!({
                    "incremental_inference_result": result,
                    "new_triples_count": new_triples.len(),
                    "processing_time_ms": processing_time.as_millis() as u64,
                    "inference_stats": stats,
                    "total_materialized_triples": reasoner.get_materialized_triples().len()
                });
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                // Text format
                println!("\n=== Incremental Inference Results ===");
                println!("New triples processed: {}", result.new_triples_processed);
                println!("Processing time: {:?}", processing_time);
                println!("Consistent: {}", if result.consistent { "✅ Yes" } else { "❌ No" });
                println!("Classification performed: {}", if result.classification_performed { "✅ Yes" } else { "❌ No" });
                println!("Realization performed: {}", if result.realization_performed { "✅ Yes" } else { "❌ No" });
                println!("Materialized triples: {}", result.materialized_triples);
                println!("SPARQL inferences: {}", result.sparql_inferences);
                println!("Individuals classified: {}", result.individuals_classified);
                println!("Incremental: {}", if result.incremental { "✅ Yes" } else { "❌ No" });
                
                if !result.inference_errors.is_empty() {
                    println!("\nInference Errors:");
                    for error in &result.inference_errors {
                        println!("  - {}", error);
                    }
                }
                
                // Show detailed statistics
                let stats = reasoner.get_detailed_stats();
                println!("\n=== Updated Statistics ===");
                println!("Total inferences: {}", stats.total_inferences);
                println!("Incremental inferences: {}", stats.incremental_inferences);
                println!("Full inferences: {}", stats.full_inferences);
                println!("Materialized triples count: {}", stats.materialized_triples_count);
                println!("Total processing time: {}ms", stats.total_processing_time_ms);
                println!("Average processing time: {:.2}ms", stats.average_processing_time_ms);
                
                // Show newly materialized triples
                let materialized = reasoner.get_materialized_triples();
                if !materialized.is_empty() {
                    println!("\n=== Newly Materialized Triples (Sample) ===");
                    let mut count = 0;
                    for (_graph_name, triples) in materialized {
                        for triple in triples {
                            if count >= 3 { break; }
                            println!("  {}. {} {} {}", count + 1, triple.subject, triple.predicate, triple.object);
                            count += 1;
                        }
                        if count >= 3 { break; }
                    }
                    let total_triples: usize = materialized.values().map(|v| v.len()).sum();
                    if total_triples > 3 {
                        println!("  ... and {} more", total_triples - 3);
                    }
                }
            }
        },
        Err(e) => {
            eprintln!("✗ Incremental inference failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Load triples from a Turtle file
fn load_triples_from_file(file_path: &str) -> Result<Vec<oxrdf::Triple>, EpcisKgError> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| EpcisKgError::Io(e))?;
    
    // Simple Turtle parsing for demonstration
    // In a real implementation, you'd use a proper Turtle parser
    let mut triples = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Simple triple parsing: subject predicate object .
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            if let (Ok(subject), Ok(predicate), Ok(object)) = (
                oxrdf::NamedNode::new(parts[0]),
                oxrdf::NamedNode::new(parts[1]),
                oxrdf::NamedNode::new(parts[2]),
            ) {
                let triple = oxrdf::Triple::new(subject, predicate, object);
                triples.push(triple);
            }
        }
    }
    
    Ok(triples)
}

/// Perform performance optimization actions
fn perform_optimization(db_path: &str, action: &str, parallel: bool, cache_limit: usize, batch_size: usize) -> Result<(), EpcisKgError> {
    let mut reasoner = OntologyReasoner::with_store(OxigraphStore::new(db_path)?);
    
    match action {
        "configure" => {
            reasoner.configure_performance(parallel, cache_limit, batch_size);
            println!("✓ Performance configuration updated:");
            println!("  - Parallel processing: {}", parallel);
            println!("  - Cache limit: {}", cache_limit);
            println!("  - Batch size: {}", batch_size);
        },
        "run" => {
            reasoner.configure_performance(parallel, cache_limit, batch_size);
            reasoner.optimize_performance()?;
        },
        "report" => {
            let report = reasoner.get_performance_report();
            println!("{}", report);
        },
        "benchmark" => {
            println!("Running performance benchmark...");
            run_performance_benchmark(&mut reasoner)?;
        },
        _ => {
            return Err(EpcisKgError::Config(format!("Unknown optimization action: {}. Use 'configure', 'run', 'report', or 'benchmark'", action)));
        }
    }
    
    Ok(())
}

/// Perform parallel inference
fn perform_parallel_inference(db_path: &str, format: &str) -> Result<(), EpcisKgError> {
    let mut reasoner = OntologyReasoner::with_store(OxigraphStore::new(db_path)?);
    
    match reasoner.perform_parallel_inference() {
        Ok(result) => {
            println!("\n=== Parallel Inference Results ===");
            println!("✓ Inference completed successfully");
            println!("  - Materialized triples: {}", result.materialized_triples);
            println!("  - Processing time: {}ms", result.processing_time_ms);
            println!("  - Classification performed: {}", result.classification_performed);
            println!("  - Consistent: {}", result.consistent);
            
            if format == "json" {
                let json_output = serde_json::to_string_pretty(&result)?;
                println!("\nJSON Output:");
                println!("{}", json_output);
            }
        },
        Err(e) => {
            eprintln!("✗ Parallel inference failed: {}", e);
        }
    }
    
    Ok(())
}

/// Run performance benchmarks
fn run_performance_benchmark(reasoner: &mut OntologyReasoner) -> Result<(), EpcisKgError> {
    println!("Running performance benchmarks...");
    
    // Test sequential vs parallel performance
    let iterations = 10;
    
    // Sequential benchmark
    let start_sequential = Instant::now();
    for _ in 0..iterations {
        reasoner.perform_inference_with_materialization()?;
    }
    let sequential_time = start_sequential.elapsed();
    
    // Parallel benchmark
    reasoner.configure_performance(true, 10000, 1000);
    let start_parallel = Instant::now();
    for _ in 0..iterations {
        reasoner.perform_parallel_inference()?;
    }
    let parallel_time = start_parallel.elapsed();
    
    println!("\n=== Performance Benchmark Results ===");
    println!("Iterations: {}", iterations);
    println!("Sequential time: {:?}", sequential_time);
    println!("Parallel time: {:?}", parallel_time);
    println!("Speedup: {:.2}x", sequential_time.as_secs_f64() / parallel_time.as_secs_f64());
    
    // Cache performance
    let metrics = reasoner.get_performance_metrics();
    println!("Cache hit rate: {:.1}%", metrics.cache_hit_rate() * 100.0);
    println!("Parallel operation rate: {:.1}%", metrics.parallel_operation_rate() * 100.0);
    
    Ok(())
}

/// Perform monitoring actions
fn perform_monitoring_action(_db_path: &str, action: &str, format: String, limit: usize) -> Result<(), EpcisKgError> {
    let monitor = SystemMonitor::new();
    
    match action.to_lowercase().as_str() {
        "metrics" => {
            let metrics = monitor.get_metrics();
            
            if format == "json" {
                let json_output = serde_json::json!({
                    "action": "metrics",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "metrics": metrics
                });
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                println!("\n=== System Metrics ===");
                println!("Uptime: {} seconds", metrics.uptime_seconds);
                println!("Total requests: {}", metrics.total_requests);
                println!("Successful requests: {}", metrics.successful_requests);
                println!("Failed requests: {}", metrics.failed_requests);
                println!("Average response time: {:.2}ms", metrics.avg_response_time_ms);
                println!("Memory usage: {}MB", metrics.memory_usage_mb);
                println!("CPU usage: {:.1}%", metrics.cpu_usage_percent);
                println!("Active connections: {}", metrics.active_connections);
                
                println!("\nDatabase Metrics:");
                println!("  Total triples: {}", metrics.database_metrics.total_triples);
                println!("  Named graphs: {}", metrics.database_metrics.named_graphs);
                println!("  Average query time: {:.2}ms", metrics.database_metrics.avg_query_time_ms);
                println!("  Cache hit ratio: {:.2}", metrics.database_metrics.cache_hit_ratio);
                println!("  Storage size: {}MB", metrics.database_metrics.storage_size_mb);
                
                println!("\nReasoning Metrics:");
                println!("  Total inferences: {}", metrics.reasoning_metrics.total_inferences);
                println!("  Average inference time: {:.2}ms", metrics.reasoning_metrics.avg_inference_time_ms);
                println!("  Materialized triples: {}", metrics.reasoning_metrics.materialized_triples);
                println!("  Reasoning cache hit ratio: {:.2}", metrics.reasoning_metrics.reasoning_cache_hit_ratio);
                println!("  Materialization strategy: {}", metrics.reasoning_metrics.materialization_strategy);
            }
        },
        "alerts" => {
            let alerts = monitor.get_alerts(Some(limit));
            let active_alerts = monitor.check_alerts();
            
            if format == "json" {
                let json_output = serde_json::json!({
                    "action": "alerts",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "alerts": alerts,
                    "active_alerts": active_alerts,
                    "total_alerts": alerts.len(),
                    "active_count": active_alerts.len(),
                    "limit": limit
                });
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                println!("\n=== System Alerts ===");
                println!("Total alerts: {}", alerts.len());
                println!("Active alerts: {}", active_alerts.len());
                println!("Limit: {}", limit);
                
                if alerts.is_empty() {
                    println!("✅ No alerts found");
                } else {
                    println!("\nRecent alerts:");
                    for (i, alert) in alerts.iter().enumerate() {
                        println!("  {}. [{:?}] {:?}: {}", i + 1, alert.severity, alert.alert_type, alert.message);
                        println!("     ID: {} | Time: {}", alert.id, alert.timestamp);
                        if !alert.context.is_object() || alert.context.as_object().map_or(false, |obj| !obj.is_empty()) {
                            println!("     Context: {}", alert.context);
                        }
                        println!();
                    }
                }
                
                if !active_alerts.is_empty() {
                    println!("Active alerts:");
                    for alert in &active_alerts {
                        println!("  ⚠️  [{:?}] {:?}: {}", alert.severity, alert.alert_type, alert.message);
                    }
                }
            }
        },
        "health" => {
            let metrics = monitor.get_metrics();
            let alerts = monitor.check_alerts();
            
            let health_status = if alerts.is_empty() {
                "healthy"
            } else if alerts.iter().any(|a| matches!(a.severity, AlertSeverity::Critical)) {
                "critical"
            } else if alerts.iter().any(|a| matches!(a.severity, AlertSeverity::Error)) {
                "degraded"
            } else {
                "warning"
            };
            
            if format == "json" {
                let json_output = serde_json::json!({
                    "action": "health",
                    "status": health_status,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "metrics": metrics,
                    "alerts": alerts,
                    "alert_count": alerts.len()
                });
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                println!("\n=== System Health Check ===");
                println!("Overall Status: {}", health_status.to_uppercase());
                
                let status_icon = match health_status {
                    "healthy" => "✅",
                    "warning" => "⚠️ ",
                    "degraded" => "🟡",
                    "critical" => "🔴",
                    _ => "❓"
                };
                println!("{} {}", status_icon, health_status.to_uppercase());
                
                println!("\nSystem Information:");
                println!("  Uptime: {} seconds", metrics.uptime_seconds);
                println!("  Memory usage: {}MB", metrics.memory_usage_mb);
                println!("  CPU usage: {:.1}%", metrics.cpu_usage_percent);
                println!("  Active connections: {}", metrics.active_connections);
                
                println!("\nPerformance:");
                println!("  Total requests: {}", metrics.total_requests);
                println!("  Successful requests: {}", metrics.successful_requests);
                println!("  Failed requests: {}", metrics.failed_requests);
                println!("  Average response time: {:.2}ms", metrics.avg_response_time_ms);
                
                if !alerts.is_empty() {
                    println!("\nActive Alerts ({}):", alerts.len());
                    for alert in &alerts {
                        println!("  [{:?}] {:?}: {}", alert.severity, alert.alert_type, alert.message);
                    }
                } else {
                    println!("\n✅ No active alerts");
                }
            }
        },
        "status" => {
            let metrics = monitor.get_metrics();
            let request_history = monitor.get_request_history(Some(10));
            
            if format == "json" {
                let json_output = serde_json::json!({
                    "action": "status",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "metrics": metrics,
                    "recent_requests": request_history,
                    "system_status": "operational"
                });
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                println!("\n=== System Status ===");
                println!("System Status: ✅ Operational");
                println!("Timestamp: {}", chrono::Utc::now().to_rfc3339());
                
                println!("\nKey Metrics:");
                println!("  Uptime: {} seconds", metrics.uptime_seconds);
                println!("  Memory: {}MB | CPU: {:.1}%", metrics.memory_usage_mb, metrics.cpu_usage_percent);
                println!("  Requests: {} total | {} successful | {} failed", 
                    metrics.total_requests, metrics.successful_requests, metrics.failed_requests);
                println!("  Avg response time: {:.2}ms", metrics.avg_response_time_ms);
                
                println!("\nDatabase:");
                println!("  Triples: {} | Graphs: {}", metrics.database_metrics.total_triples, metrics.database_metrics.named_graphs);
                
                println!("\nReasoning:");
                println!("  Inferences: {} | Materialized: {}", 
                    metrics.reasoning_metrics.total_inferences, metrics.reasoning_metrics.materialized_triples);
                
                if !request_history.is_empty() {
                    println!("\nRecent Requests (last {}):", request_history.len().min(10));
                    for (i, req) in request_history.iter().take(10).enumerate() {
                        let status_icon = if req.success { "✅" } else { "❌" };
                        println!("  {}. {} {} {} - {}ms", i + 1, status_icon, req.method, req.endpoint, req.duration_ms);
                    }
                }
            }
        },
        _ => {
            return Err(EpcisKgError::Config(format!(
                "Unknown monitoring action: {}. Use 'metrics', 'alerts', 'health', or 'status'",
                action
            )));
        }
    }
    
    Ok(())
}

/// Load pre-generated sample data into the knowledge graph
fn load_sample_data(scale: &str, db_path: &str, force: bool) -> Result<usize, EpcisKgError> {
    info!("Loading sample data with scale '{}' into database at {}", scale, db_path);
    
    // Clear existing data if force is enabled
    if force {
        let path = std::path::Path::new(db_path);
        if path.exists() {
            info!("Clearing existing database at {}", db_path);
            std::fs::remove_dir_all(path)?;
        }
    }
    
    // Determine sample file path based on scale
    let sample_file = match scale.to_lowercase().as_str() {
        "small" => "samples/epcis_data_small.ttl",
        "medium" => "samples/epcis_data_medium.ttl",
        "large" => "samples/epcis_data_large.ttl",
        "xlarge" => "samples/epcis_data_xlarge.ttl",
        _ => "samples/epcis_data_medium.ttl",
    };
    
    // Check if sample file exists
    if !std::path::Path::new(sample_file).exists() {
        return Err(EpcisKgError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Sample file not found: {}. Run 'cargo run -- generate --scale {} --output-path samples/' first.", sample_file, scale),
        )));
    }
    
    // Load the sample data
    load_generated_data(sample_file, db_path)
}

/// Load generated data into the knowledge graph
fn load_generated_data(file_path: &str, db_path: &str) -> Result<usize, EpcisKgError> {
    info!("Loading data from {} into database at {}", file_path, db_path);
    
    // Check if file exists
    if !std::path::Path::new(file_path).exists() {
        return Err(EpcisKgError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", file_path),
        )));
    }
    
    // Initialize store
    let mut store = OxigraphStore::new(db_path)?;
    
    // Load the ontology data using the loader
    let loader = OntologyLoader::new();
    match loader.load_ontology(file_path) {
        Ok(ontology_data) => {
            store.store_ontology_data(&ontology_data)?;
            println!("✓ Successfully loaded {} triples from {}", ontology_data.triples_count, file_path);
            Ok(ontology_data.triples_count)
        },
        Err(e) => {
            Err(EpcisKgError::Storage(format!("Failed to load data from {}: {}", file_path, e)))
        }
    }
}
