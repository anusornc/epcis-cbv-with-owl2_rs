/// EPCIS Transaction wrapper for blockchain
///
/// Converts EPCIS events into blockchain transactions

use crate::models::epcis::EpcisEvent;
use crate::blockchain::{Transaction, TransactionType};
use crate::EpcisKgError;
use serde::{Serialize, Deserialize};

/// EPCIS Transaction - wraps EPCIS events for blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpcisTransaction {
    pub epcis_event: EpcisEvent,
    pub blockchain_tx: Transaction,
}

impl EpcisTransaction {
    /// Create a new EPCIS transaction from an EPCIS event
    pub fn from_event(event: EpcisEvent, sender: String) -> Result<Self, EpcisKgError> {
        // Determine transaction type based on event type
        let tx_type = match event.event_type.as_str() {
            "ObjectEvent" => TransactionType::ObjectEvent,
            "AggregationEvent" => TransactionType::AggregationEvent,
            "TransactionEvent" => TransactionType::TransactionEvent,
            "TransformationEvent" => TransactionType::TransformationEvent,
            _ => TransactionType::ObjectEvent,
        };

        // Serialize EPCIS event as payload
        let payload = serde_json::to_vec(&event)
            .map_err(|e| EpcisKgError::Json(e))?;

        // Extract receiver from biz_location if available
        let receiver = event.biz_location.clone();

        // Create blockchain transaction
        let blockchain_tx = Transaction::new(
            tx_type,
            sender,
            receiver,
            payload,
        );

        Ok(Self {
            epcis_event: event,
            blockchain_tx,
        })
    }

    /// Extract EPCIS event from a blockchain transaction
    pub fn from_transaction(tx: &Transaction) -> Result<EpcisEvent, EpcisKgError> {
        if !tx.is_epcis_event() {
            return Err(EpcisKgError::Validation("Not an EPCIS transaction".to_string()));
        }

        let event: EpcisEvent = serde_json::from_slice(&tx.payload)
            .map_err(|e| EpcisKgError::Json(e))?;

        Ok(event)
    }

    /// Get transaction ID
    pub fn id(&self) -> &str {
        &self.blockchain_tx.tx_id
    }

    /// Get EPCIS event ID
    pub fn event_id(&self) -> &str {
        &self.epcis_event.event_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event() -> EpcisEvent {
        EpcisEvent {
            event_id: "event_001".to_string(),
            event_type: "ObjectEvent".to_string(),
            event_time: "2024-01-01T00:00:00Z".to_string(),
            record_time: "2024-01-01T00:00:00Z".to_string(),
            event_action: "OBSERVE".to_string(),
            epc_list: vec!["urn:epc:id:sgtin:123456.789.100".to_string()],
            biz_step: Some("receiving".to_string()),
            disposition: Some("in_transit".to_string()),
            biz_location: Some("urn:epc:id:sgln:123456.789.0".to_string()),
        }
    }

    #[test]
    fn test_from_event() {
        let event = create_test_event();
        let epcis_tx = EpcisTransaction::from_event(event.clone(), "farm1".to_string());

        assert!(epcis_tx.is_ok());
        let tx = epcis_tx.unwrap();
        assert_eq!(tx.epcis_event.event_id, "event_001");
        assert_eq!(tx.blockchain_tx.sender, "farm1");
    }

    #[test]
    fn test_from_transaction() {
        let event = create_test_event();
        let epcis_tx = EpcisTransaction::from_event(event.clone(), "farm1".to_string()).unwrap();

        let extracted_event = EpcisTransaction::from_transaction(&epcis_tx.blockchain_tx);

        assert!(extracted_event.is_ok());
        let event = extracted_event.unwrap();
        assert_eq!(event.event_id, "event_001");
    }
}
