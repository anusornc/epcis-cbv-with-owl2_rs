use crate::blockchain::{Block, ValidatorConfig};
use crate::blockchain::consensus::Consensus;
use crate::EpcisKgError;
use serde::{Serialize, Deserialize};

/// Proof of Authority Round Robin consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoAConfig {
    /// List of authorized validators
    pub validators: Vec<Validator>,
    /// Block time in seconds
    pub block_time: u64,
}

/// Validator in the PoA network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Validator {
    pub address: String,
    pub name: String,
    pub public_key: Vec<u8>,
    pub is_active: bool,
}

/// PoA Round Robin consensus mechanism
#[derive(Debug, Clone)]
pub struct PoARoundRobin {
    pub validators: Vec<Validator>,
    pub block_time: u64,
    pub current_round: u64,
}

impl PoARoundRobin {
    /// Create a new PoA Round Robin consensus
    pub fn new(validator_configs: Vec<ValidatorConfig>) -> Result<Self, EpcisKgError> {
        if validator_configs.is_empty() {
            return Err(EpcisKgError::Config("At least one validator required".to_string()));
        }

        let validators: Vec<Validator> = validator_configs
            .into_iter()
            .map(|config| Validator {
                address: config.address,
                name: config.name,
                public_key: config.public_key,
                is_active: true,
            })
            .collect();

        Ok(Self {
            validators,
            block_time: 15, // Default 15 seconds
            current_round: 0,
        })
    }

    /// Get the current validator based on round-robin
    pub fn get_current_validator(&self, block_height: u64) -> String {
        let index = (block_height as usize) % self.validators.len();
        self.validators[index].address.clone()
    }

    /// Check if it's a specific validator's turn
    pub fn is_validator_turn(&self, address: &str, block_height: u64) -> bool {
        let current_validator = self.get_current_validator(block_height);
        current_validator == address
    }

    /// Get validator by address
    pub fn get_validator(&self, address: &str) -> Option<&Validator> {
        self.validators.iter().find(|v| v.address == address)
    }

    /// Get all active validators
    pub fn get_active_validators(&self) -> Vec<&Validator> {
        self.validators.iter().filter(|v| v.is_active).collect()
    }

    /// Add a new validator (requires consensus)
    pub fn add_validator(&mut self, validator: Validator) -> Result<(), EpcisKgError> {
        // Check if validator already exists
        if self.validators.iter().any(|v| v.address == validator.address) {
            return Err(EpcisKgError::Validation("Validator already exists".to_string()));
        }

        self.validators.push(validator);
        Ok(())
    }

    /// Remove a validator (requires consensus)
    pub fn remove_validator(&mut self, address: &str) -> Result<(), EpcisKgError> {
        let initial_len = self.validators.len();

        if initial_len <= 1 {
            return Err(EpcisKgError::Validation("Cannot remove the last validator".to_string()));
        }

        self.validators.retain(|v| v.address != address);

        if self.validators.len() == initial_len {
            return Err(EpcisKgError::Validation("Validator not found".to_string()));
        }

        Ok(())
    }

    /// Deactivate a validator
    pub fn deactivate_validator(&mut self, address: &str) -> Result<(), EpcisKgError> {
        let validator = self.validators.iter_mut()
            .find(|v| v.address == address)
            .ok_or_else(|| EpcisKgError::Validation("Validator not found".to_string()))?;

        validator.is_active = false;
        Ok(())
    }

    /// Activate a validator
    pub fn activate_validator(&mut self, address: &str) -> Result<(), EpcisKgError> {
        let validator = self.validators.iter_mut()
            .find(|v| v.address == address)
            .ok_or_else(|| EpcisKgError::Validation("Validator not found".to_string()))?;

        validator.is_active = true;
        Ok(())
    }

    /// Get validator count
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }

    /// Get active validator count
    pub fn active_validator_count(&self) -> usize {
        self.validators.iter().filter(|v| v.is_active).count()
    }
}

