use crate::types::*;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct ContractStorage {
    contract_states: HashMap<ContractId, HashMap<String, Vec<u8>>>,
}

impl ContractStorage {
    pub fn new() -> Self {
        Self {
            contract_states: HashMap::new(),
        }
    }
    
    pub fn read(&self, contract_id: &ContractId, key: &str) -> Option<Vec<u8>> {
        self.contract_states
            .get(contract_id)
            .and_then(|state| state.get(key))
            .cloned()
    }
    
    pub fn write(&mut self, contract_id: &ContractId, key: &str, value: Vec<u8>) {
        let state = self.contract_states
            .entry(contract_id.clone())
            .or_insert_with(HashMap::new);
        state.insert(key.to_string(), value);
    }
    
    pub fn delete(&mut self, contract_id: &ContractId, key: &str) -> bool {
        if let Some(state) = self.contract_states.get_mut(contract_id) {
            state.remove(key).is_some()
        } else {
            false
        }
    }
    
    pub fn has_key(&self, contract_id: &ContractId, key: &str) -> bool {
        self.contract_states
            .get(contract_id)
            .map(|state| state.contains_key(key))
            .unwrap_or(false)
    }
    
    pub fn get_all_keys(&self, contract_id: &ContractId) -> Vec<String> {
        self.contract_states
            .get(contract_id)
            .map(|state| state.keys().cloned().collect())
            .unwrap_or_default()
    }
    
    pub fn get_state_size(&self, contract_id: &ContractId) -> usize {
        self.contract_states
            .get(contract_id)
            .map(|state| {
                state.iter()
                    .map(|(k, v)| k.len() + v.len())
                    .sum()
            })
            .unwrap_or(0)
    }
    
    pub fn clear_contract_state(&mut self, contract_id: &ContractId) {
        self.contract_states.remove(contract_id);
    }
    
    // Specialized storage methods for common data types
    
    pub fn read_string(&self, contract_id: &ContractId, key: &str) -> Result<Option<String>> {
        if let Some(data) = self.read(contract_id, key) {
            let s = String::from_utf8(data)
                .map_err(|e| anyhow!("Failed to decode string from storage: {}", e))?;
            Ok(Some(s))
        } else {
            Ok(None)
        }
    }
    
    pub fn write_string(&mut self, contract_id: &ContractId, key: &str, value: &str) {
        self.write(contract_id, key, value.as_bytes().to_vec());
    }
    
    pub fn read_u128(&self, contract_id: &ContractId, key: &str) -> Result<Option<u128>> {
        if let Some(data) = self.read(contract_id, key) {
            if data.len() != 16 {
                return Err(anyhow!("Invalid u128 data length: {}", data.len()));
            }
            let mut bytes = [0u8; 16];
            bytes.copy_from_slice(&data);
            Ok(Some(u128::from_le_bytes(bytes)))
        } else {
            Ok(None)
        }
    }
    
    pub fn write_u128(&mut self, contract_id: &ContractId, key: &str, value: u128) {
        self.write(contract_id, key, value.to_le_bytes().to_vec());
    }
    
    pub fn read_json<T>(&self, contract_id: &ContractId, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if let Some(data) = self.read(contract_id, key) {
            let value = serde_json::from_slice(&data)
                .map_err(|e| anyhow!("Failed to deserialize JSON from storage: {}", e))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
    
    pub fn write_json<T>(&mut self, contract_id: &ContractId, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let data = serde_json::to_vec(value)
            .map_err(|e| anyhow!("Failed to serialize JSON for storage: {}", e))?;
        self.write(contract_id, key, data);
        Ok(())
    }
    
    // Domain-specific storage helpers
    
    pub fn store_domain_data(&mut self, contract_id: &ContractId, domain: &str, data: &DomainData) -> Result<()> {
        let key = format!("domain:{}", domain);
        self.write_json(contract_id, &key, data)
    }
    
    pub fn load_domain_data(&self, contract_id: &ContractId, domain: &str) -> Result<Option<DomainData>> {
        let key = format!("domain:{}", domain);
        self.read_json(contract_id, &key)
    }
    
    pub fn store_domain_owner(&mut self, contract_id: &ContractId, domain: &str, owner: &Address) {
        let key = format!("owner:{}", domain);
        self.write_string(contract_id, &key, owner);
    }
    
    pub fn load_domain_owner(&self, contract_id: &ContractId, domain: &str) -> Result<Option<Address>> {
        let key = format!("owner:{}", domain);
        self.read_string(contract_id, &key)
    }
    
    pub fn store_owner_domains(&mut self, contract_id: &ContractId, owner: &Address, domains: &[String]) -> Result<()> {
        let key = format!("owner_domains:{}", owner);
        self.write_json(contract_id, &key, &domains.to_vec())
    }
    
    pub fn load_owner_domains(&self, contract_id: &ContractId, owner: &Address) -> Result<Option<Vec<String>>> {
        let key = format!("owner_domains:{}", owner);
        self.read_json(contract_id, &key)
    }
    
    // Token balance storage helpers
    
    pub fn store_token_balance(&mut self, contract_id: &ContractId, address: &Address, token: &TokenType, balance: u128) {
        let key = format!("balance:{}:{:?}", address, token);
        self.write_u128(contract_id, &key, balance);
    }
    
    pub fn load_token_balance(&self, contract_id: &ContractId, address: &Address, token: &TokenType) -> Result<u128> {
        let key = format!("balance:{}:{:?}", address, token);
        Ok(self.read_u128(contract_id, &key)?.unwrap_or(0))
    }
    
    pub fn store_total_supply(&mut self, contract_id: &ContractId, token: &TokenType, supply: u128) {
        let key = format!("total_supply:{:?}", token);
        self.write_u128(contract_id, &key, supply);
    }
    
    pub fn load_total_supply(&self, contract_id: &ContractId, token: &TokenType) -> Result<u128> {
        let key = format!("total_supply:{:?}", token);
        Ok(self.read_u128(contract_id, &key)?.unwrap_or(0))
    }
}

impl Default for ContractStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct StorageOperation {
    pub operation_type: StorageOperationType,
    pub contract_id: ContractId,
    pub key: String,
    pub value: Option<Vec<u8>>,
    pub gas_cost: u128,
}

#[derive(Debug, Clone)]
pub enum StorageOperationType {
    Read,
    Write,
    Delete,
}

pub struct StorageTracker {
    operations: Vec<StorageOperation>,
    total_gas: u128,
}

impl StorageTracker {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            total_gas: 0,
        }
    }
    
