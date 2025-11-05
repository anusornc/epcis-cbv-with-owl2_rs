use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::blockchain::transaction::Transaction;

/// Block in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub validator_signature: Option<Vec<u8>>,
}

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub timestamp: u64,
    pub previous_hash: String,
    pub transactions_root: String,
    pub state_root: String,
    pub validator_address: String,
    pub nonce: u64,
}

impl Block {
    /// Create a new block
    pub fn new(
        height: u64,
        previous_hash: String,
        transactions: Vec<Transaction>,
        validator_address: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let transactions_root = Self::calculate_transactions_root(&transactions);
        let state_root = String::from("0"); // Will be calculated from state changes

        let header = BlockHeader {
            height,
            timestamp,
            previous_hash,
            transactions_root,
            state_root,
            validator_address,
            nonce: 0,
        };

        Self {
            header,
            transactions,
            validator_signature: None,
        }
    }

    /// Create genesis block
    pub fn genesis(genesis_data: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let header = BlockHeader {
            height: 0,
            timestamp,
            previous_hash: String::from("0"),
            transactions_root: String::from("0"),
            state_root: String::from("0"),
            validator_address: String::from("genesis"),
            nonce: 0,
        };

        Self {
            header,
            transactions: Vec::new(),
            validator_signature: None,
        }
    }

    /// Calculate block hash
    pub fn hash(&self) -> String {
        let header_bytes = bincode::serialize(&self.header).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&header_bytes);
        format!("{:x}", hasher.finalize())
    }

    /// Calculate Merkle root of transactions
    fn calculate_transactions_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return String::from("0");
        }

        let tx_hashes: Vec<String> = transactions
            .iter()
            .map(|tx| tx.hash())
            .collect();

        Self::merkle_root(tx_hashes)
    }

    /// Calculate Merkle root from hashes
    fn merkle_root(mut hashes: Vec<String>) -> String {
        if hashes.is_empty() {
            return String::from("0");
        }

        while hashes.len() > 1 {
            let mut new_hashes = Vec::new();

            for i in (0..hashes.len()).step_by(2) {
                let left = &hashes[i];
                let right = if i + 1 < hashes.len() {
                    &hashes[i + 1]
                } else {
                    &hashes[i]
                };

                let combined = format!("{}{}", left, right);
                let mut hasher = Sha256::new();
                hasher.update(combined.as_bytes());
                new_hashes.push(format!("{:x}", hasher.finalize()));
            }

            hashes = new_hashes;
        }

        hashes[0].clone()
    }

    /// Verify block integrity
    pub fn verify(&self) -> bool {
        // Verify transactions root
        let calculated_root = Self::calculate_transactions_root(&self.transactions);
        if calculated_root != self.header.transactions_root {
            return false;
        }

        // Verify all transactions
        for tx in &self.transactions {
            if !tx.verify() {
                return false;
            }
        }

        true
    }

    /// Sign the block with validator's private key
    pub fn sign(&mut self, signature: Vec<u8>) {
        self.validator_signature = Some(signature);
    }

    /// Get block size in bytes
    pub fn size(&self) -> usize {
        bincode::serialize(self).unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::transaction::{Transaction, TransactionType};

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis("test genesis");
        assert_eq!(genesis.header.height, 0);
        assert_eq!(genesis.header.previous_hash, "0");
        assert_eq!(genesis.transactions.len(), 0);
    }

    #[test]
    fn test_block_hash() {
        let block = Block::genesis("test");
        let hash1 = block.hash();
        let hash2 = block.hash();
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }

    #[test]
    fn test_merkle_root() {
        let hashes = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let root = Block::merkle_root(hashes);
        assert!(!root.is_empty());
        assert_ne!(root, "0");
    }

    #[test]
    fn test_block_verification() {
        let block = Block::genesis("test");
        assert!(block.verify());
    }
}
