use clap::{Parser, Subcommand};
use epcis_knowledge_graph::EpcisKgError;
use epcis_knowledge_graph::ontology::loader::OntologyLoader;
use epcis_knowledge_graph::storage::oxigraph_store::OxigraphStore;
use epcis_knowledge_graph::ontology::reasoner::OntologyReasoner;
use tracing::{info, Level};

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

    /// Initialize the knowledge graph
    Init {
        /// Database path
        #[arg(short, long, default_value = "./data")]
        db_path: String,

        /// Force initialization (overwrite existing data)
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), EpcisKgError> {
    let args = Args::parse();

    // Initialize logging
    let level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt().with_max_level(level).init();

    info!("Starting EPCIS Knowledge Graph");

    match args.command {
        Commands::Serve { port, db_path } => {
            info!(
                "Starting server on port {} with database at {}",
                port, db_path
            );
            // TODO: Implement server startup
            println!("Server functionality not yet implemented");
        }
        Commands::Load { files, db_path } => {
            info!(
                "Loading ontologies from {:?} into database at {}",
                files, db_path
            );
            load_ontologies(&files, &db_path)?;
        }
        Commands::Query {
            query,
            db_path,
            format,
        } => {
            info!("Executing query against database at {}", db_path);
            execute_query(&query, &db_path, &format)?;
        }
        Commands::Validate {
            event_file,
            db_path,
        } => {
            info!(
                "Validating EPCIS events from {} against database at {}",
                event_file, db_path
            );
            // TODO: Implement event validation
            println!("Event validation not yet implemented");
        }
        Commands::Reason { db_path, profile, inference } => {
            info!(
                "Performing reasoning on knowledge graph at {} (profile: {}, inference: {})",
                db_path, profile, inference
            );
            perform_reasoning(&db_path, &profile, inference)?;
        }
        Commands::Init { db_path, force } => {
            info!(
                "Initializing knowledge graph at {} (force: {})",
                db_path, force
            );
            initialize_knowledge_graph(&db_path, force)?;
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
    let reasoner = OntologyReasoner::with_store(store);
    
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
fn initialize_knowledge_graph(db_path: &str, force: bool) -> Result<(), EpcisKgError> {
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
    let default_ontologies = vec![
        "ontologies/epcis2.ttl",
        "ontologies/cbv.ttl",
    ];
    
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
