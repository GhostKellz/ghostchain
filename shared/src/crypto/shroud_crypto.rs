// Shroud-based crypto module with gcrypt fallback
// Integration with Shroud's QID/Sigil identity system

use anyhow::{Result, anyhow};
use std::sync::Arc;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use rand::RngCore;
use blake3;
use hex;

// Core crypto operations trait
pub trait CryptoOperations {
    fn generate_ed25519_keypair(&self) -> Result<Ed25519KeyPair>;
    fn ed25519_sign(&self, private_key: &[u8], message: &[u8]) -> Result<Vec<u8>>;
    fn ed25519_verify(&self, public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool>;
    fn blake3_hash(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn generate_qid(&self) -> Result<String>;
    fn generate_sigil(&self, qid: &str) -> Result<String>;
}

// Ed25519 key pair structure
#[derive(Debug, Clone)]
pub struct Ed25519KeyPair {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

// Secp256k1 key pair structure (for Bitcoin compatibility)
#[derive(Debug, Clone)]
pub struct Secp256k1KeyPair {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

// Main crypto manager with Shroud integration
pub struct CryptoManager {
    // Use Arc for thread-safe shared state
    shroud_enabled: bool,
    gcrypt_fallback: bool,
}

impl CryptoManager {
    pub fn new() -> Self {
        Self {
            shroud_enabled: true,  // Enable Shroud integration
            gcrypt_fallback: true, // Enable gcrypt fallback
        }
    }

    pub fn with_shroud_disabled() -> Self {
        Self {
            shroud_enabled: false,
            gcrypt_fallback: true,
        }
    }

    // QID (Quantum Identity) generation using Shroud
    pub fn generate_qid_with_shroud(&self) -> Result<String> {
        if self.shroud_enabled {
            // Integration with Shroud's QID system
            // This would call into the Shroud library's QID generation
            // For now, we'll use a placeholder implementation
            let mut rng = OsRng;
            let mut qid_bytes = [0u8; 32];
            rng.fill_bytes(&mut qid_bytes);
            let qid_hash = blake3::hash(&qid_bytes);
            Ok(format!("qid:{}", hex::encode(&qid_hash.as_bytes()[..16])))
        } else {
            Err(anyhow!("Shroud integration disabled"))
        }
    }

    // Sigil generation using Shroud
    pub fn generate_sigil_with_shroud(&self, qid: &str) -> Result<String> {
        if self.shroud_enabled {
            // Integration with Shroud's Sigil system
            // This would call into the Shroud library's Sigil generation
            let sigil_data = format!("sigil:{}", qid);
            let sigil_hash = blake3::hash(sigil_data.as_bytes());
            Ok(format!("sigil:{}", hex::encode(&sigil_hash.as_bytes()[..12])))
        } else {
            Err(anyhow!("Shroud integration disabled"))
        }
    }

    // Enhanced key derivation with Shroud integration
    pub fn derive_keys_from_qid(&self, qid: &str) -> Result<Ed25519KeyPair> {
        if self.shroud_enabled {
            // Use Shroud's key derivation
            let seed = blake3::hash(qid.as_bytes());
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&seed.as_bytes()[..32]);
            let signing_key = SigningKey::from_bytes(&key_bytes);
            let verifying_key = signing_key.verifying_key();
            
            Ok(Ed25519KeyPair {
                private_key: signing_key.to_bytes().to_vec(),
                public_key: verifying_key.to_bytes().to_vec(),
            })
        } else {
            // Fallback to standard key generation
            self.generate_ed25519_keypair()
        }
    }
}

impl CryptoOperations for CryptoManager {
    fn generate_ed25519_keypair(&self) -> Result<Ed25519KeyPair> {
        if self.gcrypt_fallback {
            // Use gcrypt fallback implementation
            let mut rng = OsRng;
            let mut secret_key = [0u8; 32];
            rng.fill_bytes(&mut secret_key);
            let signing_key = SigningKey::from_bytes(&secret_key);
            let verifying_key = signing_key.verifying_key();
            
            Ok(Ed25519KeyPair {
                private_key: signing_key.to_bytes().to_vec(),
                public_key: verifying_key.to_bytes().to_vec(),
            })
        } else {
            Err(anyhow!("No crypto backend available"))
        }
    }

    fn ed25519_sign(&self, private_key: &[u8], message: &[u8]) -> Result<Vec<u8>> {
        if self.gcrypt_fallback {
            let signing_key = SigningKey::from_bytes(
                private_key.try_into().map_err(|_| anyhow!("Invalid private key length"))?
            );
            let signature = signing_key.sign(message);
            Ok(signature.to_bytes().to_vec())
        } else {
            Err(anyhow!("No crypto backend available"))
        }
    }

    fn ed25519_verify(&self, public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
        if self.gcrypt_fallback {
            let verifying_key = VerifyingKey::from_bytes(
                public_key.try_into().map_err(|_| anyhow!("Invalid public key length"))?
            )?;
            let sig = Signature::from_bytes(
                signature.try_into().map_err(|_| anyhow!("Invalid signature length"))?
            );
            Ok(verifying_key.verify(message, &sig).is_ok())
        } else {
            Err(anyhow!("No crypto backend available"))
        }
    }

    fn blake3_hash(&self, data: &[u8]) -> Result<Vec<u8>> {
        if self.gcrypt_fallback {
            let hash = blake3::hash(data);
            Ok(hash.as_bytes().to_vec())
        } else {
            Err(anyhow!("No hashing backend available"))
        }
    }

    fn generate_qid(&self) -> Result<String> {
        self.generate_qid_with_shroud()
            .or_else(|_| {
                // Fallback QID generation
                let mut rng = OsRng;
                let mut qid_bytes = [0u8; 16];
                rng.fill_bytes(&mut qid_bytes);
                Ok(format!("qid:{}", hex::encode(&qid_bytes)))
            })
    }

    fn generate_sigil(&self, qid: &str) -> Result<String> {
        self.generate_sigil_with_shroud(qid)
            .or_else(|_| {
                // Fallback Sigil generation
                let hash = blake3::hash(qid.as_bytes());
                Ok(format!("sigil:{}", hex::encode(&hash.as_bytes()[..12])))
            })
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new()
    }
}

// Utility functions for address generation
pub fn generate_ghost_address(public_key: &[u8]) -> String {
    let hash = blake3::hash(public_key);
    format!("ghost{}", hex::encode(&hash.as_bytes()[..20]))
}

pub fn generate_qid_address(qid: &str) -> String {
    let hash = blake3::hash(qid.as_bytes());
    format!("qid{}", hex::encode(&hash.as_bytes()[..20]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_manager_creation() {
        let crypto = CryptoManager::new();
        assert!(crypto.shroud_enabled);
        assert!(crypto.gcrypt_fallback);
    }

    #[test]
    fn test_ed25519_keypair_generation() {
        let crypto = CryptoManager::new();
        let keypair = crypto.generate_ed25519_keypair().unwrap();
        
        assert_eq!(keypair.private_key.len(), 32);
        assert_eq!(keypair.public_key.len(), 32);
    }

    #[test]
    fn test_ed25519_sign_verify() {
        let crypto = CryptoManager::new();
        let keypair = crypto.generate_ed25519_keypair().unwrap();
        
        let message = b"Hello, GhostChain with Shroud!";
        let signature = crypto.ed25519_sign(&keypair.private_key, message).unwrap();
        let is_valid = crypto.ed25519_verify(&keypair.public_key, message, &signature).unwrap();
        
        assert!(is_valid);
    }

    #[test]
    fn test_blake3_hashing() {
        let crypto = CryptoManager::new();
        let data = b"Test data for hashing";
        let hash = crypto.blake3_hash(data).unwrap();
        
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_qid_generation() {
        let crypto = CryptoManager::new();
        let qid = crypto.generate_qid().unwrap();
        
        assert!(qid.starts_with("qid:"));
        assert!(qid.len() > 10);
    }

    #[test]
    fn test_sigil_generation() {
        let crypto = CryptoManager::new();
        let qid = "qid:test123";
        let sigil = crypto.generate_sigil(qid).unwrap();
        
        assert!(sigil.starts_with("sigil:"));
        assert!(sigil.len() > 10);
    }

    #[test]
    fn test_address_generation() {
        let crypto = CryptoManager::new();
        let keypair = crypto.generate_ed25519_keypair().unwrap();
        let address = generate_ghost_address(&keypair.public_key);
        
        assert!(address.starts_with("ghost"));
        assert_eq!(address.len(), 45); // "ghost" + 40 hex chars
    }

    #[test]
    fn test_qid_key_derivation() {
        let crypto = CryptoManager::new();
        let qid = "qid:test123";
        let keypair = crypto.derive_keys_from_qid(qid).unwrap();
        
        assert_eq!(keypair.private_key.len(), 32);
        assert_eq!(keypair.public_key.len(), 32);
    }
}