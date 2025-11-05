/// Consensus module for blockchain
///
/// Implements Proof of Authority (PoA) with Round Robin selection

pub mod poa_round_robin;

pub use poa_round_robin::{PoARoundRobin, Validator, PoAConfig};

use crate::EpcisKgError;

/// Consensus trait for different consensus mechanisms
pub trait Consensus {
    /// Validate a block
    fn validate_block(&self, block: &crate::blockchain::Block) -> Result<(), EpcisKgError>;

    /// Get the validator for a given block height
    fn get_validator_for_height(&self, height: u64) -> String;

    /// Check if a validator is authorized
    fn is_authorized_validator(&self, address: &str) -> bool;
}