    pub fn track_read(&mut self, contract_id: ContractId, key: String, gas_cost: u128) {
        self.operations.push(StorageOperation {
            operation_type: StorageOperationType::Read,
            contract_id,
            key,
            value: None,
            gas_cost,
        });
        self.total_gas += gas_cost;
    }
    
    pub fn track_write(&mut self, contract_id: ContractId, key: String, value: Vec<u8>, gas_cost: u128) {
        self.operations.push(StorageOperation {
            operation_type: StorageOperationType::Write,
            contract_id,
            key,
            value: Some(value),
            gas_cost,
        });
        self.total_gas += gas_cost;
    }
    
    pub fn track_delete(&mut self, contract_id: ContractId, key: String, gas_cost: u128) {
        self.operations.push(StorageOperation {
            operation_type: StorageOperationType::Delete,
            contract_id,
            key,
            value: None,
            gas_cost,
        });
        self.total_gas += gas_cost;
    }
    
    pub fn get_total_gas(&self) -> u128 {
        self.total_gas
    }
    
    pub fn get_operations(&self) -> &[StorageOperation] {
        &self.operations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_storage_operations() {
        let mut storage = ContractStorage::new();
        let contract_id = "test_contract".to_string();
        
        // Test write and read
        storage.write(&contract_id, "key1", b"value1".to_vec());
        assert_eq!(storage.read(&contract_id, "key1"), Some(b"value1".to_vec()));
        
        // Test delete
        assert!(storage.delete(&contract_id, "key1"));
        assert_eq!(storage.read(&contract_id, "key1"), None);
        
        // Test has_key
        storage.write(&contract_id, "key2", b"value2".to_vec());
        assert!(storage.has_key(&contract_id, "key2"));
        assert!(!storage.has_key(&contract_id, "nonexistent"));
    }
    
    #[test]
    fn test_typed_storage() {
        let mut storage = ContractStorage::new();
        let contract_id = "test_contract".to_string();
        
        // Test string storage
        storage.write_string(&contract_id, "str_key", "hello world");
        assert_eq!(storage.read_string(&contract_id, "str_key").unwrap(), Some("hello world".to_string()));
        
        // Test u128 storage
        storage.write_u128(&contract_id, "num_key", 12345u128);
        assert_eq!(storage.read_u128(&contract_id, "num_key").unwrap(), Some(12345u128));
        
        // Test JSON storage
        let domain_data = DomainData {
            domain: "test.ghost".to_string(),
            owner: "owner123".to_string(),
            records: vec![],
            contract_address: None,
            last_updated: 1234567890,
            expiry: None,
            signature: vec![],
        };
        
        storage.write_json(&contract_id, "domain_key", &domain_data).unwrap();
        let loaded: Option<DomainData> = storage.read_json(&contract_id, "domain_key").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().domain, "test.ghost");
    }
    
    #[test]
    fn test_domain_storage_helpers() {
        let mut storage = ContractStorage::new();
        let contract_id = "domain_registry".to_string();
        
        let domain_data = DomainData {
            domain: "example.ghost".to_string(),
            owner: "owner456".to_string(),
            records: vec![],
            contract_address: None,
            last_updated: 1234567890,
            expiry: None,
            signature: vec![],
        };
        
        // Store domain data
        storage.store_domain_data(&contract_id, "example.ghost", &domain_data).unwrap();
        
        // Load domain data
        let loaded = storage.load_domain_data(&contract_id, "example.ghost").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().owner, "owner456");
        
        // Store and load domain owner
        storage.store_domain_owner(&contract_id, "example.ghost", "owner456");
        let owner = storage.load_domain_owner(&contract_id, "example.ghost").unwrap();
        assert_eq!(owner, Some("owner456".to_string()));
    }
}