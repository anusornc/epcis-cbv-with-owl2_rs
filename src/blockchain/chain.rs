use crate::blockchain::{Block, Transaction, BlockchainConfig};
use crate::blockchain::consensus::{Consensus, PoARoundRobin};
use crate::EpcisKgError;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use dashmap::DashMap;

/// The main blockchain structure
#[derive(Debug, Clone)]
pub struct Blockchain {
    pub config: BlockchainConfig,
    pub chain: Vec<Block>,
    pub pending_transactions: DashMap<String, Transaction>,
    pub transaction_pool: Vec<Transaction>,
    pub consensus: PoARoundRobin,
    pub state: BlockchainState,
}

/// Blockchain state (UTXO or account-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainState {
    /// Maps transaction ID to block height
    pub transaction_index: HashMap<String, u64>,
    /// Maps EPC to latest event
    pub epc_index: HashMap<String, String>,
    /// Total transactions processed
    pub total_transactions: u64,
}

impl Blockchain {
    /// Create a new blockchain with genesis block
    pub fn new(config: BlockchainConfig) -> Result<Self, EpcisKgError> {
        let genesis = Block::genesis(&config.genesis_data);
        let mut chain = vec![genesis];

        let consensus = PoARoundRobin::new(config.validators.clone())?;

        Ok(Self {
            config,
            chain,
            pending_transactions: DashMap::new(),
            transaction_pool: Vec::new(),
            consensus,
            state: BlockchainState {
                transaction_index: HashMap::new(),
                epc_index: HashMap::new(),
                total_transactions: 0,
            },
        })
    }

    /// Get the latest block
    pub fn latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    /// Get block by height
    pub fn get_block(&self, height: u64) -> Option<&Block> {
        self.chain.get(height as usize)
    }

    /// Get block by hash
    pub fn get_block_by_hash(&self, hash: &str) -> Option<&Block> {
        self.chain.iter().find(|block| block.hash() == hash)
    }

    /// Add transaction to pool
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), EpcisKgError> {
        // Verify transaction
        if !transaction.verify() {
            return Err(EpcisKgError::Validation("Invalid transaction".to_string()));
        }

        // Check for duplicates
        if self.state.transaction_index.contains_key(&transaction.tx_id) {
            return Err(EpcisKgError::Validation("Transaction already exists".to_string()));
        }

        // Add to pending pool
        self.pending_transactions.insert(transaction.tx_id.clone(), transaction.clone());
        self.transaction_pool.push(transaction);

        Ok(())
    }

    /// Create a new block with pending transactions
    pub fn create_block(&mut self, validator_address: String) -> Result<Block, EpcisKgError> {
        // Check if this validator's turn
        if !self.consensus.is_validator_turn(&validator_address, self.chain.len() as u64) {
            return Err(EpcisKgError::Validation(format!(
                "Not validator {}'s turn to create block",
                validator_address
            )));
        }

        // Get transactions up to max limit
        let max_tx = self.config.max_transactions_per_block;
        let transactions: Vec<Transaction> = self.transaction_pool
            .iter()
            .take(max_tx)
            .cloned()
            .collect();

        if transactions.is_empty() {
            return Err(EpcisKgError::Validation("No transactions to include in block".to_string()));
        }

        // Create new block
        let previous_hash = self.latest_block().hash();
        let height = self.chain.len() as u64;

        let block = Block::new(
            height,
            previous_hash,
            transactions.clone(),
            validator_address,
        );

        Ok(block)
    }

    /// Add a validated block to the chain
    pub fn add_block(&mut self, mut block: Block) -> Result<(), EpcisKgError> {
        // Verify block
        if !block.verify() {
            return Err(EpcisKgError::Validation("Invalid block".to_string()));
        }

        // Verify previous hash
        let expected_previous_hash = self.latest_block().hash();
        if block.header.previous_hash != expected_previous_hash {
            return Err(EpcisKgError::Validation("Invalid previous hash".to_string()));
        }

        // Verify height
        let expected_height = self.chain.len() as u64;
        if block.header.height != expected_height {
            return Err(EpcisKgError::Validation("Invalid block height".to_string()));
        }

        // Verify validator turn
        if !self.consensus.is_validator_turn(&block.header.validator_address, block.header.height) {
            return Err(EpcisKgError::Validation("Invalid validator for this block".to_string()));
        }

        // Update state
        for tx in &block.transactions {
            self.state.transaction_index.insert(tx.tx_id.clone(), block.header.height);
            self.state.total_transactions += 1;

            // Remove from pending
            self.pending_transactions.remove(&tx.tx_id);
            self.transaction_pool.retain(|t| t.tx_id != tx.tx_id);
        }

        // Add block to chain
        self.chain.push(block);

        Ok(())
    }

    /// Get blockchain statistics
    pub fn get_stats(&self) -> BlockchainStats {
        BlockchainStats {
            total_blocks: self.chain.len() as u64,
            total_transactions: self.state.total_transactions,
            pending_transactions: self.transaction_pool.len() as u64,
            validators: self.consensus.validators.len() as u64,
            current_validator: self.consensus.get_current_validator(self.chain.len() as u64),
        }
    }

    /// Verify entire blockchain integrity
    pub fn verify_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            // Verify block
            if !current.verify() {
                return false;
            }

            // Verify hash chain
            if current.header.previous_hash != previous.hash() {
                return false;
            }

            // Verify height
            if current.header.height != i as u64 {
                return false;
            }
        }

        true
    }

    /// Get transaction by ID
    pub fn get_transaction(&self, tx_id: &str) -> Option<(u64, &Transaction)> {
        if let Some(&height) = self.state.transaction_index.get(tx_id) {
            if let Some(block) = self.get_block(height) {
                for tx in &block.transactions {
                    if tx.tx_id == tx_id {
                        return Some((height, tx));
                    }
                }
            }
        }
        None
    }

    /// Get all transactions in a block
    pub fn get_block_transactions(&self, height: u64) -> Option<&Vec<Transaction>> {
        self.get_block(height).map(|block| &block.transactions)
    }

    /// Get chain height
    pub fn height(&self) -> u64 {
        self.chain.len() as u64 - 1
    }
}

/// Blockchain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub pending_transactions: u64,
    pub validators: u64,
    pub current_validator: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::TransactionType;

    fn create_test_config() -> BlockchainConfig {
        BlockchainConfig::default()
    }

    #[test]
    fn test_blockchain_creation() {
        let config = create_test_config();
        let blockchain = Blockchain::new(config).unwrap();

        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.latest_block().header.height, 0);
    }

    #[test]
    fn test_add_transaction() {
        let config = create_test_config();
        let mut blockchain = Blockchain::new(config).unwrap();

        let tx = Transaction::new(
            TransactionType::ObjectEvent,
            "farm1".to_string(),
            Some("plant1".to_string()),
            vec![1, 2, 3],
        );

        let result = blockchain.add_transaction(tx);
        assert!(result.is_ok());
        assert_eq!(blockchain.transaction_pool.len(), 1);
    }

    #[test]
    fn test_verify_chain() {
        let config = create_test_config();
        let blockchain = Blockchain::new(config).unwrap();

        assert!(blockchain.verify_chain());
    }

    #[test]
    fn test_blockchain_stats() {
        let config = create_test_config();
        let blockchain = Blockchain::new(config).unwrap();

        let stats = blockchain.get_stats();
        assert_eq!(stats.total_blocks, 1);
        assert_eq!(stats.total_transactions, 0);
    }
}