impl Consensus for PoARoundRobin {
    fn validate_block(&self, block: &Block) -> Result<(), EpcisKgError> {
        // Check if validator is authorized
        if !self.is_authorized_validator(&block.header.validator_address) {
            return Err(EpcisKgError::Validation("Unauthorized validator".to_string()));
        }

        // Check if it's the validator's turn
        if !self.is_validator_turn(&block.header.validator_address, block.header.height) {
            return Err(EpcisKgError::Validation("Not validator's turn".to_string()));
        }

        // Verify block integrity
        if !block.verify() {
            return Err(EpcisKgError::Validation("Block verification failed".to_string()));
        }

        Ok(())
    }

    fn get_validator_for_height(&self, height: u64) -> String {
        self.get_current_validator(height)
    }

    fn is_authorized_validator(&self, address: &str) -> bool {
        self.validators.iter().any(|v| v.address == address && v.is_active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::ValidatorConfig;

    fn create_test_validators() -> Vec<ValidatorConfig> {
        vec![
            ValidatorConfig {
                name: "Validator 1".to_string(),
                address: "addr1".to_string(),
                public_key: vec![1, 2, 3],
            },
            ValidatorConfig {
                name: "Validator 2".to_string(),
                address: "addr2".to_string(),
                public_key: vec![4, 5, 6],
            },
            ValidatorConfig {
                name: "Validator 3".to_string(),
                address: "addr3".to_string(),
                public_key: vec![7, 8, 9],
            },
        ]
    }

    #[test]
    fn test_poa_creation() {
        let validators = create_test_validators();
        let poa = PoARoundRobin::new(validators).unwrap();

        assert_eq!(poa.validator_count(), 3);
        assert_eq!(poa.active_validator_count(), 3);
    }

    #[test]
    fn test_round_robin_rotation() {
        let validators = create_test_validators();
        let poa = PoARoundRobin::new(validators).unwrap();

        // Block 0 -> Validator 1
        assert_eq!(poa.get_current_validator(0), "addr1");

        // Block 1 -> Validator 2
        assert_eq!(poa.get_current_validator(1), "addr2");

        // Block 2 -> Validator 3
        assert_eq!(poa.get_current_validator(2), "addr3");

        // Block 3 -> Back to Validator 1
        assert_eq!(poa.get_current_validator(3), "addr1");
    }

    #[test]
    fn test_validator_turn() {
        let validators = create_test_validators();
        let poa = PoARoundRobin::new(validators).unwrap();

        assert!(poa.is_validator_turn("addr1", 0));
        assert!(!poa.is_validator_turn("addr2", 0));

        assert!(poa.is_validator_turn("addr2", 1));
        assert!(!poa.is_validator_turn("addr1", 1));
    }

    #[test]
    fn test_is_authorized_validator() {
        let validators = create_test_validators();
        let poa = PoARoundRobin::new(validators).unwrap();

        assert!(poa.is_authorized_validator("addr1"));
        assert!(poa.is_authorized_validator("addr2"));
        assert!(!poa.is_authorized_validator("unknown"));
    }

    #[test]
    fn test_add_validator() {
        let validators = create_test_validators();
        let mut poa = PoARoundRobin::new(validators).unwrap();

        let new_validator = Validator {
            address: "addr4".to_string(),
            name: "Validator 4".to_string(),
            public_key: vec![10, 11, 12],
            is_active: true,
        };

        assert!(poa.add_validator(new_validator).is_ok());
        assert_eq!(poa.validator_count(), 4);
    }

    #[test]
    fn test_remove_validator() {
        let validators = create_test_validators();
        let mut poa = PoARoundRobin::new(validators).unwrap();

        assert!(poa.remove_validator("addr2").is_ok());
        assert_eq!(poa.validator_count(), 2);
        assert!(!poa.is_authorized_validator("addr2"));
    }

    #[test]
    fn test_cannot_remove_last_validator() {
        let validators = vec![
            ValidatorConfig {
                name: "Only Validator".to_string(),
                address: "addr1".to_string(),
                public_key: vec![1, 2, 3],
            },
        ];
        let mut poa = PoARoundRobin::new(validators).unwrap();

        assert!(poa.remove_validator("addr1").is_err());
    }

    #[test]
    fn test_deactivate_validator() {
        let validators = create_test_validators();
        let mut poa = PoARoundRobin::new(validators).unwrap();

        assert!(poa.deactivate_validator("addr2").is_ok());
        assert!(!poa.is_authorized_validator("addr2"));
        assert_eq!(poa.active_validator_count(), 2);
    }
}
