/// Cryptographic utilities for blockchain
///
/// Provides signing, verification, and key management

use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use sha2::{Sha256, Digest};
use crate::EpcisKgError;

/// Generate a new keypair for a validator
pub fn generate_keypair() -> Keypair {
    let mut csprng = rand::rngs::OsRng;
    Keypair::generate(&mut csprng)
}

/// Sign data with a private key
pub fn sign_data(data: &[u8], keypair: &Keypair) -> Vec<u8> {
    let signature = keypair.sign(data);
    signature.to_bytes().to_vec()
}

/// Verify a signature
pub fn verify_signature(data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool, EpcisKgError> {
    let public_key = PublicKey::from_bytes(public_key)
        .map_err(|e| EpcisKgError::Crypto(format!("Invalid public key: {}", e)))?;

    let signature = Signature::from_bytes(signature)
        .map_err(|e| EpcisKgError::Crypto(format!("Invalid signature: {}", e)))?;

    Ok(public_key.verify(data, &signature).is_ok())
}

/// Hash data using SHA-256
pub fn hash_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = generate_keypair();
        assert_eq!(keypair.public.as_bytes().len(), 32);
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = generate_keypair();
        let data = b"test data";

        let signature = sign_data(data, &keypair);
        let result = verify_signature(data, &signature, keypair.public.as_bytes());

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_hash_data() {
        let data = b"test";
        let hash1 = hash_data(data);
        let hash2 = hash_data(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex characters
    }
}
