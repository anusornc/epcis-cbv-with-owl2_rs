pub mod generator;
pub mod entities;
pub mod events;
pub mod utils;

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Configuration for data generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    pub scale: DataScale,
    pub output_format: OutputFormat,
    pub output_path: PathBuf,
    pub custom_counts: Option<(usize, usize, usize)>,
}

/// Data scale options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataScale {
    Small,      // ~1K triples
    Medium,     // ~10K triples
    Large,      // ~100K triples
    XLarge,     // ~1M triples
    Custom(usize),
}

impl DataScale {
    pub fn triple_count(&self) -> usize {
        match self {
            DataScale::Small => 1_000,
            DataScale::Medium => 10_000,
            DataScale::Large => 100_000,
            DataScale::XLarge => 1_000_000,
            DataScale::Custom(count) => *count,
        }
    }
}

/// Output format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Turtle,
    NTriples,
    JsonLd,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Turtle => write!(f, "turtle"),
            OutputFormat::NTriples => write!(f, "n-triples"),
            OutputFormat::JsonLd => write!(f, "json-ld"),
        }
    }
}

/// Default configuration
impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            scale: DataScale::Medium,
            output_format: OutputFormat::Turtle,
            output_path: PathBuf::from("data/generated"),
            custom_counts: None,
        }
    }
}

/// Data generation result
#[derive(Debug, Clone)]
pub struct GenerationResult {
    pub triple_count: usize,
    pub event_count: usize,
    pub location_count: usize,
    pub product_count: usize,
    pub generation_time_ms: u64,
    pub output_files: Vec<String>,
}

/// Trait for data generators
pub trait DataGenerator {
    fn generate(&self, config: &GeneratorConfig) -> Result<GenerationResult, Box<dyn std::error::Error + Send + Sync>>;
    fn validate_config(&self, config: &GeneratorConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}