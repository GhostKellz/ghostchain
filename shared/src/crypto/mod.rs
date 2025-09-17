// Legacy crypto module - kept for compatibility during migration
pub mod legacy {
    use ed25519_dalek::{Signature, SigningKey, VerifyingKey, Signer, Verifier};
    use rand::rngs::OsRng;
    use rand::RngCore;
    use blake3;
    use hex;
    use anyhow::{Result, anyhow};

    pub struct KeyPair {
        pub signing_key: SigningKey,
        pub verifying_key: VerifyingKey,
    }

    impl KeyPair {
        pub fn generate() -> Self {
            let mut csprng = OsRng;
            let mut secret_key = [0u8; 32];
            csprng.fill_bytes(&mut secret_key);
            let signing_key = SigningKey::from_bytes(&secret_key);
            let verifying_key = signing_key.verifying_key();
            
            Self {
                signing_key,
                verifying_key,
            }
        }
        
        pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
            let signing_key = SigningKey::from_bytes(bytes.try_into().map_err(|_| anyhow!("Invalid key length"))?);
            let verifying_key = signing_key.verifying_key();
            
            Ok(Self {
                signing_key,
                verifying_key,
            })
        }
        
        pub fn sign(&self, message: &[u8]) -> Signature {
            self.signing_key.sign(message)
        }
        
        pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
            self.verifying_key.verify(message, signature).is_ok()
        }
        
        pub fn address(&self) -> String {
            let public_key_bytes = self.verifying_key.as_bytes();
            let hash = blake3::hash(public_key_bytes);
            format!("ghost{}", hex::encode(&hash.as_bytes()[..20]))
        }
    }

    pub fn hash_data(data: &[u8]) -> String {
        let hash = blake3::hash(data);
        hex::encode(hash.as_bytes())
    }

    pub fn verify_signature(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
        let verifying_key = VerifyingKey::from_bytes(public_key.try_into()?)?;
        let sig = Signature::from_bytes(signature.try_into()?);
        
        Ok(verifying_key.verify(message, &sig).is_ok())
    }
}

// New Shroud-based crypto module
pub mod shroud_crypto;

// Re-export shroud crypto as the primary interface
pub use shroud_crypto::{CryptoManager, CryptoOperations, Ed25519KeyPair, Secp256k1KeyPair};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shroud_crypto_keypair_generation() {
        let crypto = CryptoManager::new();
        let keypair = crypto.generate_ed25519_keypair().unwrap();
        
        let message = b"Hello, GhostChain with Shroud!";
        let signature = crypto.ed25519_sign(&keypair.private_key, message).unwrap();
        let is_valid = crypto.ed25519_verify(&keypair.public_key, message, &signature).unwrap();
        
        assert!(is_valid);
    }
    
    #[test]
    fn test_shroud_crypto_hashing() {
        let crypto = CryptoManager::new();
        let data = b"Test data for hashing";
        let hash = crypto.blake3_hash(data).unwrap();
        
        assert_eq!(hash.len(), 32);
    }
}