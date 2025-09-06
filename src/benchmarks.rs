use std::time::{Duration, Instant};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use crate::storage::oxigraph_store::OxigraphStore;
use crate::Result;

/// Performance benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub duration_ms: f64,
    pub operations_per_second: f64,
    pub memory_usage_mb: Option<f64>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub data_scale: DataScale,
    pub include_memory_metrics: bool,
}

#[derive(Debug, Clone)]
pub enum DataScale {
    Small,
    Medium,
    Large,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 10,
            warmup_iterations: 3,
            data_scale: DataScale::Medium,
            include_memory_metrics: true,
        }
    }
}

/// Performance benchmarking suite
pub struct PerformanceBenchmarker {
    config: BenchmarkConfig,
    results: Vec<BenchmarkResult>,
}

impl PerformanceBenchmarker {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    pub fn run_all_benchmarks(&mut self, db_path: &Path) -> Result<()> {
        println!("🚀 Starting comprehensive performance benchmarks...");
        println!("📊 Configuration: {} iterations, {} warmup, {:?} data scale",
                 self.config.iterations, self.config.warmup_iterations, self.config.data_scale);

        // Warm up the system
        self.warmup_system(db_path)?;

        // Run benchmarks
        self.benchmark_data_loading(db_path)?;
        self.benchmark_sparql_queries(db_path)?;
        self.benchmark_reasoning_operations(db_path)?;
        self.benchmark_concurrent_operations(db_path)?;
        
        if self.config.include_memory_metrics {
            self.benchmark_memory_usage(db_path)?;
        }

        self.generate_report();
        Ok(())
    }

    fn warmup_system(&mut self, db_path: &Path) -> Result<()> {
        println!("🔥 Warming up system...");
        
        for i in 0..self.config.warmup_iterations {
            let store = OxigraphStore::new(db_path)?;
            
            // Simple warmup query
            let query = "SELECT * WHERE { ?s ?p ?o } LIMIT 10";
            let _ = store.query_select(query)?;
            
            println!("  Warmup iteration {}/{} completed", i + 1, self.config.warmup_iterations);
        }
        
        Ok(())
    }

