use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

/// Transaction types supported by the blockchain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    /// EPCIS ObjectEvent (product creation, observation)
    ObjectEvent,
    /// EPCIS AggregationEvent (packaging, palletization)
    AggregationEvent,
    /// EPCIS TransactionEvent (ownership transfer)
    TransactionEvent,
    /// EPCIS TransformationEvent (UHT processing, manufacturing)
    TransformationEvent,
    /// System transaction (validator registration, etc.)
    System,
}

/// Transaction on the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub tx_id: String,
    pub tx_type: TransactionType,
    pub timestamp: u64,
    pub sender: String,
    pub receiver: Option<String>,
    pub payload: Vec<u8>,  // EPCIS event data (serialized)
    pub nonce: u64,
    pub signature: Option<Vec<u8>>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        tx_type: TransactionType,
        sender: String,
        receiver: Option<String>,
        payload: Vec<u8>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let nonce = timestamp; // Simplified nonce

        let mut tx = Self {
            tx_id: String::new(),
            tx_type,
            timestamp,
            sender,
            receiver,
            payload,
            nonce,
            signature: None,
        };

        tx.tx_id = tx.calculate_id();
        tx
    }

    /// Calculate transaction ID (hash)
    fn calculate_id(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", self.tx_type).as_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(self.sender.as_bytes());
        if let Some(ref receiver) = self.receiver {
            hasher.update(receiver.as_bytes());
        }
        hasher.update(&self.payload);
        hasher.update(&self.nonce.to_le_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Get transaction hash
    pub fn hash(&self) -> String {
        self.tx_id.clone()
    }

    /// Sign the transaction
    pub fn sign(&mut self, signature: Vec<u8>) {
        self.signature = Some(signature);
    }

    /// Verify transaction integrity
    pub fn verify(&self) -> bool {
        // Verify tx_id matches calculated hash
        let calculated_id = self.calculate_id();
        if calculated_id != self.tx_id {
            return false;
        }

        // In a full implementation, verify signature here
        true
    }

    /// Get transaction size in bytes
    pub fn size(&self) -> usize {
        bincode::serialize(self).unwrap().len()
    }

    /// Check if transaction is an EPCIS event
    pub fn is_epcis_event(&self) -> bool {
        matches!(
            self.tx_type,
            TransactionType::ObjectEvent
                | TransactionType::AggregationEvent
                | TransactionType::TransactionEvent
                | TransactionType::TransformationEvent
        )
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self::new(
            TransactionType::System,
            String::from("system"),
            None,
            Vec::new(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(
            TransactionType::ObjectEvent,
            "farm1".to_string(),
            Some("plant1".to_string()),
            vec![1, 2, 3, 4],
        );

        assert!(!tx.tx_id.is_empty());
        assert_eq!(tx.sender, "farm1");
        assert_eq!(tx.receiver, Some("plant1".to_string()));
    }

    #[test]
    fn test_transaction_hash() {
        let tx1 = Transaction::new(
            TransactionType::ObjectEvent,
            "farm1".to_string(),
            None,
            vec![1, 2, 3],
        );

        let hash1 = tx1.hash();
        let hash2 = tx1.hash();

        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }

    #[test]
    fn test_transaction_verify() {
        let tx = Transaction::new(
            TransactionType::ObjectEvent,
            "farm1".to_string(),
            None,
            vec![1, 2, 3],
        );

        assert!(tx.verify());
    }

    #[test]
    fn test_is_epcis_event() {
        let epcis_tx = Transaction::new(
            TransactionType::ObjectEvent,
            "farm1".to_string(),
            None,
            vec![],
        );
        assert!(epcis_tx.is_epcis_event());

        let system_tx = Transaction::new(
            TransactionType::System,
            "system".to_string(),
            None,
            vec![],
        );
        assert!(!system_tx.is_epcis_event());
    }
}
