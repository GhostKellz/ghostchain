// GSig - Ghost Signature Service
//
// Provides cryptographic signing and verification for GhostChain
// Supports multiple algorithms: Ed25519, Secp256k1, BLS, Post-Quantum

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use sha2::{Sha256, Digest};
use blake3;
use rand::rngs::OsRng;
use ghostchain_shared::types::Address;

/// Supported signature algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    Ed25519,
    Secp256k1,
    BLS,
    PostQuantum,
}

impl SignatureAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignatureAlgorithm::Ed25519 => "ed25519",
            SignatureAlgorithm::Secp256k1 => "secp256k1",
            SignatureAlgorithm::BLS => "bls",
            SignatureAlgorithm::PostQuantum => "post-quantum",
        }
    }
}

/// Hash algorithms for message preparation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
    Keccak256,
}

/// Cryptographic key pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoKeyPair {
    pub key_id: String,
    pub algorithm: SignatureAlgorithm,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>, // Should be encrypted in production
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: KeyMetadata,
}

/// Key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner: Option<Address>,
    pub purpose: KeyPurpose,
    pub tags: Vec<String>,
}

/// Key usage purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyPurpose {
    Signing,
    Authentication,
    Encryption,
    KeyAgreement,
    General,
}

/// Signature request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureRequest {
    pub key_id: String,
    pub message: Vec<u8>,
    pub hash_algorithm: Option<HashAlgorithm>,
    pub metadata: Option<SignatureMetadata>,
}

/// Signature metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureMetadata {
    pub purpose: String,
    pub timestamp: DateTime<Utc>,
    pub context: Option<String>,
}

/// Signature result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureResult {
    pub signature: Vec<u8>,
    pub algorithm: SignatureAlgorithm,
    pub hash_algorithm: HashAlgorithm,
    pub key_id: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<SignatureMetadata>,
}

/// Verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    pub message: Vec<u8>,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
    pub algorithm: SignatureAlgorithm,
    pub hash_algorithm: Option<HashAlgorithm>,
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub algorithm: SignatureAlgorithm,
    pub timestamp: DateTime<Utc>,
    pub error: Option<String>,
}

/// GSig service errors
#[derive(Debug, thiserror::Error)]
pub enum GSigError {
    #[error("Key not found: {key_id}")]
    KeyNotFound { key_id: String },

    #[error("Algorithm not supported: {algorithm:?}")]
    UnsupportedAlgorithm { algorithm: SignatureAlgorithm },

    #[error("Invalid signature format")]
    InvalidSignature,

    #[error("Invalid public key format")]
    InvalidPublicKey,

    #[error("Key has expired: {key_id}")]
    KeyExpired { key_id: String },

    #[error("Cryptographic error: {message}")]
    CryptographicError { message: String },
}

/// Ed25519 implementation
pub struct Ed25519Signer;

impl Ed25519Signer {
    pub fn generate_keypair() -> Result<(Vec<u8>, Vec<u8>)> {
        let mut csprng = OsRng {};
        let keypair = Keypair::generate(&mut csprng);
        Ok((keypair.public.to_bytes().to_vec(), keypair.secret.to_bytes().to_vec()))
    }

    pub fn sign(private_key: &[u8], message: &[u8]) -> Result<Vec<u8>> {
        let secret = SecretKey::from_bytes(private_key)
            .map_err(|e| GSigError::CryptographicError { message: e.to_string() })?;
        let public = PublicKey::from(&secret);
        let keypair = Keypair { secret, public };

        let signature = keypair.sign(message);
        Ok(signature.to_bytes().to_vec())
    }

    pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
        let public = PublicKey::from_bytes(public_key)
            .map_err(|_| GSigError::InvalidPublicKey)?;

        let sig = Signature::from_bytes(signature)
            .map_err(|_| GSigError::InvalidSignature)?;

        Ok(public.verify(message, &sig).is_ok())
    }
}

/// Message hashing utilities
pub struct MessageHasher;

impl MessageHasher {
    pub fn hash(message: &[u8], algorithm: HashAlgorithm) -> Vec<u8> {
        match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(message);
                hasher.finalize().to_vec()
            },
            HashAlgorithm::Blake3 => {
                blake3::hash(message).as_bytes().to_vec()
            },
            HashAlgorithm::Keccak256 => {
                // TODO: Implement Keccak256
                let mut hasher = Sha256::new();
                hasher.update(message);
                hasher.finalize().to_vec()
            },
        }
    }

    pub fn should_hash(algorithm: SignatureAlgorithm) -> bool {
        match algorithm {
            SignatureAlgorithm::Ed25519 => false, // Ed25519 handles hashing internally
            SignatureAlgorithm::Secp256k1 => true,
            SignatureAlgorithm::BLS => true,
            SignatureAlgorithm::PostQuantum => true,
        }
    }
}