    fn benchmark_data_loading(&mut self, db_path: &Path) -> Result<()> {
        println!("📦 Benchmarking data loading performance...");
        
        let data_file = match self.config.data_scale {
            DataScale::Small => "samples/epcis_data_small.ttl",
            DataScale::Medium => "samples/epcis_data_medium.ttl",
            DataScale::Large => "samples/epcis_data_large.ttl",
        };

        if !Path::new(data_file).exists() {
            return Err(crate::EpcisKgError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Sample data file not found: {}", data_file),
            )));
        }

        let file_size = fs::metadata(data_file)?.len();
        let data_content = fs::read_to_string(data_file)?;

        // Warmup iterations
        for _ in 0..self.config.warmup_iterations {
            let mut store = OxigraphStore::new(db_path)?;
            let _ = store.store_ontology_turtle(&data_content, "benchmark")?;
        }

        // Timed iterations
        let mut durations = Vec::new();
        let mut successes = 0;

        for i in 0..self.config.iterations {
            let start = Instant::now();
            
            let result = (|| {
                let mut store = OxigraphStore::new(db_path)?;
                store.store_ontology_turtle(&data_content, "benchmark")
            })();

            let duration = start.elapsed();
            durations.push(duration);

            match result {
                Ok(_) => {
                    successes += 1;
                    println!("  Load iteration {}/{}: {:.2}ms", 
                             i + 1, self.config.iterations, duration.as_millis());
                }
                Err(e) => {
                    println!("  Load iteration {}/{} failed: {}", i + 1, self.config.iterations, e);
                }
            }
        }

        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        let avg_ops_per_sec = (file_size as f64 * 1000.0) / (avg_duration.as_millis() as f64);
        let success_rate = (successes as f64 / self.config.iterations as f64) * 100.0;

        self.results.push(BenchmarkResult {
            test_name: format!("Data Loading ({:?})", self.config.data_scale),
            duration_ms: avg_duration.as_millis() as f64,
            operations_per_second: avg_ops_per_sec,
            memory_usage_mb: None,
            success: success_rate >= 90.0,
            error_message: if success_rate < 90.0 {
                Some(format!("Low success rate: {:.1}%", success_rate))
            } else {
                None
            },
        });

        Ok(())
    }

    fn benchmark_sparql_queries(&mut self, db_path: &Path) -> Result<()> {
        println!("🔍 Benchmarking SPARQL query performance...");
        
        let test_queries = vec![
            ("Simple SELECT", "SELECT * WHERE { ?s ?p ?o } LIMIT 100"),
            ("Count Query", "SELECT (COUNT(*) as ?count) WHERE { ?s ?p ?o }"),
            ("EPCIS Events", "SELECT * WHERE { ?event rdf:type epcis:ObjectEvent }"),
            ("Product Journey", 
             "SELECT ?product ?event WHERE { ?product ex:hasEvent ?event }"),
            ("Complex Join", 
             "SELECT DISTINCT ?manufacturer ?product ?location WHERE {
                 ?product ex:manufacturedBy ?manufacturer .
                 ?event epcis:epcList ?product .
                 ?event epcis:bizLocation ?location .
             }"),
            ("Aggregation", 
             "SELECT ?location (COUNT(?event) as ?eventCount) WHERE {
                 ?event epcis:bizLocation ?location .
             } GROUP BY ?location"),
        ];

        for (query_name, query) in test_queries {
            self.benchmark_single_query(db_path, query_name, query)?;
        }

        Ok(())
    }

    fn benchmark_single_query(&mut self, db_path: &Path, query_name: &str, query: &str) -> Result<()> {
        let store = OxigraphStore::new(db_path)?;

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = store.query_select(query);
        }

        // Timed iterations
        let mut durations = Vec::new();
        let mut successes = 0;

        for i in 0..self.config.iterations {
            let start = Instant::now();
            
            let result = store.query_select(query);
            let duration = start.elapsed();
            durations.push(duration);

            match result {
                Ok(_) => {
                    successes += 1;
                }
                Err(e) => {
                    println!("  {} query iteration {} failed: {}", query_name, i + 1, e);
                }
            }
        }

        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        let avg_ops_per_sec = 1000.0 / avg_duration.as_millis() as f64;
        let success_rate = (successes as f64 / self.config.iterations as f64) * 100.0;

        self.results.push(BenchmarkResult {
            test_name: format!("SPARQL Query: {}", query_name),
            duration_ms: avg_duration.as_millis() as f64,
            operations_per_second: avg_ops_per_sec,
            memory_usage_mb: None,
            success: success_rate >= 90.0,
            error_message: if success_rate < 90.0 {
                Some(format!("Low success rate: {:.1}%", success_rate))
            } else {
                None
            },
        });

        Ok(())
    }

    fn benchmark_reasoning_operations(&mut self, db_path: &Path) -> Result<()> {
        println!("🧠 Benchmarking reasoning operations...");
        
        // Test materialization performance
        let materialize_query = "SELECT * WHERE { ?s ?p ?o }";
        let store = OxigraphStore::new(db_path)?;

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = store.query_select(materialize_query);
        }

        // Timed iterations
        let mut durations = Vec::new();
        let mut successes = 0;

        for i in 0..self.config.iterations {
            let start = Instant::now();
            
            let result = store.query_select(materialize_query);
            let duration = start.elapsed();
            durations.push(duration);

            match result {
                Ok(_) => {
                    successes += 1;
                }
                Err(e) => {
                    println!("  Reasoning iteration {} failed: {}", i + 1, e);
                }
            }
        }

        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        let avg_ops_per_sec = 1000.0 / avg_duration.as_millis() as f64;
        let success_rate = (successes as f64 / self.config.iterations as f64) * 100.0;

        self.results.push(BenchmarkResult {
            test_name: "Reasoning Operations".to_string(),
            duration_ms: avg_duration.as_millis() as f64,
            operations_per_second: avg_ops_per_sec,
            memory_usage_mb: None,
            success: success_rate >= 90.0,
            error_message: if success_rate < 90.0 {
                Some(format!("Low success rate: {:.1}%", success_rate))
            } else {
                None
            },
        });

        Ok(())
    }

    fn benchmark_concurrent_operations(&mut self, db_path: &Path) -> Result<()> {
        println!("🔄 Benchmarking concurrent operations...");
        
        use std::thread;
        let num_threads = 4;
        let queries_per_thread = 25;

        // Warmup - create separate stores for each thread to avoid sharing issues
        for _ in 0..self.config.warmup_iterations {
            let mut handles = Vec::new();
            for _ in 0..num_threads {
                let db_path = db_path.to_path_buf();
                handles.push(thread::spawn(move || {
                    let store = OxigraphStore::new(&db_path).unwrap();
                    for _ in 0..5 {
                        let _ = store.query_select("SELECT * WHERE { ?s ?p ?o } LIMIT 50");
                    }
                }));
            }
            for handle in handles {
                handle.join().unwrap();
            }
        }

        // Timed test - each thread creates its own store instance
        let start = Instant::now();
        let mut handles = Vec::new();
        let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for _ in 0..num_threads {
            let db_path = db_path.to_path_buf();
            let success_clone = success_count.clone();
            handles.push(thread::spawn(move || {
                let store = OxigraphStore::new(&db_path).unwrap();
                for _ in 0..queries_per_thread {
                    if store.query_select("SELECT * WHERE { ?s ?p ?o } LIMIT 50").is_ok() {
                        success_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let duration = start.elapsed();
        let total_queries = num_threads * queries_per_thread;
        let successful_queries = success_count.load(std::sync::atomic::Ordering::Relaxed);
        let success_rate = (successful_queries as f64 / total_queries as f64) * 100.0;
        let queries_per_second = (successful_queries as f64 * 1000.0) / duration.as_millis() as f64;

        self.results.push(BenchmarkResult {
            test_name: "Concurrent Operations".to_string(),
            duration_ms: duration.as_millis() as f64,
            operations_per_second: queries_per_second,
            memory_usage_mb: None,
            success: success_rate >= 95.0,
            error_message: if success_rate < 95.0 {
                Some(format!("Low success rate: {:.1}%", success_rate))
            } else {
                None
            },
        });

        Ok(())
    }

    fn benchmark_memory_usage(&mut self, db_path: &Path) -> Result<()> {
        println!("💾 Benchmarking memory usage...");
        
        // Get initial memory usage
        let initial_memory = self.get_memory_usage();
        
        // Load data and measure memory
        let data_file = match self.config.data_scale {
            DataScale::Small => "samples/epcis_data_small.ttl",
            DataScale::Medium => "samples/epcis_data_medium.ttl",
            DataScale::Large => "samples/epcis_data_large.ttl",
        };

        if Path::new(data_file).exists() {
            let data_content = fs::read_to_string(data_file)?;
            let mut store = OxigraphStore::new(db_path)?;
            let _ = store.store_ontology_turtle(&data_content, "benchmark")?;
            
            // Run some queries to ensure data is processed
            for _ in 0..5 {
                let _ = store.query_select("SELECT * WHERE { ?s ?p ?o } LIMIT 100");
            }
            
            let peak_memory = self.get_memory_usage();
            let memory_increase = peak_memory - initial_memory;
            
            self.results.push(BenchmarkResult {
                test_name: format!("Memory Usage ({:?})", self.config.data_scale),
                duration_ms: 0.0,
                operations_per_second: 0.0,
                memory_usage_mb: Some(memory_increase),
                success: memory_increase < 1000.0, // Less than 1GB increase
                error_message: if memory_increase >= 1000.0 {
                    Some(format!("High memory usage: {:.1}MB", memory_increase))
                } else {
                    None
                },
            });
        }

        Ok(())
    }

    fn get_memory_usage(&self) -> f64 {
        // Simple memory usage approximation
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("ps").args(&["-p", &std::process::id().to_string(), "-o", "rss"]).output() {
                if let Ok(rss_str) = String::from_utf8(output.stdout) {
                    if let Some(rss_bytes) = rss_str.lines().nth(1).and_then(|s| s.trim().parse::<u64>().ok()) {
                        return (rss_bytes as f64) / 1024.0 / 1024.0; // Convert to MB
                    }
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("ps").args(&["-p", &std::process::id().to_string(), "-o", "rss"]).output() {
                if let Ok(rss_str) = String::from_utf8(output.stdout) {
                    if let Some(rss_kb) = rss_str.lines().nth(1).and_then(|s| s.trim().parse::<u64>().ok()) {
                        return (rss_kb as f64) / 1024.0; // Convert to MB
                    }
                }
            }
        }
        
        0.0 // Fallback
    }

    fn generate_report(&self) {
        println!("\n📊 Performance Benchmark Report");
        println!("====================================");
        
        let mut total_passed = 0;
        let mut total_failed = 0;
        
        for result in &self.results {
            let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
            println!("{}: {:.2}ms, {:.1} ops/sec", 
                     status, result.duration_ms, result.operations_per_second);
            
            if let Some(memory) = result.memory_usage_mb {
                println!("  Memory: {:.1}MB", memory);
            }
            
            if let Some(ref error) = result.error_message {
                println!("  Error: {}", error);
            }
            
            if result.success {
                total_passed += 1;
            } else {
                total_failed += 1;
            }
        }
        
        println!("\n📈 Summary:");
        println!("  Passed: {}/{}", total_passed, self.results.len());
        println!("  Failed: {}/{}", total_failed, self.results.len());
        
        if total_failed == 0 {
            println!("🎉 All benchmarks passed!");
        } else {
            println!("⚠️  {} benchmarks failed", total_failed);
        }
        
        // Performance targets
        println!("\n🎯 Performance Targets:");
        println!("  Data loading: < 500ms (small), < 2000ms (medium), < 10000ms (large)");
        println!("  SPARQL queries: < 100ms (simple), < 1000ms (complex)");
        println!("  Concurrent operations: > 100 queries/sec");
        println!("  Memory usage: < 1GB for large datasets");
    }
}

/// Run performance benchmarks with default configuration
pub fn run_performance_benchmarks(db_path: &Path) -> Result<()> {
    let config = BenchmarkConfig::default();
    let mut benchmarker = PerformanceBenchmarker::new(config);
    benchmarker.run_all_benchmarks(db_path)
}

/// Run performance benchmarks with custom configuration
pub fn run_custom_benchmarks(db_path: &Path, iterations: usize, data_scale: DataScale) -> Result<()> {
    let config = BenchmarkConfig {
        iterations,
        data_scale,
        ..Default::default()
    };
    let mut benchmarker = PerformanceBenchmarker::new(config);
    benchmarker.run_all_benchmarks(db_path)
}