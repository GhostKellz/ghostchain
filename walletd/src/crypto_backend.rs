// Enhanced Crypto Backend
use anyhow::Result;

pub struct CryptoBackend;

impl CryptoBackend {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
    
    pub fn supported_algorithms(&self) -> Vec<String> {
        vec!["ed25519".to_string(), "secp256k1".to_string()]
    }
}