/// Key management service
pub struct KeyManager {
    keys: RwLock<HashMap<String, CryptoKeyPair>>,
}

impl KeyManager {
    pub fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
        }
    }

    /// Generate a new key pair
    pub async fn generate_key(
        &self,
        algorithm: SignatureAlgorithm,
        metadata: KeyMetadata,
    ) -> Result<String> {
        let key_id = uuid::Uuid::new_v4().to_string();

        let (public_key, private_key) = match algorithm {
            SignatureAlgorithm::Ed25519 => Ed25519Signer::generate_keypair()?,
            _ => return Err(GSigError::UnsupportedAlgorithm { algorithm }.into()),
        };

        let keypair = CryptoKeyPair {
            key_id: key_id.clone(),
            algorithm,
            public_key,
            private_key,
            created_at: Utc::now(),
            expires_at: None,
            metadata,
        };

        let mut keys = self.keys.write().await;
        keys.insert(key_id.clone(), keypair);

        Ok(key_id)
    }

    /// Import an existing key pair
    pub async fn import_key(
        &self,
        algorithm: SignatureAlgorithm,
        public_key: Vec<u8>,
        private_key: Vec<u8>,
        metadata: KeyMetadata,
    ) -> Result<String> {
        let key_id = uuid::Uuid::new_v4().to_string();

        let keypair = CryptoKeyPair {
            key_id: key_id.clone(),
            algorithm,
            public_key,
            private_key,
            created_at: Utc::now(),
            expires_at: None,
            metadata,
        };

        let mut keys = self.keys.write().await;
        keys.insert(key_id.clone(), keypair);

        Ok(key_id)
    }

    /// Get public key
    pub async fn get_public_key(&self, key_id: &str) -> Result<Vec<u8>> {
        let keys = self.keys.read().await;
        let keypair = keys.get(key_id)
            .ok_or_else(|| GSigError::KeyNotFound { key_id: key_id.to_string() })?;

        // Check expiration
        if let Some(expires_at) = keypair.expires_at {
            if Utc::now() > expires_at {
                return Err(GSigError::KeyExpired { key_id: key_id.to_string() }.into());
            }
        }

        Ok(keypair.public_key.clone())
    }

    /// Get key info (without private key)
    pub async fn get_key_info(&self, key_id: &str) -> Result<CryptoKeyPair> {
        let keys = self.keys.read().await;
        let mut keypair = keys.get(key_id)
            .ok_or_else(|| GSigError::KeyNotFound { key_id: key_id.to_string() })?
            .clone();

        // Don't return private key in info
        keypair.private_key.clear();
        Ok(keypair)
    }

    /// List all keys (without private keys)
    pub async fn list_keys(&self) -> Vec<CryptoKeyPair> {
        let keys = self.keys.read().await;
        keys.values()
            .map(|keypair| {
                let mut info = keypair.clone();
                info.private_key.clear();
                info
            })
            .collect()
    }

    /// Delete a key
    pub async fn delete_key(&self, key_id: &str) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.remove(key_id)
            .ok_or_else(|| GSigError::KeyNotFound { key_id: key_id.to_string() })?;
        Ok(())
    }

    /// Get private key (internal use)
    async fn get_private_key(&self, key_id: &str) -> Result<(Vec<u8>, SignatureAlgorithm)> {
        let keys = self.keys.read().await;
        let keypair = keys.get(key_id)
            .ok_or_else(|| GSigError::KeyNotFound { key_id: key_id.to_string() })?;

        // Check expiration
        if let Some(expires_at) = keypair.expires_at {
            if Utc::now() > expires_at {
                return Err(GSigError::KeyExpired { key_id: key_id.to_string() }.into());
            }
        }

        Ok((keypair.private_key.clone(), keypair.algorithm))
    }
}

/// Main signature service
pub struct GSigService {
    key_manager: KeyManager,
}

impl GSigService {
    pub fn new() -> Self {
        Self {
            key_manager: KeyManager::new(),
        }
    }

    /// Sign a message
    pub async fn sign(&self, request: SignatureRequest) -> Result<SignatureResult> {
        let (private_key, algorithm) = self.key_manager.get_private_key(&request.key_id).await?;

        let hash_algorithm = request.hash_algorithm.unwrap_or(HashAlgorithm::Sha256);

        // Prepare message
        let message_to_sign = if MessageHasher::should_hash(algorithm) {
            MessageHasher::hash(&request.message, hash_algorithm)
        } else {
            request.message
        };

        // Sign with appropriate algorithm
        let signature = match algorithm {
            SignatureAlgorithm::Ed25519 => {
                Ed25519Signer::sign(&private_key, &message_to_sign)?
            },
            _ => return Err(GSigError::UnsupportedAlgorithm { algorithm }.into()),
        };

        Ok(SignatureResult {
            signature,
            algorithm,
            hash_algorithm,
            key_id: request.key_id,
            timestamp: Utc::now(),
            metadata: request.metadata,
        })
    }

