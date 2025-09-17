// Wallet Management
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub name: String,
    pub algorithm: String,
    pub address: String,
    pub balance: HashMap<String, u128>,
}

impl Wallet {
    pub fn new(name: String, algorithm: String) -> Self {
        Self {
            name,
            algorithm,
            address: "0x1234567890abcdef".to_string(), // Mock address
            balance: HashMap::new(),
        }
    }
}