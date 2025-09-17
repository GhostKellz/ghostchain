// Identity Management
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub name: String,
    pub id: String,
    pub algorithm: String,
    pub public_key: String,
}

impl Identity {
    pub fn new(name: String, algorithm: String) -> Self {
        Self {
            name,
            id: format!("ghost{}", rand::random::<u64>()),
            algorithm,
            public_key: "0xpubkey123".to_string(), // Mock public key
        }
    }
}