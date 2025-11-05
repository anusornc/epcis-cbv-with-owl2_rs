/// Blockchain module for EPCIS supply chain traceability
///
/// This module implements a private blockchain with:
/// - Proof of Authority (PoA) Round Robin consensus
/// - EPCIS event transactions
/// - Knowledge graph integration
/// - Cryptographic verification

pub mod block;
pub mod transaction;
pub mod chain;
pub mod consensus;
pub mod crypto;
pub mod epcis_tx;
pub mod graph_builder;
pub mod network;

pub use block::{Block, BlockHeader};
pub use transaction::{Transaction, TransactionType};
pub use chain::Blockchain;
pub use consensus::{Consensus, PoAConfig, Validator};
pub use epcis_tx::EpcisTransaction;

use serde::{Serialize, Deserialize};

/// Blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    /// Block time in seconds
    pub block_time_seconds: u64,

    /// Maximum transactions per block
    pub max_transactions_per_block: usize,

    /// Genesis block data
    pub genesis_data: String,

    /// Validators for PoA consensus
    pub validators: Vec<ValidatorConfig>,

    /// Network configuration
    pub network: NetworkConfig,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            block_time_seconds: 15,
            max_transactions_per_block: 100,
            genesis_data: "EPCIS Blockchain Genesis Block - UHT Supply Chain".to_string(),
            validators: vec![
                ValidatorConfig {
                    name: "Dairy Farm".to_string(),
                    address: "validator1".to_string(),
                    public_key: vec![],
                },
                ValidatorConfig {
                    name: "Processing Plant".to_string(),
                    address: "validator2".to_string(),
                    public_key: vec![],
                },
                ValidatorConfig {
                    name: "Distribution Center".to_string(),
                    address: "validator3".to_string(),
                    public_key: vec![],
                },
                ValidatorConfig {
                    name: "Retailer".to_string(),
                    address: "validator4".to_string(),
                    public_key: vec![],
                },
            ],
            network: NetworkConfig::default(),
        }
    }
}

/// Validator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    pub name: String,
    pub address: String,
    pub public_key: Vec<u8>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_addr: String,
    pub bootstrap_peers: Vec<String>,
    pub enable_mdns: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/0".to_string(),
            bootstrap_peers: Vec::new(),
            enable_mdns: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_config_default() {
        let config = BlockchainConfig::default();
        assert_eq!(config.block_time_seconds, 15);
        assert_eq!(config.max_transactions_per_block, 100);
        assert_eq!(config.validators.len(), 4);
    }
}