    /// Verify a signature
    pub async fn verify(&self, request: VerificationRequest) -> Result<VerificationResult> {
        let hash_algorithm = request.hash_algorithm.unwrap_or(HashAlgorithm::Sha256);

        // Prepare message
        let message_to_verify = if MessageHasher::should_hash(request.algorithm) {
            MessageHasher::hash(&request.message, hash_algorithm)
        } else {
            request.message
        };

        // Verify with appropriate algorithm
        let valid = match request.algorithm {
            SignatureAlgorithm::Ed25519 => {
                Ed25519Signer::verify(&request.public_key, &message_to_verify, &request.signature)
                    .unwrap_or(false)
            },
            _ => {
                return Ok(VerificationResult {
                    valid: false,
                    algorithm: request.algorithm,
                    timestamp: Utc::now(),
                    error: Some(format!("Unsupported algorithm: {:?}", request.algorithm)),
                });
            }
        };

        Ok(VerificationResult {
            valid,
            algorithm: request.algorithm,
            timestamp: Utc::now(),
            error: None,
        })
    }

    /// Access to key manager
    pub fn key_manager(&self) -> &KeyManager {
        &self.key_manager
    }
}

/// GSig daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GSigConfig {
    pub bind_address: String,
    pub rpc_port: u16,
    pub grpc_port: u16,
    pub enable_key_generation: bool,
    pub enable_key_import: bool,
    pub max_keys_per_client: usize,
}

impl Default for GSigConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            rpc_port: 8554,
            grpc_port: 9554,
            enable_key_generation: true,
            enable_key_import: true,
            max_keys_per_client: 100,
        }
    }
}

/// GSig daemon service
pub struct GSigDaemon {
    service: GSigService,
    config: GSigConfig,
}

impl GSigDaemon {
    pub fn new(config: GSigConfig) -> Self {
        Self {
            service: GSigService::new(),
            config,
        }
    }

    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting GSig daemon on {}:{}", self.config.bind_address, self.config.rpc_port);

        // TODO: Start RPC server
        // TODO: Start gRPC server

        Ok(())
    }

    pub fn service(&self) -> &GSigService {
        &self.service
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GSigService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_generation() {
        let manager = KeyManager::new();

        let metadata = KeyMetadata {
            name: Some("Test Key".to_string()),
            description: Some("Test Ed25519 key".to_string()),
            owner: None,
            purpose: KeyPurpose::Signing,
            tags: vec!["test".to_string()],
        };

        let key_id = manager.generate_key(SignatureAlgorithm::Ed25519, metadata).await.unwrap();
        assert!(!key_id.is_empty());

        let public_key = manager.get_public_key(&key_id).await.unwrap();
        assert_eq!(public_key.len(), 32); // Ed25519 public key length
    }

    #[tokio::test]
    async fn test_signing_and_verification() {
        let service = GSigService::new();

        // Generate key
        let metadata = KeyMetadata {
            name: Some("Signing Key".to_string()),
            description: None,
            owner: None,
            purpose: KeyPurpose::Signing,
            tags: vec![],
        };

        let key_id = service.key_manager.generate_key(SignatureAlgorithm::Ed25519, metadata).await.unwrap();

        // Sign message
        let message = b"Hello, GhostChain!";
        let sign_request = SignatureRequest {
            key_id: key_id.clone(),
            message: message.to_vec(),
            hash_algorithm: None,
            metadata: None,
        };

        let signature_result = service.sign(sign_request).await.unwrap();
        assert!(!signature_result.signature.is_empty());

        // Verify signature
        let public_key = service.key_manager.get_public_key(&key_id).await.unwrap();
        let verify_request = VerificationRequest {
            message: message.to_vec(),
            signature: signature_result.signature,
            public_key,
            algorithm: SignatureAlgorithm::Ed25519,
            hash_algorithm: None,
        };

        let verification_result = service.verify(verify_request).await.unwrap();
        assert!(verification_result.valid);
    }

    #[test]
    fn test_message_hashing() {
        let message = b"test message";

        let sha256_hash = MessageHasher::hash(message, HashAlgorithm::Sha256);
        assert_eq!(sha256_hash.len(), 32);

        let blake3_hash = MessageHasher::hash(message, HashAlgorithm::Blake3);
        assert_eq!(blake3_hash.len(), 32);

        // Hashes should be different
        assert_ne!(sha256_hash, blake3_hash);
    }
}